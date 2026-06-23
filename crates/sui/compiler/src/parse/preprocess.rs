//! Source preprocessor — exact mirror of
//! `packages/sui/dom/src/preprocess.ts`.
//!
//! Two transforms run before the HTML parser sees the source:
//!  1. Quote unquoted brace attributes — `disabled={x || y}` becomes
//!     `disabled="{x || y}"` with `"` inside the expression escaped
//!     to `&quot;`.
//!  2. Expand non-void self-closing tags — `<data … />` becomes
//!     `<data …></data>`.

const VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
    "source", "track", "wbr",
];

pub fn preprocess(source: &str) -> String {
    expand_self_close(&quote_brace_attrs(source))
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
    fn unterminated_comment_copies_to_eof() {
        // Defensive: a malformed file with no closing `-->` shouldn't
        // panic or read past end-of-file. Copy the rest verbatim and
        // let the downstream parser produce its own diagnostic.
        let src = "<!-- never closed";
        let out = preprocess(src);
        assert_eq!(out, src);
    }
}
