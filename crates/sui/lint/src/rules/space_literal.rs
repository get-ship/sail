//! `sui-space-literal` (POC) — raw `px` in a spacing property (`padding`,
//! `margin`, `gap` + longhands) inside a `.sui` `<style>` block should be a
//! design-system spacing token, `var(--space-N)`.
//!
//! This is the proof-of-concept extension of the design-system enforcement
//! beyond colour (`sui-css-literal`). The principle: an agent writes normal CSS
//! properties, but the *values* come from a closed, themeable vocabulary — so
//! output is consistent by construction and the diagnostic teaches the token.
//!
//! Scope (deliberately narrow for the POC):
//! - Properties: `padding`/`margin`/`gap` and their longhands + logical forms.
//! - Flags a bare `<number>px` value (e.g. `12px`). Suggests the matching
//!   `--space-N` when the number is on the scale (4/8/12/16/20/24/32/40 →
//!   1..8), otherwise nudges to the nearest token.
//! - Allowed: `0`, `auto`, `%`, `calc(...)`, `var(...)` — anything that isn't a
//!   raw px length. `1px`/`2px` hairlines on the *border* family are out of
//!   scope (border isn't a spacing property here).
//!
//! Severity is `Warn` (not `Error` like `sui-css-literal`): ~500 raw-px spacing
//! values exist today. Surface them as warnings, sweep, then promote to `Error`
//! once clean — the same playbook colour followed.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-space-literal";

const SPACING_PROPERTIES: &[&str] = &[
    "padding",
    "padding-top",
    "padding-right",
    "padding-bottom",
    "padding-left",
    "padding-block",
    "padding-block-start",
    "padding-block-end",
    "padding-inline",
    "padding-inline-start",
    "padding-inline-end",
    "margin",
    "margin-top",
    "margin-right",
    "margin-bottom",
    "margin-left",
    "margin-block",
    "margin-block-start",
    "margin-block-end",
    "margin-inline",
    "margin-inline-start",
    "margin-inline-end",
    "gap",
    "row-gap",
    "column-gap",
];

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for block in scan_style_blocks(source) {
        for decl in scan_declarations(block.content) {
            if !SPACING_PROPERTIES.contains(&decl.property) {
                continue;
            }
            let value_no_var = blank_var_references(decl.value);
            for lit in find_px_literals(&value_no_var) {
                let abs = block.content_offset + decl.value_offset + lit.offset;
                let (line, col) = byte_offset_to_line_col(source, abs);
                let suggest = match space_token_for(lit.px) {
                    Some(n) => format!(
                        "Replace `{}px` with `var(--space-{n})`.",
                        lit.px
                    ),
                    None => format!(
                        "`{}px` is off the spacing scale — use the nearest \
                         `var(--space-N)` (4/8/12/16/20/24/32/40 = 1..8) or a \
                         local `--token` derived from one.",
                        lit.px
                    ),
                };
                findings.push(Finding {
                    file: path.to_path_buf(),
                    line,
                    col,
                    rule: RULE_ID,
                    severity: Severity::Warn,
                    message: format!(
                        "raw `{}px` in `{}` — spacing must be a design-system \
                         token (`var(--space-N)`), not a literal length",
                        lit.px, decl.property
                    ),
                    suggest: Some(suggest),
                });
            }
        }
    }
    findings
}

fn space_token_for(px: u32) -> Option<u8> {
    match px {
        4 => Some(1),
        8 => Some(2),
        12 => Some(3),
        16 => Some(4),
        20 => Some(5),
        24 => Some(6),
        32 => Some(7),
        40 => Some(8),
        _ => None,
    }
}

struct StyleBlock<'a> {
    content_offset: usize,
    content: &'a str,
}

struct Decl<'a> {
    property: &'a str,
    value_offset: usize,
    value: &'a str,
}

struct PxLiteral {
    /// Offset of the number's first digit, into the value.
    offset: usize,
    px: u32,
}

/// Walk `<style>...</style>` blocks. (Mirror of the scanners in
/// `css_literal.rs` / `css_token.rs` — duplicated by the same convention.)
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

fn scan_declarations(content: &str) -> Vec<Decl<'_>> {
    let mut out = Vec::new();
    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len()
            && (bytes[i].is_ascii_whitespace() || matches!(bytes[i], b';' | b'{' | b'}'))
        {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            if let Some(end_rel) = find_subslice(&bytes[i + 2..], b"*/") {
                i += 2 + end_rel + 2;
                continue;
            }
            break;
        }
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
        let mut depth = 0i32;
        while v < bytes.len() {
            match bytes[v] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                b';' | b'}' if depth <= 0 => break,
                _ => {}
            }
            v += 1;
        }
        out.push(Decl {
            property: name,
            value_offset: value_start,
            value: content[value_start..v].trim_end(),
        });
        i = v;
    }
    out
}

/// Blank out `var(...)` expressions (incl. their `--space-N` fallbacks) with
/// spaces so the px scanner ignores tokenised values; offsets stay aligned.
fn blank_var_references(value: &str) -> String {
    let mut out = value.as_bytes().to_vec();
    let bytes = value.as_bytes();
    let mut i = 0;
    while i + 4 <= bytes.len() {
        if &bytes[i..i + 4] == b"var(" {
            let mut depth = 0i32;
            let mut j = i;
            while j < bytes.len() {
                match bytes[j] {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            j += 1;
                            break;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            for b in &mut out[i..j] {
                *b = b' ';
            }
            i = j;
        } else {
            i += 1;
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| value.to_string())
}

/// Find `<number>px` length literals. Skips `0` (and `0px`).
fn find_px_literals(value: &str) -> Vec<PxLiteral> {
    let mut out = Vec::new();
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            // A length only starts at a word boundary (not mid-identifier
            // like `space-1`), and not right after a digit.
            let prev = if i > 0 { bytes[i - 1] } else { b' ' };
            if is_ident_byte(prev) {
                // inside an identifier (e.g. `--space-12`); skip the run.
                while i < bytes.len() && is_ident_byte(bytes[i]) {
                    i += 1;
                }
                continue;
            }
            let num_start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            // optional fractional part — treat as non-scale (won't match token)
            if i < bytes.len() && bytes[i] == b'.' {
                i += 1;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
            }
            if i + 1 < bytes.len() && bytes[i] == b'p' && bytes[i + 1] == b'x' {
                let num = &value[num_start..i];
                let px: u32 = num.parse().unwrap_or(u32::MAX);
                if px != 0 {
                    out.push(PxLiteral {
                        offset: num_start,
                        px,
                    });
                }
                i += 2;
            }
            continue;
        }
        i += 1;
    }
    out
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
    for &b in &bytes[..cap] {
        if b == b'\n' {
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
    use std::path::PathBuf;

    fn run(src: &str) -> Vec<Finding> {
        check(&PathBuf::from("/fake/foo.sui"), src)
    }

    #[test]
    fn raw_px_padding_warns_with_token_suggestion() {
        let f = run("<style>:host { padding: 12px; }</style>");
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert!(f[0].suggest.as_ref().unwrap().contains("--space-3"), "{:?}", f[0].suggest);
    }

    #[test]
    fn token_value_passes() {
        let f = run("<style>:host { gap: var(--space-2); }</style>");
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn zero_passes() {
        assert!(run("<style>:host { margin: 0; }</style>").is_empty());
        assert!(run("<style>:host { padding: 0px; }</style>").is_empty());
    }

    #[test]
    fn auto_and_percent_pass() {
        assert!(run("<style>:host { margin: 0 auto; }</style>").is_empty());
        assert!(run("<style>:host { gap: 50%; }</style>").is_empty());
    }

    #[test]
    fn shorthand_flags_each_raw_px() {
        // `8px 12px` → two findings (space-2 + space-3).
        let f = run("<style>:host { padding: 8px 12px; }</style>");
        assert_eq!(f.len(), 2, "{f:?}");
    }

    #[test]
    fn off_scale_px_warns_without_exact_token() {
        let f = run("<style>:host { gap: 13px; }</style>");
        assert_eq!(f.len(), 1);
        assert!(f[0].suggest.as_ref().unwrap().contains("off the spacing scale"));
    }

    #[test]
    fn non_spacing_property_ignored() {
        // border/width/height px is out of scope for this rule.
        assert!(run("<style>:host { border: 1px solid var(--border); }</style>").is_empty());
        assert!(run("<style>:host { width: 220px; }</style>").is_empty());
    }

    #[test]
    fn var_with_space_token_in_name_not_double_counted() {
        // `--space-12` mentions 12 but it's inside a var() — blanked out.
        assert!(run("<style>:host { gap: var(--space-4); }</style>").is_empty());
    }

    #[test]
    fn logical_props_flagged() {
        let f = run("<style>:host { padding-inline: 16px; }</style>");
        assert_eq!(f.len(), 1);
        assert!(f[0].suggest.as_ref().unwrap().contains("--space-4"));
    }
}
