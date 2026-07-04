//! Source preprocessor — the single, Rust-only path. (There is no JS twin:
//! the legacy `@ship/sui-dom` preprocess was removed when every directive was
//! made to round-trip through this crate; deck's bundler and the Node test
//! harness both compile in-process through `sui_compiler`.)
//!
//! Transforms, in order, before the HTML parser sees the source:
//!  1. Quote unquoted brace attributes — `disabled={x || y}` becomes
//!     `disabled="{x || y}"` with `"` inside the expression escaped to `&quot;`.
//!  2. Expand non-void self-closing tags — `<data … />` becomes `<data …></data>`.
//!  3. Lower `sx="…"` surface attributes → atomic `class`es.
//!  4. Lower the layout keywords (`<column>`/`<row>`/…) → `<div>` + atomic classes.
//! Steps 3–4 emit deduped CSS that `preprocess` injects as one `<style>`.

const VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
    "source", "track", "wbr",
];

pub fn preprocess(source: &str) -> String {
    // Brace-quoting + self-close expansion normalize the raw source first. Then
    // the two compile-time lowerings run on that HTML: `sx=` attributes → atomic
    // surface classes, then the layout keywords (`<column>`/`<row>`/…) → `<div>`
    // + atomic layout classes. Both return the deduped CSS they need; we inject
    // it as ONE `<style>` at the very end so the brace-attr quoter never sees its
    // `{…}`-bearing rules. sx runs before layout so a layout element's own `sx=`
    // is already a `class` that the layout rewrite folds into the div's classes.
    // Off-vocab values are rejected earlier by the lints (`sui-sx` / `sui-layout`)
    // reading the raw source; here we only lower valid ones.
    let s = expand_self_close(&quote_brace_attrs(source));
    let (s, sx_css) = lower_sx(&s);
    let (s, layout_css) = lower_layout(&s);
    let css = format!("{sx_css}{layout_css}");
    if css.is_empty() {
        s
    } else {
        format!("<style>{css}</style>\n{s}")
    }
}

/// Lower every static `sx="…"` attribute to `class="sx-…"`, returning the
/// rewritten body and the deduped atomic CSS (the caller injects it into the
/// shared `<style>`). The prop→CSS mapping is `sui_lint::rules::sx::lower` — the
/// same vocabulary the lint validates.
fn lower_sx(source: &str) -> (String, String) {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len() + 256);
    let mut css = String::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut i = 0;
    let mut last = 0;
    while i < bytes.len() {
        if source[i..].starts_with("<!--") {
            match source[i..].find("-->") {
                Some(end) => {
                    i += end + 3;
                    continue;
                }
                None => break,
            }
        }
        if i > 0
            && bytes[i - 1].is_ascii_whitespace()
            && source[i..].starts_with("sx")
            && is_eq_quote(&bytes[i + 2..])
        {
            let mut j = i + 2;
            while j < bytes.len() && bytes[j] != b'"' && bytes[j] != b'{' {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'"' {
                let vstart = j + 1;
                if let Some(rel) = source[vstart..].find('"') {
                    let val = &source[vstart..vstart + rel];
                    out.push_str(&source[last..i]);
                    out.push_str("class=\"");
                    out.push_str(&lower_sx_value(val, &mut css, &mut seen));
                    out.push('"');
                    i = vstart + rel + 1;
                    last = i;
                    continue;
                }
            }
        }
        // Advance one FULL UTF-8 char. A bare `i += 1` lands mid-codepoint on
        // any multi-byte char (…, emoji, accents) and the next `source[i..]`
        // slice panics — the crash the async golden fixtures hit. Mirrors the
        // char-safe stepping in `quote_brace_attrs` / `expand_self_close`.
        let mut n = i + 1;
        while n < bytes.len() && (bytes[n] & 0xC0) == 0x80 {
            n += 1;
        }
        i = n;
    }
    out.push_str(&source[last..]);
    (out, css)
}

fn lower_sx_value(
    val: &str,
    css: &mut String,
    seen: &mut std::collections::HashSet<String>,
) -> String {
    let mut classes: Vec<String> = Vec::new();
    for pair in val.split_whitespace() {
        let Some((prop, token)) = pair.split_once(':') else {
            continue;
        };
        let sanitized: String = token.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
        let class = format!("sx-{prop}-{sanitized}");
        if let Some(decls) = sui_lint::rules::sx::lower(prop, token) {
            if seen.insert(class.clone()) {
                let body = decls
                    .iter()
                    .map(|(p, v)| format!("{p}:{v}"))
                    .collect::<Vec<_>>()
                    .join(";");
                css.push_str(&format!(".{class}{{{body}}}"));
            }
        }
        classes.push(class);
    }
    classes.join(" ")
}

/// After the `sx` name: optional ws, `=`, optional ws, then a `"`.
fn is_eq_quote(rest: &[u8]) -> bool {
    let mut k = 0;
    while k < rest.len() && rest[k].is_ascii_whitespace() {
        k += 1;
    }
    if k >= rest.len() || rest[k] != b'=' {
        return false;
    }
    k += 1;
    while k < rest.len() && rest[k].is_ascii_whitespace() {
        k += 1;
    }
    k < rest.len() && rest[k] == b'"'
}

/// Lower each layout keyword (`<column>`, `<row>`, `<grid>`, `<center>`,
/// `<stack>`) to a `<div>` + atomic layout classes, returning the rewritten body
/// and the deduped CSS. Axis props (gap/align/justify/min/flow) become classes
/// and drop off the tag; every other attribute passes through, and an existing
/// `class` (static or dynamic) is merged into the computed classes. Close tags
/// become `</div>`. The class/CSS mapping is `sui_lint::rules::layout` — the same
/// vocabulary the `sui-layout` lint validates against the raw source.
fn lower_layout(source: &str) -> (String, String) {
    use sui_lint::rules::layout;
    let opens = layout::scan_open_tags(source);
    if opens.is_empty() {
        return (source.to_string(), String::new());
    }
    let open_at: std::collections::HashMap<usize, &layout::OpenTag> =
        opens.iter().map(|t| (t.start, t)).collect();

    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len() + 256);
    let mut css = String::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut i = 0;
    while i < bytes.len() {
        if source[i..].starts_with("<!--") {
            match source[i..].find("-->") {
                Some(end) => {
                    out.push_str(&source[i..i + end + 3]);
                    i += end + 3;
                    continue;
                }
                None => break,
            }
        }
        if let Some(t) = open_at.get(&i) {
            let axis: Vec<(String, String)> = t
                .attrs
                .iter()
                .filter(|(n, _)| matches!(n.as_str(), "gap" | "align" | "justify" | "min" | "flow"))
                .cloned()
                .collect();
            if let Some((classes, rules)) = layout::lower(&t.tag, &axis) {
                for r in rules {
                    if seen.insert(r.clone()) {
                        css.push_str(&r);
                    }
                }
                let mut static_class = classes.join(" ");
                let mut dyn_class: Option<String> = None;
                let mut rest = String::new();
                for (name, value) in &t.attrs {
                    match name.as_str() {
                        "gap" | "align" | "justify" | "min" | "flow" => {}
                        "class" if value.starts_with('{') => {
                            dyn_class = Some(value[1..value.len() - 1].to_string());
                        }
                        "class" => {
                            if !value.is_empty() {
                                static_class.push(' ');
                                static_class.push_str(value);
                            }
                        }
                        _ => {
                            rest.push(' ');
                            rest.push_str(name);
                            if !value.is_empty() {
                                rest.push_str("=\"");
                                rest.push_str(value);
                                rest.push('"');
                            }
                        }
                    }
                }
                out.push_str("<div class=");
                match dyn_class {
                    // Fold computed classes into the user's dynamic binding, as a
                    // quoted brace attr (what quote_brace_attrs would have emitted).
                    Some(expr) => {
                        out.push_str(&format!("\"{{'{static_class} ' + ({expr})}}\""));
                    }
                    None => {
                        out.push_str(&format!("\"{static_class}\""));
                    }
                }
                out.push_str(&rest);
                out.push('>');
                if t.self_closing {
                    out.push_str("</div>");
                }
                i = t.end;
                continue;
            }
        }
        // Layout close tag → `</div>` (every layout tag lowers to a div).
        if source[i..].starts_with("</")
            && let Some(tag) = layout::TAGS.iter().find(|&&tg| {
                source[i + 2..].starts_with(tg) && bytes.get(i + 2 + tg.len()) == Some(&b'>')
            })
        {
            out.push_str("</div>");
            i += 2 + tag.len() + 1;
            continue;
        }
        let mut n = i + 1;
        while n < bytes.len() && (bytes[n] & 0xC0) == 0x80 {
            n += 1;
        }
        out.push_str(&source[i..n]);
        i = n;
    }
    out.push_str(&source[i..]);
    (out, css)
}

fn quote_brace_attrs(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            // Append one UTF-8 char (1–4 bytes). `bytes[i] as char` would
            // mangle multi-byte sequences like `…` (e2 80 a6) into latin-1
            // codepoints and re-encode them as gibberish.
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j] & 0xC0) == 0x80 {
                j += 1;
            }
            out.push_str(&source[i..j]);
            i = j;
            continue;
        }
        // HTML comments — skip the body verbatim. The tag-mode quote
        // tracker doesn't understand comments; an apostrophe inside
        // (e.g. "doesn't") would leak `in_single` state into every
        // later attribute on the page, breaking `={…}` brace-attr
        // quoting for the rest of the file.
        if source[i..].starts_with("<!--") {
            let body_start = i + 4;
            let close_rel = source[body_start..].find("-->");
            let end = match close_rel {
                Some(rel) => body_start + rel + 3,
                None => bytes.len(),
            };
            out.push_str(&source[i..end]);
            i = end;
            continue;
        }
        out.push('<');
        i += 1;
        let mut in_single = false;
        let mut in_double = false;
        while i < bytes.len() {
            let ch = bytes[i];
            if ch == b'"' && !in_single {
                in_double = !in_double;
                out.push('"');
                i += 1;
                continue;
            }
            if ch == b'\'' && !in_double {
                in_single = !in_single;
                out.push('\'');
                i += 1;
                continue;
            }
            if !in_single && !in_double && ch == b'>' {
                out.push('>');
                i += 1;
                break;
            }
            if !in_single
                && !in_double
                && ch == b'='
                && i + 1 < bytes.len()
                && bytes[i + 1] == b'{'
                && let Some(end) = find_close(source, i + 1) {
                    let expr = &source[i + 1..=end];
                    out.push('=');
                    out.push('"');
                    for c in expr.chars() {
                        if c == '"' {
                            out.push_str("&quot;");
                        } else {
                            out.push(c);
                        }
                    }
                    out.push('"');
                    i = end + 1;
                    continue;
                }
            // Not the start of a brace attr — copy verbatim.
            // Use char boundary safe append.
            let ch_start = i;
            // Find next char boundary
            let mut j = ch_start + 1;
            while j < bytes.len() && (bytes[j] & 0xC0) == 0x80 {
                j += 1;
            }
            out.push_str(&source[ch_start..j]);
            i = j;
        }
    }
    out
}

fn find_close(s: &str, open_brace_at: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut str_quote: Option<u8> = None;
    let mut i = open_brace_at;
    while i < bytes.len() {
        let c = bytes[i];
        if let Some(q) = str_quote {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == q {
                str_quote = None;
            }
            i += 1;
            continue;
        }
        if c == b'"' || c == b'\'' || c == b'`' {
            str_quote = Some(c);
            i += 1;
            continue;
        }
        if c == b'{' {
            depth += 1;
        } else if c == b'}' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn expand_self_close(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            // Append a single UTF-8 char.
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j] & 0xC0) == 0x80 {
                j += 1;
            }
            out.push_str(&source[i..j]);
            i = j;
            continue;
        }
        // Skip </…> close tags and <!…> directives whole.
        if i + 1 < bytes.len() && (bytes[i + 1] == b'/' || bytes[i + 1] == b'!') {
            if let Some(end_rel) = source[i..].find('>') {
                out.push_str(&source[i..=i + end_rel]);
                i += end_rel + 1;
                continue;
            }
            out.push_str(&source[i..]);
            break;
        }
        // Match a tag name `[A-Za-z][A-Za-z0-9-]*` after `<`.
        let tag_start = i + 1;
        let mut tag_end = tag_start;
        if tag_end >= bytes.len()
            || !(bytes[tag_end].is_ascii_alphabetic())
        {
            // Not actually a tag; copy `<` and continue.
            out.push('<');
            i += 1;
            continue;
        }
        tag_end += 1;
        while tag_end < bytes.len()
            && (bytes[tag_end].is_ascii_alphanumeric() || bytes[tag_end] == b'-')
        {
            tag_end += 1;
        }
        let tag = source[tag_start..tag_end].to_ascii_lowercase();
        // Find matching `>`.
        let mut j = tag_end;
        let mut in_single = false;
        let mut in_double = false;
        while j < bytes.len() {
            let ch = bytes[j];
            if ch == b'"' && !in_single {
                in_double = !in_double;
            } else if ch == b'\'' && !in_double {
                in_single = !in_single;
            } else if !in_single && !in_double && ch == b'>' {
                break;
            }
            j += 1;
        }
        if j == bytes.len() {
            out.push_str(&source[i..]);
            break;
        }
        let head = &source[i..j];
        // Self-close requires an actual `/` before `>`. Trailing whitespace
        // alone (multi-line attr layout `<button\n  …\n>`) MUST NOT be
        // treated as self-close — that would flatten every multi-line
        // element's children into siblings.
        let trimmed_ws = head.trim_end_matches(|c: char| c.is_whitespace());
        let is_self_close = trimmed_ws.ends_with('/');
        if is_self_close && !VOID_TAGS.contains(&tag.as_str()) {
            // Strip the slash, then any whitespace before it (`<x  />` → `<x>`).
            let trimmed_head = trimmed_ws
                .trim_end_matches('/')
                .trim_end_matches(|c: char| c.is_whitespace());
            out.push_str(trimmed_head);
            out.push('>');
            out.push_str("</");
            out.push_str(&tag);
            out.push('>');
        } else {
            out.push_str(&source[i..=j]);
        }
        i = j + 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_unquoted_brace_attr() {
        let out = quote_brace_attrs(r#"<x disabled={a || b} />"#);
        assert!(out.contains(r#"disabled="{a || b}""#));
    }

    #[test]
    fn escapes_double_quote_inside_expr() {
        let out = quote_brace_attrs(r#"<x v={a("z")}>"#);
        assert!(out.contains(r#"v="{a(&quot;z&quot;)}""#));
    }

    #[test]
    fn expands_non_void_selfclose() {
        let out = expand_self_close("<custom-tag attr=\"v\" />");
        assert_eq!(out, "<custom-tag attr=\"v\"></custom-tag>");
    }

    #[test]
    fn leaves_void_selfclose() {
        let out = expand_self_close("<br />");
        assert_eq!(out, "<br />");
    }

    #[test]
    fn multiline_open_tag_is_not_self_close() {
        // Multi-line attribute layout — trailing newline before `>` must
        // NOT be treated as a self-close marker. Regression for sui-button
        // emitting children as siblings instead of nesting them inside.
        let src = "<button\n  type=\"button\"\n  disabled\n>\n  <span>x</span>\n</button>";
        let out = expand_self_close(src);
        assert_eq!(out, src, "multi-line open tag should pass through unchanged");
    }

    #[test]
    fn slash_with_leading_whitespace_is_self_close() {
        // `<x  />` (whitespace before `/`) is still XHTML-style self-close.
        let out = expand_self_close("<x  />");
        assert_eq!(out, "<x></x>");
    }

    #[test]
    fn full_pipeline() {
        let src = r#"<data name="agents" source="ship.agents.summaries()" live />
<button on:click={() => x()}>hi</button>"#;
        let out = preprocess(src);
        assert!(out.contains("</data>"));
        assert!(out.contains(r#"on:click="{() =&gt; x()}""#) || out.contains(r#"on:click="{() => x()}""#));
    }

    #[test]
    fn html_comment_with_apostrophe_does_not_leak_quote_state() {
        // Regression: an apostrophe inside an HTML comment (e.g.
        // "doesn't") used to toggle the preprocessor's `in_single`
        // tracker, which then leaked into every subsequent attribute
        // — every brace-attr after the comment failed to get quoted
        // and the HTML parser later rejected the unquoted `={` shape.
        // Real repro: views/eval/page.sui's header comment.
        let src = r#"<!--
  Note: the renderable subset doesn't admit `.items({…})` here.
-->
<article attr:data-passed={c ? 'true' : 'false'}></article>"#;
        let out = preprocess(src);
        assert!(
            out.contains(r#"attr:data-passed="{c ? 'true' : 'false'}""#),
            "brace-attr after a comment containing an apostrophe must \
             still get quoted, got:\n{out}",
        );
    }

    #[test]
    fn html_comment_with_nested_tag_does_not_leak() {
        // The fake `<data>` inside this comment used to be read as a
        // tag boundary. The first `>` (inside `<data>`) ended the
        // inner-while's "tag mode" early; subsequent text was
        // misclassified. With the comment-aware skip, the body is
        // copied verbatim and the real `<button>` is parsed correctly.
        let src = r#"<!-- see <data> usage --><button on:click={() => f()}>x</button>"#;
        let out = preprocess(src);
        assert!(out.contains("</data>") || out.contains("<data>"));
        assert!(
            out.contains(r#"on:click="{() => f()}""#)
                || out.contains(r#"on:click="{() =&gt; f()}""#),
            "on:click brace-attr after comment must get quoted, got:\n{out}",
        );
    }

    #[test]
    fn lower_sx_handles_multibyte_chars_without_panicking() {
        // Regression: `lower_sx` byte-walked with `i += 1` and sliced
        // `source[i..]`, panicking mid-codepoint on any multi-byte char — the
        // crash the async golden fixtures hit (an em-dash / ellipsis in copy).
        let src = "<p>Failed — retry…</p>\n<sui-card sx=\"p:4\">café ☕</sui-card>";
        let out = preprocess(src);
        assert!(out.contains("café ☕"), "multibyte content preserved: {out}");
        assert!(out.contains("class=\"sx-p-4\""), "sx still lowered: {out}");
    }

    #[test]
    fn lowers_column_to_div_with_classes_and_style() {
        let out = preprocess(r#"<column gap="4"><p>hi</p></column>"#);
        assert!(out.contains(r#"<div class="sui-col sui-g-4 sui-ai-stretch">"#), "col lowered: {out}");
        assert!(out.contains("</div>"), "close tag lowered: {out}");
        assert!(!out.contains("<column"), "no column tag remains: {out}");
        assert!(out.contains(".sui-col{display:flex;flex-direction:column"), "base css: {out}");
        assert!(out.contains(".sui-g-4{gap:var(--space-4)}"), "gap css: {out}");
    }

    #[test]
    fn layout_element_sx_merges_into_one_class() {
        // sx lowers first (→ class), then layout folds it into the div's classes.
        let out = preprocess(r#"<column gap="2" sx="p:4">x</column>"#);
        assert!(out.contains(r#"class="sui-col sui-g-2 sui-ai-stretch sx-p-4""#), "merged: {out}");
        assert_eq!(out.matches("class=").count(), 1, "exactly one class attr: {out}");
    }

    #[test]
    fn passes_through_id_and_merges_static_class() {
        let out = preprocess(r#"<row id="bar" class="mine">x</row>"#);
        assert!(out.contains(r#"id="bar""#), "id preserved: {out}");
        assert!(out.contains("sui-row") && out.contains("mine"), "class merged: {out}");
    }

    #[test]
    fn nested_layout_lowers_every_box() {
        let out = preprocess(r#"<column><row><center>x</center></row></column>"#);
        assert_eq!(out.matches("<div").count(), 3, "three divs: {out}");
        assert_eq!(out.matches("</div>").count(), 3, "three closes: {out}");
    }

    #[test]
    fn unterminated_comment_copies_to_eof() {
        // Defensive: a malformed file with no closing `-->` shouldn't
        // panic or read past end-of-file. Copy the rest verbatim and
        // let the downstream parser produce its own diagnostic.
        let src = "<!-- never closed";
        let out = preprocess(src);
        assert_eq!(out, src);
    }
}
