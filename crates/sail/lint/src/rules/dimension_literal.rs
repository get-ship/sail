//! `sail-dimension-literal` — raw scalar values in `border-radius`, `font-size`,
//! and `z-index` should be the matching design-system token. Completes the
//! token-enforced vocabulary alongside `sail-css-literal` (colour) and
//! `sail-space-literal` (spacing).
//!
//! Each dimension maps a raw value to a *named* token (the scales aren't 1..N):
//! - radius:    4px→--radius-sm, 6→md, 10→lg, 14→xl, 9999→full
//! - font-size: 11px→--text-xs, 12→sm, 13→base, 14→md, 16→lg, 20→xl,
//!              24→2xl, 28→3xl, 36→4xl
//! - z-index:   0→--z-base, 1→elevated, 10→dropdown, 20→sticky, 30→popover,
//!              40→dialog, 50→toast, 100→max
//!
//! Exact matches get a direct token suggestion; off-scale values are nudged to
//! the nearest token (the value itself being a smell). `0` radius and tokenised
//! `var(...)` values pass. Severity `Warn` (surface → sweep → promote, like the
//! others). Shadow tokenisation is deferred — `box-shadow` colour literals are
//! already caught by `sail-css-literal`.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-dimension-literal";

struct Dim {
    /// Properties whose value is a single scalar in this dimension.
    properties: &'static [&'static str],
    /// `true` → values carry a `px` unit; `false` → unitless (z-index).
    px: bool,
    /// (raw numeric value ×100 to keep integer keys, token name).
    scale: &'static [(u32, &'static str)],
    label: &'static str,
}

const DIMENSIONS: &[Dim] = &[
    Dim {
        properties: &[
            "border-radius",
            "border-top-left-radius",
            "border-top-right-radius",
            "border-bottom-left-radius",
            "border-bottom-right-radius",
            "border-start-start-radius",
            "border-start-end-radius",
            "border-end-start-radius",
            "border-end-end-radius",
        ],
        px: true,
        scale: &[
            (400, "--radius-sm"),
            (600, "--radius-md"),
            (1000, "--radius-lg"),
            (1400, "--radius-xl"),
            (999900, "--radius-full"),
        ],
        label: "radius",
    },
    Dim {
        properties: &["font-size"],
        px: true,
        scale: &[
            (1100, "--text-xs"),
            (1200, "--text-sm"),
            (1300, "--text-base"),
            (1400, "--text-md"),
            (1600, "--text-lg"),
            (2000, "--text-xl"),
            (2400, "--text-2xl"),
            (2800, "--text-3xl"),
            (3600, "--text-4xl"),
        ],
        label: "font-size",
    },
    Dim {
        properties: &["z-index"],
        px: false,
        scale: &[
            (0, "--z-base"),
            (100, "--z-elevated"),
            (1000, "--z-dropdown"),
            (2000, "--z-sticky"),
            (3000, "--z-popover"),
            (4000, "--z-dialog"),
            (5000, "--z-toast"),
            (10000, "--z-max"),
        ],
        label: "z-index",
    },
];

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for block in scan_style_blocks(source) {
        for decl in scan_declarations(block.content) {
            let Some(dim) = DIMENSIONS.iter().find(|d| d.properties.contains(&decl.property))
            else {
                continue;
            };
            // Skip tokenised / non-literal values.
            let v = decl.value.trim();
            if v.starts_with("var(") || v == "0" || v == "inherit" || v == "initial" || v == "unset"
            {
                continue;
            }
            let Some((centi, lit_off, lit_text)) = first_scalar(decl.value, dim.px) else {
                continue;
            };
            // z-index 0 / radius 0 are fine.
            if centi == 0 {
                continue;
            }
            let suggest = match dim.scale.iter().find(|(c, _)| *c == centi) {
                Some((_, tok)) => format!("Replace `{lit_text}` with `var({tok})`."),
                None => {
                    let nearest = nearest_token(centi, dim.scale);
                    format!(
                        "`{lit_text}` is off the {} scale — use the nearest \
                         token `var({nearest})` or reconsider the value.",
                        dim.label
                    )
                }
            };
            let abs = block.content_offset + decl.value_offset + lit_off;
            let (line, col) = byte_offset_to_line_col(source, abs);
            findings.push(Finding {
                file: path.to_path_buf(),
                line,
                col,
                rule: RULE_ID,
                severity: Severity::Warn,
                message: format!(
                    "raw `{lit_text}` in `{}` — {} must be a design-system token \
                     (`var(--…)`), not a literal",
                    decl.property, dim.label
                ),
                suggest: Some(suggest),
            });
        }
    }
    findings
}

fn nearest_token(centi: u32, scale: &[(u32, &'static str)]) -> &'static str {
    scale
        .iter()
        .min_by_key(|(c, _)| (*c as i64 - centi as i64).abs())
        .map(|(_, t)| *t)
        .unwrap_or("--…")
}

/// Parse the first scalar in the value: a number (× 100) + its source text.
/// When `px` is set, the number must be followed by `px`; otherwise it's a bare
/// integer (z-index).
fn first_scalar(value: &str, px: bool) -> Option<(u32, usize, &str)> {
    let b = value.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i].is_ascii_digit() {
            let prev = if i > 0 { b[i - 1] } else { b' ' };
            if is_ident_byte(prev) {
                while i < b.len() && is_ident_byte(b[i]) {
                    i += 1;
                }
                continue;
            }
            let start = i;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
            let mut frac = 0u32;
            if i < b.len() && b[i] == b'.' {
                i += 1;
                let fs = i;
                while i < b.len() && b[i].is_ascii_digit() {
                    i += 1;
                }
                // up to 2 frac digits → centi
                let f = &value[fs..i];
                frac = format!("{:0<2}", &f[..f.len().min(2)]).parse().unwrap_or(0);
            }
            let whole: u32 = value[start..].split(|c: char| !c.is_ascii_digit()).next()
                .and_then(|s| s.parse().ok()).unwrap_or(0);
            let centi = whole * 100 + frac;
            if px {
                if i + 1 < b.len() && b[i] == b'p' && b[i + 1] == b'x' {
                    return Some((centi, start, &value[start..i + 2]));
                }
                // a number without px in a px-dimension (e.g. unitless) — skip.
                continue;
            } else {
                return Some((centi, start, &value[start..i]));
            }
        }
        i += 1;
    }
    None
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
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-') {
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
    fn radius_exact_token() {
        let f = run("<style>.x { border-radius: 6px; }</style>");
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].suggest.as_ref().unwrap().contains("--radius-md"));
    }

    #[test]
    fn font_size_exact_token() {
        let f = run("<style>.x { font-size: 13px; }</style>");
        assert_eq!(f.len(), 1);
        assert!(f[0].suggest.as_ref().unwrap().contains("--text-base"));
    }

    #[test]
    fn font_size_xs_token() {
        let f = run("<style>.x { font-size: 11px; }</style>");
        assert_eq!(f.len(), 1);
        assert!(f[0].suggest.as_ref().unwrap().contains("--text-xs"), "{:?}", f[0].suggest);
    }

    #[test]
    fn zindex_exact_token() {
        let f = run("<style>.x { z-index: 30; }</style>");
        assert_eq!(f.len(), 1);
        assert!(f[0].suggest.as_ref().unwrap().contains("--z-popover"));
    }

    #[test]
    fn token_value_passes() {
        assert!(run("<style>.x { border-radius: var(--radius-md); }</style>").is_empty());
        assert!(run("<style>.x { z-index: var(--z-popover); }</style>").is_empty());
    }

    #[test]
    fn zero_radius_passes() {
        assert!(run("<style>.x { border-radius: 0; }</style>").is_empty());
    }

    #[test]
    fn off_scale_radius_nudges() {
        let f = run("<style>.x { border-radius: 7px; }</style>");
        assert_eq!(f.len(), 1);
        assert!(f[0].suggest.as_ref().unwrap().contains("off the radius scale"));
    }

    #[test]
    fn unrelated_property_ignored() {
        assert!(run("<style>.x { padding: 12px; width: 200px; }</style>").is_empty());
    }
}
