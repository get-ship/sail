//! `sail-attr-empty-presence` — an attribute binding whose expression returns an
//! empty string as a ternary *consequent* (`attr:x={cond ? '' : null}`) never
//! actually sets the attribute. The runtime's `bindAttr` auto-omits `''` (so
//! string attrs like `aria-label` fall back to other naming instead of
//! rendering an empty accessible name) — see `packages/sail/runtime/src/bind.ts`.
//!
//! The author almost always meant "present when `cond`". The presence idiom is
//! the boolean form `attr:x={cond}` — `bindAttr` maps `true` → `''` (attribute
//! present) and `false` → removed. For a real value, use a non-empty string.
//!
//! Flags only the consequent form (`? ''`). The alternate form
//! (`x ? y : ''`) and nullish fallback (`x ?? ''`) are the *legitimate*
//! auto-omit uses and are left alone. `?.` / `??` are skipped so optional
//! chaining and nullish coalescing never trip the rule.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-attr-empty-presence";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Match on the byte slice, not `source[i..]`: `i` walks byte-by-byte,
        // so a string slice here panics the moment it lands inside a multibyte
        // char (e.g. an em-dash in a comment). The needle is ASCII, so a byte
        // compare is equivalent and boundary-safe — same idiom the helpers use.
        if !bytes[i..].starts_with(b"attr:") {
            i += 1;
            continue;
        }
        let Some(brace) = binding_brace(source, i) else {
            i += 5;
            continue;
        };
        let Some(end) = expr_end(source, brace) else {
            i += 5;
            continue;
        };
        let expr = &source[brace + 1..end];
        if has_empty_consequent(expr) {
            let (line, col) = line_col(source, i);
            findings.push(Finding {
                file: path.to_path_buf(),
                line,
                col,
                rule: RULE_ID,
                severity: Severity::Error,
                message: "Attribute binding returns `''` from a `?` branch — an \
                     empty string is auto-omitted, so `bindAttr` removes the \
                     attribute and this never renders. For presence use the \
                     boolean idiom `attr:x={cond}` (true sets the attribute, \
                     false removes it); for a value use a non-empty string."
                    .to_string(),
                suggest: Some(
                    "Replace `{cond ? '' : null}` with `{cond}` — the runtime \
                     sets the attribute when the expression is `true` and \
                     removes it when `false`."
                        .to_string(),
                ),
            });
        }
        i = end + 1;
    }

    findings
}

/// From a byte offset pointing at `attr:`, return the index of the `{` that
/// opens the binding value, or `None` when this attr isn't a `{}` binding
/// (e.g. `attr:foo="static"`).
fn binding_brace(source: &str, attr_start: usize) -> Option<usize> {
    let b = source.as_bytes();
    let mut i = attr_start + 5; // past "attr:"
    while i < b.len()
        && (b[i].is_ascii_alphanumeric() || b[i] == b'-' || b[i] == b'_' || b[i] == b':')
    {
        i += 1;
    }
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= b.len() || b[i] != b'=' {
        return None;
    }
    i += 1;
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    if i < b.len() && b[i] == b'{' {
        Some(i)
    } else {
        None
    }
}

/// From the opening `{` (byte index), return the index of the matching `}`,
/// honouring nested braces and string literals.
fn expr_end(source: &str, brace: usize) -> Option<usize> {
    let b = source.as_bytes();
    let mut depth = 0i32;
    let mut i = brace;
    let mut in_str: Option<u8> = None;
    while i < b.len() {
        let c = b[i];
        if let Some(q) = in_str {
            if c == b'\\' && i + 1 < b.len() {
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

/// True when `expr` has a single-`?` ternary whose consequent is an empty
/// string literal (`'' ` or `""`). Skips `?.` and `??` so optional chaining and
/// nullish coalescing don't match, and ignores `?` inside string literals.
fn has_empty_consequent(expr: &str) -> bool {
    let b = expr.as_bytes();
    let mut i = 0;
    let mut in_str: Option<u8> = None;
    while i < b.len() {
        let c = b[i];
        if let Some(q) = in_str {
            if c == b'\\' && i + 1 < b.len() {
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
        if c == b'?' {
            let next = b.get(i + 1).copied();
            let prev = if i > 0 { b.get(i - 1).copied() } else { None };
            // `?.` (optional chaining) or `??` (nullish) — not a ternary.
            if next == Some(b'?') || next == Some(b'.') || prev == Some(b'?') {
                i += 1;
                continue;
            }
            let mut j = i + 1;
            while j < b.len() && b[j].is_ascii_whitespace() {
                j += 1;
            }
            if j + 1 < b.len()
                && ((b[j] == b'\'' && b[j + 1] == b'\'') || (b[j] == b'"' && b[j + 1] == b'"'))
            {
                return true;
            }
            i = j;
            continue;
        }
        i += 1;
    }
    false
}

fn line_col(source: &str, offset: usize) -> (usize, usize) {
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
        check(Path::new("test.sui"), src)
    }

    #[test]
    fn empty_consequent_warns() {
        let f = lint(r#"<div attr:data-active={active() ? '' : null}></div>"#);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
    }

    #[test]
    fn double_quote_empty_consequent_warns() {
        let f = lint(r#"<sail-button attr:disabled={busy() ? "" : null}></sail-button>"#);
        assert_eq!(f.len(), 1);
    }

    #[test]
    fn boolean_idiom_passes() {
        let f = lint(r#"<div attr:data-active={active()}></div>"#);
        assert_eq!(f.len(), 0, "{f:?}");
    }

    #[test]
    fn empty_alternate_is_legit_auto_omit() {
        // `x ? value : ''` — the empty is the fallback, the intended auto-omit.
        let f = lint(r#"<div attr:title={hasTitle() ? label() : ''}></div>"#);
        assert_eq!(f.len(), 0, "{f:?}");
    }

    #[test]
    fn nullish_fallback_passes() {
        let f = lint(r#"<div attr:title={label() ?? ''}></div>"#);
        assert_eq!(f.len(), 0, "{f:?}");
    }

    #[test]
    fn optional_chaining_is_not_a_ternary() {
        let f = lint(r#"<div attr:title={obj?.label}></div>"#);
        assert_eq!(f.len(), 0, "{f:?}");
    }

    #[test]
    fn question_inside_string_does_not_trip() {
        let f = lint(r#"<div attr:title={label === "a ? '' : b" ? label() : null}></div>"#);
        assert_eq!(f.len(), 0, "{f:?}");
    }

    #[test]
    fn multibyte_chars_do_not_panic() {
        // `i` walks byte-by-byte; a string slice at a non-char-boundary used to
        // panic. An em-dash (3 bytes) in a comment before a real binding must
        // both not panic and still let the binding be flagged.
        let f = lint(
            "<!-- progress — bar -->\n<div attr:data-active={active() ? '' : null}></div>",
        );
        assert_eq!(f.len(), 1, "{f:?}");
    }

    #[test]
    fn static_attr_is_ignored() {
        let f = lint(r#"<div attr:role="region"></div>"#);
        assert_eq!(f.len(), 0);
    }

    #[test]
    fn two_bindings_each_evaluated() {
        let f = lint(
            r#"<div attr:data-a={x() ? '' : null} attr:data-b={y()}></div>"#,
        );
        assert_eq!(f.len(), 1, "only the first is the misuse: {f:?}");
    }

    #[test]
    fn multibyte_chars_with_symbols_do_not_panic() {
        // Em-dashes in comments (and `°`/`§`, etc.) put non-char-boundary
        // bytes in the byte-walk; the scan must not str-slice into them.
        let f = lint(
            "<!-- a comment — with em-dashes — and °/§ -->\n\
             <div attr:data-active={active() ? '' : null}>x—y</div>",
        );
        assert_eq!(f.len(), 1, "still detects after multibyte content: {f:?}");
        assert_eq!(f[0].rule, RULE_ID);
    }
}
