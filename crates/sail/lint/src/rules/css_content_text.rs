//! `sail-css-content-text` — meaningful text in CSS `content:` is illegal.
//!
//! Why this rule exists:
//! - Label / status text rendered through `::before`/`::after` `content`
//!   (`content: 'agent'`, `content: 'ok'`, `content: 'review'`) is
//!   invisible to the accessibility tree (screen readers do not reliably
//!   announce generated content), unselectable, and untranslatable.
//! - Text is *content*. The framework's first rule is "templates declare
//!   structure" — content belongs in the DOM (a slot, a `{expr}` bound to
//!   a controller `derived:` value), never baked into a stylesheet.
//! - A prop-driven label (type → "AGENT", state → "review") is exactly the
//!   `derived:` case: `derived: { label: ({ type }) => … }` then
//!   `<span>{label()}</span>`. The CSS-content shortcut bypasses that.
//!
//! Flags: a `content:` declaration whose quoted string holds a run of two
//! or more ASCII letters (a word) — `'agent'`, `'ok'`, `'no steps'`.
//!
//! Allows (decorative, tokenless): empty `''`, single glyphs / symbols
//! (`'●'`, `'→'`, `'·'`, `'✓'`), single letters, numeric/escape codepoints
//! (`'\2014'`, icon-font PUA), `attr(...)`, `counter(...)`, `none`, `var(...)`.
//!
//! Severity is `Error`: `sail-compiler` promotes error-severity findings to
//! a build failure, the same path `sail-css-literal` uses.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-css-content-text";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for block in scan_style_blocks(source) {
        for decl in scan_content_declarations(block.content) {
            if let Some(word) = first_word_in_quotes(decl.value) {
                let abs_offset = block.content_offset + decl.value_offset;
                let (line, col) = byte_offset_to_line_col(source, abs_offset);
                findings.push(Finding {
                    file: path.to_path_buf(),
                    line,
                    col,
                    rule: RULE_ID,
                    severity: Severity::Error,
                    message: format!(
                        "text `{word}` in CSS `content:` — label text must live in the DOM, \
                         not a pseudo-element (inaccessible to screen readers, unselectable, \
                         untranslatable). Reserve `content` for decorative symbols."
                    ),
                    suggest: Some(
                        "Render the text as real markup driven by a controller value: declare \
                         `const label = derived(() => …)` in the `<controller>` block, \
                         `return { label }`, and write `{label()}` where the text belongs in the \
                         template — or accept it via a <slot>. Keep `content` only for empty \
                         strings or single decorative glyphs (and mark purely ornamental hosts \
                         aria-hidden)."
                            .to_string(),
                    ),
                });
            }
        }
    }
    findings
}

struct StyleBlock<'a> {
    content_offset: usize,
    content: &'a str,
}

struct ContentDecl<'a> {
    /// Byte offset of the value into the style block's content.
    value_offset: usize,
    value: &'a str,
}

/// Walk `<style>...</style>` blocks. Mirror of the helper in `css_literal.rs`
/// — kept duplicated rather than introducing a shared AST module while the
/// rules' position-mapping needs are still in flux.
fn scan_style_blocks(source: &str) -> Vec<StyleBlock<'_>> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    while let Some(rel) = find_subslice(&bytes[i..], b"<style") {
        let tag_start = i + rel;
        let after_tag = tag_start + b"<style".len();
        if !matches!(
            bytes.get(after_tag).copied(),
            Some(b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/')
        ) {
            i = after_tag;
            continue;
        }
        let Some(open_end_rel) = find_subslice(&bytes[after_tag..], b">") else {
            break;
        };
        if open_end_rel > 0 && bytes.get(after_tag + open_end_rel - 1) == Some(&b'/') {
            i = after_tag + open_end_rel + 1;
            continue;
        }
        let content_start = after_tag + open_end_rel + 1;
        let Some(close_rel) = find_subslice(&bytes[content_start..], b"</style>") else {
            break;
        };
        let content_end = content_start + close_rel;
        out.push(StyleBlock {
            content_offset: content_start,
            content: &source[content_start..content_end],
        });
        i = content_end + b"</style>".len();
    }
    out
}

/// Find every `content:` declaration. Declarations only exist *inside* a
/// `{ }` rule block, so the scanner tracks brace depth: at depth 0 it is in
/// selector context (`.kind::before`, `a:hover`) and skips to the next `{`,
/// which is what stops a pseudo-element's `:` from being mistaken for a
/// declaration colon. Inside a block it reads `ident: value` pairs, skipping
/// comments and string contents (so a `;`/`}` inside a quoted value can't
/// terminate the value early). A "value" that runs into a `{` was actually a
/// nested selector (`@media { .x::before { … } }`) — abandon it and descend.
fn scan_content_declarations(content: &str) -> Vec<ContentDecl<'_>> {
    let mut out = Vec::new();
    let bytes = content.as_bytes();
    let mut i = 0;
    let mut depth = 0i32;
    while i < bytes.len() {
        // CSS comments, at any depth.
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
            if let Some(end_rel) = find_subslice(&bytes[i + 2..], b"*/") {
                i += 2 + end_rel + 2;
                continue;
            }
            break;
        }
        if depth == 0 {
            // Selector / at-rule prelude — only an opening brace matters.
            if bytes[i] == b'{' {
                depth += 1;
            }
            i += 1;
            continue;
        }
        match bytes[i] {
            b'}' => {
                depth -= 1;
                i += 1;
                continue;
            }
            b'{' => {
                depth += 1;
                i += 1;
                continue;
            }
            b';' | b' ' | b'\t' | b'\n' | b'\r' => {
                i += 1;
                continue;
            }
            _ => {}
        }
        // Read an identifier (potential property name).
        let name_start = i;
        while i < bytes.len() && is_ident_byte(bytes[i]) {
            i += 1;
        }
        if i == name_start {
            i += 1;
            continue;
        }
        let name = &content[name_start..i];
        let mut k = i;
        while k < bytes.len() && bytes[k].is_ascii_whitespace() {
            k += 1;
        }
        if k >= bytes.len() || bytes[k] != b':' {
            // Not a declaration (nested selector, at-rule). Skip to a boundary.
            while i < bytes.len() && !matches!(bytes[i], b';' | b'{' | b'}') {
                i += 1;
            }
            continue;
        }
        let mut v = k + 1;
        while v < bytes.len() && bytes[v].is_ascii_whitespace() {
            v += 1;
        }
        let value_start = v;
        let mut pdepth = 0i32;
        let mut quote: Option<u8> = None;
        let mut hit_brace = false;
        while v < bytes.len() {
            let b = bytes[v];
            match quote {
                Some(q) => {
                    if b == b'\\' {
                        v += 1;
                    } else if b == q {
                        quote = None;
                    }
                }
                None => match b {
                    b'\'' | b'"' => quote = Some(b),
                    b'(' => pdepth += 1,
                    b')' => pdepth -= 1,
                    b'{' if pdepth <= 0 => {
                        hit_brace = true;
                        break;
                    }
                    b';' | b'}' if pdepth <= 0 => break,
                    _ => {}
                },
            }
            v += 1;
        }
        if hit_brace {
            // `ident:` was a nested selector (e.g. `.x::before {`), not a decl.
            depth += 1;
            i = v + 1;
            continue;
        }
        let value = content[value_start..v].trim_end();
        if name == "content" {
            out.push(ContentDecl {
                value_offset: value_start,
                value,
            });
        }
        i = v;
    }
    out
}

/// Return the first quoted-string word (run of ≥2 ASCII letters) in a
/// `content` value, or None when the value is purely decorative.
fn first_word_in_quotes(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let q = bytes[i];
        if q == b'\'' || q == b'"' {
            i += 1;
            let mut run = 0usize;
            let mut word_start = 0usize;
            while i < bytes.len() && bytes[i] != q {
                if bytes[i] == b'\\' {
                    run = 0;
                    i += 2;
                    continue;
                }
                if bytes[i].is_ascii_alphabetic() {
                    if run == 0 {
                        word_start = i;
                    }
                    run += 1;
                    if run >= 2 {
                        // Extend to the full word for the message.
                        let mut end = i + 1;
                        while end < bytes.len()
                            && bytes[end] != q
                            && bytes[end].is_ascii_alphabetic()
                        {
                            end += 1;
                        }
                        return Some(value[word_start..end].to_string());
                    }
                } else {
                    run = 0;
                }
                i += 1;
            }
        }
        i += 1;
    }
    None
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    let last = haystack.len() - needle.len();
    (0..=last).find(|&i| &haystack[i..i + needle.len()] == needle)
}

fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let mut line = 1usize;
    let mut col = 1usize;
    let cap = offset.min(bytes.len());
    let mut i = 0;
    while i < cap {
        if bytes[i] == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
        i += 1;
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn run(src: &str) -> Vec<Finding> {
        check(&PathBuf::from("/fake/foo.sail"), src)
    }

    #[test]
    fn word_in_content_reports() {
        let f = run("<style>.kind::before { content: 'agent'; }</style>");
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("agent"), "{}", f[0].message);
    }

    #[test]
    fn two_letter_state_reports() {
        assert_eq!(
            run("<style>.chip::after { content: 'ok'; }</style>").len(),
            1
        );
    }

    #[test]
    fn multiword_reports_first() {
        let f = run("<style>.dag:empty::after { content: 'no steps'; }</style>");
        assert_eq!(f.len(), 1);
    }

    #[test]
    fn empty_string_passes() {
        assert!(run("<style>.dot::before { content: ''; }</style>").is_empty());
    }

    #[test]
    fn glyph_symbol_passes() {
        assert!(run("<style>.gly::before { content: '↘'; }</style>").is_empty());
        assert!(run("<style>.x::before { content: '●'; }</style>").is_empty());
        assert!(run("<style>.x::before { content: '→'; }</style>").is_empty());
    }

    #[test]
    fn single_letter_passes() {
        assert!(run("<style>.x::before { content: 'x'; }</style>").is_empty());
    }

    #[test]
    fn escape_codepoint_passes() {
        // em-dash escape + icon-font PUA — digits/escapes, no 2-letter run.
        assert!(run("<style>.x::before { content: '\\2014'; }</style>").is_empty());
    }

    #[test]
    fn attr_and_counter_pass() {
        assert!(run("<style>.x::before { content: attr(data-label); }</style>").is_empty());
        assert!(run("<style>.x::before { content: counter(step); }</style>").is_empty());
    }

    #[test]
    fn none_keyword_passes() {
        assert!(run("<style>.x::before { content: none; }</style>").is_empty());
    }

    #[test]
    fn non_content_property_ignored() {
        assert!(run("<style>.x { font-family: 'agent mono'; }</style>").is_empty());
    }

    #[test]
    fn position_points_at_value() {
        let f = run("<style>\n  .k::before { content: 'agent'; }\n</style>");
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].line, 2);
    }
}
