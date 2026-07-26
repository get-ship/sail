//! `sail-participation-style` — flex/grid **participation** is spelled with
//! layout attrs (`grow` / `shrink` / `w` / `min-w` / `max-w`), not Style setters.
//!
//! `style.grow(…)` and `style.shrink(…)` (including mid-chain `.grow(1)`) still
//! compile — they lower to the same atomic classes as the attrs — but product
//! code must use the attrs so there is one spelling for participation. Paint
//! (pad, color, type, chrome) and intrinsic size stay on Style; when a chain
//! mixed grow+paint, split: attrs for participation, `sx={…}` for paint only.
//!
//! Severity `Error` — the dual path is closed for product after the cutover.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-participation-style";

/// Style method names that are layout participation, not paint.
const SETTERS: &[(&str, &str)] = &[
    ("grow", "grow"),
    ("shrink", "shrink"),
];

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let bytes = source.as_bytes();
    let mut findings = Vec::new();
    let mut i = 0;
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
        // Mid-chain or leading: `.grow(` / `.shrink(` — the Style product API.
        if bytes[i] == b'.' {
            if let Some(&(method, attr)) = SETTERS.iter().find(|&&(m, _)| {
                source[i + 1..].starts_with(m)
                    && bytes.get(i + 1 + m.len()) == Some(&b'(')
            }) {
                let (line, col) = offset_to_line_col(source, i);
                findings.push(Finding {
                    file: path.to_path_buf(),
                    line,
                    col,
                    rule: RULE_ID,
                    severity: Severity::Error,
                    message: format!(
                        "style.{method}(…) is layout participation — use the `{attr}` attr \
                         (boolean `{attr}` = 1, `{attr}=\"0\"` / `{attr}=\"1\"` for an explicit value)"
                    ),
                    suggest: Some(format!(
                        "Move participation off Style: put `{attr}` / `{attr}=\"0\"|\"1\"` on the \
                         element (and `min-w=\"0\"` / `w=\"full\"` when those were only for flex \
                         fill). Keep `sx={{…}}` for paint and intrinsic size only."
                    )),
                });
                i += 1 + method.len();
                continue;
            }
        }
        i += 1;
        while i < bytes.len() && (bytes[i] & 0xC0) == 0x80 {
            i += 1;
        }
    }
    findings
}

fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    for (idx, b) in source.bytes().enumerate() {
        if idx == offset {
            break;
        }
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

    fn run(src: &str) -> Vec<Finding> {
        check(Path::new("t.sail"), src)
    }

    #[test]
    fn flags_style_grow_and_shrink() {
        let f = run(r#"<div sx={style.grow(1)}>x</div>"#);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].rule, RULE_ID);
        assert_eq!(f[0].severity, Severity::Error);
        assert!(f[0].message.contains("grow"));

        let f = run(r#"const x = style.shrink(0).m(0)"#);
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("shrink"));
    }

    #[test]
    fn flags_mid_chain_grow() {
        let f = run(r#"<div sx={style.pad(2).grow(1).bg(card)}>x</div>"#);
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("grow"));
    }

    #[test]
    fn allows_participation_attrs_and_paint_style() {
        assert!(run(r#"<div grow min-w="0" sx={style.truncate()}>x</div>"#).is_empty());
        assert!(run(r#"<row grow="0" sx={style.pad(2)}>x</row>"#).is_empty());
        // Intrinsic width (icon box) stays on Style — not grow/shrink.
        assert!(run(r#"<span sx={style.w(9).h(9)}>x</span>"#).is_empty());
    }

    #[test]
    fn skips_html_comments() {
        assert!(run(r#"<!-- style.grow(1) --> <div grow>x</div>"#).is_empty());
    }
}
