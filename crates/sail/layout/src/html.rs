//! Minimal HTML parser for **preprocessed** `.sail` source.
//!
//! `sail_compiler::parse::preprocess` has already done the heavy lifting:
//! `<column>` is `<div class="sail-col …">`, `sx="…"` is `class="sx-…"`. We
//! just need a stream-friendly scanner that produces a flat element tree with
//! raw `class=` / `style=` / `style:--foo=` attributes — enough for style
//! resolution, not enough for rendering.
//!
//! - Quote-aware (`"…"`, `'…'`).
//! - Comment-aware (`<!--…-->` and skipped entirely).
//! - Ignores `<style>`, `<controller>` blocks (no layout semantics).
//! - Tracks byte ranges (`span`) so a diagnostic can point at the original
//!   `.sail` source.

use std::collections::HashMap;

/// A single element (or pseudo-element — text and comments are skipped).
/// `children` are indices into the parent parse's vec so it round-trips as a
/// tree without reallocation.
#[derive(Debug, Clone, Default)]
pub struct Element {
    pub tag: String,
    /// Final, merged `class="…"` attribute (post preprocess + post
    /// `class:name={…}` accumulation by the runtime, which we approximate as
    /// the static class string the compiler emits).
    pub class: String,
    /// Style attributes: `style:--foo="value"` and `style:bar="value"` and
    /// bare `style="…"`. Lowercased key → raw value (no parsing done at this
    /// layer — style resolution decodes CSS lengths from these).
    pub style: HashMap<String, String>,
    /// Explicit grid-column span from `data-col-span`.
    pub col_span: Option<u16>,
    /// Explicit grid-row span from `data-row-span`.
    pub row_span: Option<u16>,
    /// Span into the ORIGINAL (un-preprocessed) source. We keep both: the
    /// preprocessed span for in-file precision, plus the parent source's
    /// byte offset of the preprocessed start, so we can re-locate in the
    /// original once we know the diagnostics to emit. For the spike we just
    /// carry the preprocessed span (the parser only sees the lowered text);
    /// pointing back to the original is a future pass.
    pub span: (usize, usize),
    /// Indices into the containing `Vec<Element>` (a tree flatten).
    pub children: Vec<usize>,
    /// Parent index, or `None` for the synthetic document root.
    pub parent: Option<usize>,
    /// Text content directly inside this element (between its open tag and
    /// the first child or close tag). Whitespace is collapsed to a single
    /// space the way a browser would; an empty string means "no text leaf".
    pub text: String,
}

impl Element {
    /// True if this element's size is **unknown** to the layout pass.
    /// An element is unknown when:
    ///
    /// 1. **It is a custom element** (web component) — its body is rendered
    ///    by JS at runtime, so it has no compile-time intrinsic size.
    ///    W3C naming rule: a tag name must contain at least one hyphen.
    ///
    /// 2. **It is a replaced element taffy does not model.** Browsers give
    ///    these intrinsic sizes (300×150 for iframe by default; viewBox-driven
    ///    for svg) that taffy treats as zero. Reporting a parent as "collapsed"
    ///    because its iframe-only child measures 0 in taffy is a taffy
    ///    limitation masquerading as a layout bug.
    pub fn is_unknown(&self) -> bool {
        if self.tag.is_empty() {
            return false;
        }
        if self.tag.contains('-') {
            return true;
        }
        matches!(
            self.tag.as_str(),
            "iframe" | "video" | "audio" | "embed" | "object" | "canvas" | "svg"
                | "picture" | "source" | "track" | "math" | "slot" | "template"
        )
    }
}

/// Parse the **preprocessed** source for one component into a forest of
/// elements. `Vec<Element>` is the tree flattened: each node's `children`
/// holds the indices of its direct descendants, in source order. The first
/// node (index 0) is a synthetic document root whose own children are the
/// top-level elements encountered.
pub fn parse_source(source: &str) -> Vec<Element> {
    let mut out: Vec<Element> = Vec::new();
    let synthetic = Element::default();
    out.push(synthetic);
    let mut stack: Vec<usize> = vec![0];
    let bytes = source.as_bytes();
    let mut pos: usize = 0;


    while pos < bytes.len() {
        // Comments.
        if let Some(rel) = find_from(bytes, pos, b"<!--") {
            if let Some(end_rel) = find_from(bytes, rel + 4, b"-->") {
                pos = end_rel + 3;
                continue;
            } else {
                break;
            }
        }

        // Tag open.
        if let Some(rel_lt) = find_from(bytes, pos, b"<") {
            if rel_lt == pos {
                // Could be a tag right here. Try parse_open_tag.
                if let Some((consumed, elem_opt, is_close, raw_text_to)) = parse_open_tag(source, pos) {
                    pos = consumed;
                    match (is_close, elem_opt) {
                        (false, Some(elem)) => {
                            let idx = out.len();
                            let parent = *stack.last().unwrap_or(&0);
                            let mut e = elem;
                            e.parent = Some(parent);
                            out.push(e);
                            out[parent].children.push(idx);
                            stack.push(idx);
                        }
                        (true, _) => {
                            if stack.len() > 1 {
                                stack.pop();
                            }
                        }
                        _ => {}
                    }
                    // If we just consumed a raw-text open tag (controller / style /
                    // script), skip everything until its close tag — the content is
                    // opaque to the box tree and trying to parse it would confuse
                    // the leaf-tagger with TS types like `<string>` and `{`.
                    if !is_close && raw_text_to.is_some() {
                        let needle = raw_text_to.unwrap();
                        if let Some(end_rel) = find_from(bytes, pos, needle.as_bytes()) {
                            pos = end_rel + needle.len();
                            // The raw-text tag's closing tag is a sibling's child in
                            // the parser, not its own — so we keep it off the stack
                            // by popping now. The </tag> will be consumed in the
                            // next iteration as a normal close.
                        }
                    }
                    continue;
                }
            }
            // Text content between tags — accumulate it onto whichever
            // element is currently on the stack. The measure function will
            // collapse whitespace the way a browser would.
            if let Some(&top) = stack.last() {
                let raw = &source[pos..rel_lt];
                let trimmed = raw.trim();
                if !trimmed.is_empty() {
                    if !out[top].text.is_empty() {
                        out[top].text.push(' ');
                    }
                    out[top].text.push_str(trimmed);
                }
            }
            pos = rel_lt;
            continue;
        }

        break;
    }

    out
}

fn find_from(hay: &[u8], start: usize, needle: &[u8]) -> Option<usize> {
    if start >= hay.len() {
        return None;
    }
    hay[start..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + start)
}

/// Try to parse an open or close tag at `pos`. Returns:
///   - `Some((next_pos, Some(element), false, None))` for a normal open tag,
///   - `Some((next_pos, None, false, Some(close_needle)))` for a raw-text tag
///     whose body the caller should skip (controller / style / script),
///   - `Some((next_pos, None, true, None))` for a close tag,
///   - `None` if `pos` doesn't begin a tag.
fn parse_open_tag(
    source: &str,
    pos: usize,
) -> Option<(usize, Option<Element>, bool, Option<String>)> {
    let bytes = source.as_bytes();
    if bytes.get(pos) != Some(&b'<') {
        return None;
    }
    // `</foo>`?
    if bytes.get(pos + 1) == Some(&b'/') {
        let close_end = source[pos..].find('>')?;
        return Some((pos + close_end + 1, None, true, None));
    }
    // `<!…>` (declaration) — skip.
    if bytes.get(pos + 1) == Some(&b'!') {
        let close_end = source[pos..].find('>')?;
        return Some((pos + close_end + 1, None, false, None));
    }

    let (open_end, self_close) = find_tag_end(bytes, pos)?;
    let tag_text = &source[pos + 1..open_end];
    let tag_text = tag_text.trim_end_matches('/').trim();
    let mut parts = tag_text.splitn(2, char::is_whitespace);
    let tag = parts.next()?.to_ascii_lowercase();
    let attrs_raw = parts.next().unwrap_or("").to_string();

    // Raw-text tags: their body is opaque TS / CSS / JS — not parsed for box
    // semantics. The caller should consume the body up to the matching close
    // tag before resuming the normal scan.
    const RAW_TEXT: &[&str] = &["controller", "style", "script"];
    if RAW_TEXT.contains(&tag.as_str()) {
        return Some((
            open_end + 1,
            None,
            false,
            Some(format!("</{}>", tag)),
        ));
    }

    // Skip non-visual blocks entirely (we don't even record an element for
    // these — props are part of the IR, not the layout tree).
    const SKIP: &[&str] = &[
        "data", "client", "prop", "variants",
        "theme", "host", "component", "while-loading", "while-error",
    ];
    if SKIP.contains(&tag.as_str()) {
        return Some((open_end + 1, None, false, None));
    }


    let mut elem = Element {
        tag,
        span: (pos, open_end + 1),
        ..Default::default()
    };
    parse_attrs(&attrs_raw, &mut elem);

    // Merge any pre-existing class= attributes (sx may set class without our
    // control; the compiler merges them — we approximate by concatenating
    // distinct classes split on whitespace).
    elem.class = merge_class(&elem.class);

    if self_close {
        Some((open_end + 1, Some(elem), false, None))
    } else {
        Some((open_end + 1, Some(elem), false, None))
    }
}
/// and unquoted values; bare attributes get an empty value.
fn parse_attrs(text: &str, elem: &mut Element) {
    let mut i = 0;
    let bytes = text.as_bytes();
    while i < bytes.len() {
        // Skip whitespace.
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        // Read attribute name (until `=`, whitespace, or `>`).
        let name_start = i;
        while i < bytes.len()
            && !bytes[i].is_ascii_whitespace()
            && bytes[i] != b'='
        {
            i += 1;
        }
        let name = text[name_start..i].to_ascii_lowercase();
        if name.is_empty() {
            // Stray character — skip one byte to avoid an infinite loop.
            i += 1;
            continue;
        }
        // Optional `=`.
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            // Read value, possibly quoted.
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let q = bytes[i];
                i += 1;
                let v_start = i;
                while i < bytes.len() && bytes[i] != q {
                    if bytes[i] == b'\\' && i + 1 < bytes.len() {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                let value = text[v_start..i].to_string();
                if i < bytes.len() {
                    i += 1; // closing quote
                }
                set_attr(elem, &name, value);
            } else {
                let v_start = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
                let value = text[v_start..i].to_string();
                set_attr(elem, &name, value);
            }
        } else {
            // Bare attribute (e.g. `<input disabled>`).
            set_attr(elem, &name, String::new());
        }
    }
}

fn set_attr(elem: &mut Element, name: &str, value: String) {
    if name == "class" {
        if elem.class.is_empty() {
            elem.class = value;
        } else {
            elem.class.push(' ');
            elem.class.push_str(&value);
        }
    } else if let Some(var_name) = name.strip_prefix("style:") {
        elem.style.insert(var_name.to_string(), value);
    } else if name == "style" {
        // Bare `style="…"` — parse simple `prop:value` pairs into the same map.
        for decl in value.split(';') {
            let decl = decl.trim();
            if decl.is_empty() {
                continue;
            }
            if let Some((k, v)) = decl.split_once(':') {
                elem.style.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
    } else if name == "data-col-span" {
        elem.col_span = value.parse().ok();
    } else if name == "data-row-span" {
        elem.row_span = value.parse().ok();
    }
    // Other attrs are ignored at this layer — text/event/etc. don't affect
    // box geometry.
}

fn merge_class(c: &str) -> String {
    let mut seen = std::collections::HashSet::<String>::new();
    let mut out = Vec::<String>::new();
    for tok in c.split_whitespace() {
        if seen.insert(tok.to_string()) {
            out.push(tok.to_string());
        }
    }
    out.join(" ")
}

/// Return the index of the `>` that closes an open tag, accounting for
/// quoted attribute values, and whether the tag is self-closing (`<foo …/>`).
fn find_tag_end(bytes: &[u8], pos: usize) -> Option<(usize, bool)> {
    let mut i = pos + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' && i + 1 < bytes.len() {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                if i < bytes.len() {
                    i += 1;
                }
            }
            b'\'' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'\'' {
                    if bytes[i] == b'\\' && i + 1 < bytes.len() {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                if i < bytes.len() {
                    i += 1;
                }
            }
            b'>' => return Some((i, false)),
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'>' => {
                return Some((i, true));
            }
            _ => i += 1,
        }
    }
    None
}

/// Where the document's intrinsic viewport comes from in raw author source:
/// `<column viewport>` declares one. Without it, the bin defaults viewport.
pub fn source_viewport(source: &str) -> Option<(f32, f32)> {
    // Look for `viewport="800x600"` etc. Cheap, sufficient for spike.
    let needle = "viewport=\"";
    let start = source.find(needle)? + needle.len();
    let rest = &source[start..];
    let end = rest.find('"')?;
    let v = &rest[..end];
    let (w, h) = v.split_once('x')?;
    Some((w.parse().ok()?, h.parse().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_lowered_column_with_class() {
        let src = r#"<div class="sail-col sail-ai-stretch sail-g-4"><span></span></div>"#;
        let tree = parse_source(src);
        assert!(tree.len() >= 2, "should have root + div + span");
        let div = tree.iter().find(|e| e.tag == "div").unwrap();
        assert!(div.class.contains("sail-col"));
        assert!(div.class.contains("sail-ai-stretch"));
        assert!(div.class.contains("sail-g-4"));
    }

    #[test]
    fn skips_controller_blocks() {
        let src = r#"<controller>x</controller><div></div>"#;
        let tree = parse_source(src);
        assert_eq!(tree.iter().filter(|e| e.tag == "div").count(), 1);
    }

    #[test]
    fn captures_text_on_open_element() {
        let src = r#"<h1>  Hello World  </h1><p>foo bar</p>"#;
        let tree = parse_source(src);
        let h1 = tree.iter().find(|e| e.tag == "h1").unwrap();
        let p = tree.iter().find(|e| e.tag == "p").unwrap();
        assert_eq!(h1.text, "Hello World");
        assert_eq!(p.text, "foo bar");
    }

    #[test]
    fn whitespace_only_text_stays_empty() {
        let src = r#"<div>   </div>"#;
        let tree = parse_source(src);
        let div = tree.iter().find(|e| e.tag == "div").unwrap();
        assert!(div.text.is_empty());
    }

    #[test]
    fn is_unknown_recognises_custom_and_replaced_elements() {
        let cases = [
            // known
            ("div", false),
            ("p", false),
            ("h1", false),
            ("img", false),
            ("input", false),
            // custom elements
            ("sail-code-editor", true),
            ("my-widget", true),
            // replaced elements taffy doesn't model
            ("iframe", true),
            ("video", true),
            ("svg", true),
            ("slot", true),
            ("template", true),
            ("", false),
        ];
        for (tag, expected) in cases {
            let e = Element { tag: tag.to_string(), ..Default::default() };
            assert_eq!(e.is_unknown(), expected, "tag={tag:?}");
        }
    }
}
