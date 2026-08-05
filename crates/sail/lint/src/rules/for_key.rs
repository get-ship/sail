//! `sail-for-key` — `<for {items} as="item">` blocks should declare a `key=`
//! expression so the reconciler can preserve DOM identity across
//! reorder / insert / delete.
//!
//! Without `key=`, the reconciler falls back to positional matching,
//! which silently corrupts list state when items shift. The bug class
//! is common because the failure is invisible until the user reorders
//! a list with focus / selection / animation state on a row.
//!
//! Acts on the open tag of an element-form `<for …>`. Doesn't reach into
//! the `of=` expression — we only check that `key=` is present somewhere
//! in the open tag. HTML comments (`<!-- ... -->`) are skipped, so a
//! `<for` mentioned in a primitive's leading docblock is not a false hit.
//! This matters at `Error` severity: a docblock mention must not hard-fail
//! a compile.
//!
//! Suggest text points at the runtime warning that fires when keys
//! collide — same diagnostic source. See
//! `packages/sail/runtime/src/for.ts` for the runtime side.
//!
//! **Severity: Error.** Every real `<for>` across the components/views
//! tree carries a `key=`. Error hard-fails any regression.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-for-key";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let boundaries: Vec<usize> = source
        .char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(source.len()))
        .collect();
    let mut bi = 0;

    while bi < boundaries.len() {
        let i = boundaries[bi];
        if i >= source.len() {
            break;
        }
        if source[i..].starts_with("<!--") {
            let after = i + 4;
            match source[after..].find("-->") {
                Some(rel) => {
                    let comment_end = after + rel + 3;
                    while bi < boundaries.len() && boundaries[bi] < comment_end {
                        bi += 1;
                    }
                    continue;
                }
                None => break,
            }
        }
        // Skip `{…}` expression groups so a string like `{'<for>'}` is not
        // treated as a call site (the marker is string content, not markup).
        if source.as_bytes().get(i) == Some(&b'{') {
            if let Some(end) = find_brace_group_end(source, i) {
                while bi < boundaries.len() && boundaries[bi] <= end {
                    bi += 1;
                }
                continue;
            }
        }
        if !is_for_open_tag(source, i) {
            bi += 1;
            continue;
        }
        let header_start = i;
        let header_end = match find_open_tag_end(source, i) {
            Some(end) => end,
            None => {
                bi += 1;
                continue;
            }
        };

        let header = &source[header_start..=header_end];
        if !has_key_attribute(header) {
            let (line, col) = byte_offset_to_line_col(source, header_start);
            findings.push(Finding {
                file: path.to_path_buf(),
                line,
                col,
                rule: RULE_ID,
                severity: Severity::Error,
                message: "<for> block has no `key=` — the reconciler will \
                     fall back to positional matching and DOM identity \
                     (focus, selection, in-place edits) will be lost on \
                     reorder. Add `key={item.id}` (or whatever uniquely \
                     identifies each row)."
                    .to_string(),
                suggest: Some(
                    "Add `key={EXPR}` to the <for> open tag, where \
                     EXPR returns a stable unique value per item — e.g. \
                     `<for {items} as=\"item\" key={item.id}>`."
                        .to_string(),
                ),
            });
        }

        while bi < boundaries.len() && boundaries[bi] <= header_end {
            bi += 1;
        }
    }

    findings
}

/// Scan from `start` (`{`) to the matching `}`. Honours nested braces and strings.
fn find_brace_group_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    if start >= bytes.len() || bytes[start] != b'{' {
        return None;
    }
    let mut depth = 0i32;
    let mut i = start;
    let mut in_str: Option<u8> = None;
    while i < bytes.len() {
        let c = bytes[i];
        if let Some(q) = in_str {
            if c == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => in_str = Some(c),
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn is_for_open_tag(source: &str, start: usize) -> bool {
    if !source[start..].starts_with("<for") {
        return false;
    }
    let after = start + 4;
    if after >= source.len() {
        return false;
    }
    let next = source.as_bytes()[after];
    next.is_ascii_whitespace() || next == b'>' || next == b'/'
}

fn find_open_tag_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut i = start;
    let mut in_str: Option<u8> = None;
    let mut brace_depth = 0i32;

    while i < bytes.len() {
        let c = bytes[i];
        if let Some(q) = in_str {
            if c == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => in_str = Some(c),
            b'{' => brace_depth += 1,
            b'}' => {
                if brace_depth > 0 {
                    brace_depth -= 1;
                }
            }
            b'>' if brace_depth == 0 => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

fn has_key_attribute(header: &str) -> bool {
    let bytes = header.as_bytes();
    let mut i = 0;
    let mut in_str: Option<u8> = None;
    let mut brace_depth = 0i32;
    while i + 4 < bytes.len() {
        let c = bytes[i];
        if let Some(q) = in_str {
            if c == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        if c == b'"' || c == b'\'' || c == b'`' {
            in_str = Some(c);
            i += 1;
            continue;
        }
        if c == b'{' {
            brace_depth += 1;
            i += 1;
            continue;
        }
        if c == b'}' {
            if brace_depth > 0 {
                brace_depth -= 1;
            }
            i += 1;
            continue;
        }
        if brace_depth == 0 && &bytes[i..i + 4] == b"key=" {
            if i == 0 || bytes[i - 1].is_ascii_whitespace() {
                return true;
            }
        }
        i += 1;
    }
    false
}

fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lint(src: &str) -> Vec<Finding> {
        check(Path::new("test.sail"), src)
    }

    #[test]
    fn for_with_key_attribute_passes() {
        let src = r#"<ul><for {items} as="item" key={item.id}><li>{item.name}</li></for></ul>"#;
        assert_eq!(lint(src).len(), 0);
    }

    #[test]
    fn for_without_key_warns() {
        let src = r#"<ul><for {items} as="item"><li>{item.name}</li></for></ul>"#;
        let findings = lint(src);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("`key=`"));
        assert!(findings[0].suggest.is_some());
    }

    #[test]
    fn iter_expression_using_key_identifier_does_not_count() {
        let src = r#"<for {items.sortBy(i => i.key)} as="item"><li>x</li></for>"#;
        assert_eq!(lint(src).len(), 1);
    }

    #[test]
    fn two_for_blocks_each_evaluated_separately() {
        let src = r#"
            <for {as} as="a" key={a.id}><li>{a.name}</li></for>
            <for {bs} as="b"><li>{b.name}</li></for>
        "#;
        assert_eq!(lint(src).len(), 1);
    }

    #[test]
    fn iter_with_gt_in_string_does_not_break_scan() {
        let src = r#"<for {items.filter(i => i.s === ">")} as="x" key={x.id}><li>x</li></for>"#;
        assert_eq!(lint(src).len(), 0);
    }

    #[test]
    fn malformed_for_block_does_not_panic() {
        let src = r#"<for {items"#;
        let _ = lint(src);
    }

    #[test]
    fn for_in_html_comment_is_not_a_violation() {
        let src = "<!--\n  a single `<for>` over tabs()\n-->\n\
                   <ul><for {tabs()} as=\"t\" key={t.id}><li>{t.label}</li></for></ul>";
        assert_eq!(lint(src).len(), 0);
    }

    #[test]
    fn real_for_after_comment_still_flagged() {
        let src = "<!-- <for {xs} as=\"x\"> -->\n\
                   <for {ys} as=\"y\"><li>{y}</li></for>";
        assert_eq!(lint(src).len(), 1);
    }

    #[test]
    fn form_tag_is_not_mistaken_for_for() {
        let src = r#"<form on:submit={go}><button>ok</button></form>"#;
        assert_eq!(lint(src).len(), 0);
    }

    #[test]
    fn real_keyless_for_after_a_string_expression_still_flagged() {
        let src = r#"<p>{'<for>'}</p><for {ys} as="y"><li>{y}</li></for>"#;
        assert_eq!(lint(src).len(), 1);
    }

    #[test]
    fn severity_is_error() {
        let src = r#"<for {items} as="item"><li>x</li></for>"#;
        let findings = lint(src);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Error);
    }
}
