//! `sui-no-inline-style` — the raw `style=` attribute is banned at every policy.
//!
//! `sui-no-style-block` closes the `<style>` element and `sui-no-sx` closes the
//! `sx=` attribute at the strictest tier — but an agent denied both simply falls
//! through to inline `style="…"` and hand-writes raw CSS (with raw px, raw
//! colors, invented tokens) that NO css-token lint even scans. The strictest
//! eval caught exactly this. Inline `style=` is raw CSS and bypasses the whole
//! token surface, so it is banned at *every* policy (floor included) — the
//! floor's core opinion is "one way to do CSS," and `style=` is a second way.
//!
//! Natural severity `Warn` (so an unconfigured buffer keeps compiling and the
//! LSP shows the hint); every policy (`floor`/`stripped`/`strictest`) promotes
//! it to a hard `Error`. Catches both static `style="…"` and dynamic
//! `style={…}`. Does NOT match the `<style>` *element* (that's preceded by `<`,
//! not whitespace — `sui-no-style-block`'s job). Skips comments, char-safe.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sui-no-inline-style";

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
        // A `style=` ATTRIBUTE: `style` preceded by whitespace (so `<style>` the
        // element, preceded by `<`, is excluded), followed by an optionally
        // spaced `=`.
        if i > 0
            && bytes[i - 1].is_ascii_whitespace()
            && source[i..].starts_with("style")
            && next_is_eq(&bytes[i + 5..])
        {
            let (line, col) = offset_to_line_col(source, i);
            findings.push(Finding {
                file: path.to_path_buf(),
                line,
                col,
                rule: RULE_ID,
                severity: Severity::Warn,
                message:
                    "inline style= is raw CSS and bypasses the token surface — it is not allowed \
                     at any policy. Use sx= (floor/stripped) or a primitive prop; raw values \
                     never compile."
                        .to_string(),
                suggest: Some(
                    "Text is already styled — write <h1>-<h6>/<p> plain (they take the type \
                     scale). For everything else use sx= (floor/stripped) or a primitive prop. \
                     There is no raw-CSS escape at any tier — that's the point of the floor."
                        .to_string(),
                ),
            });
        }
        i += 1;
        while i < bytes.len() && (bytes[i] & 0xC0) == 0x80 {
            i += 1;
        }
    }
    findings
}

/// After the attribute name: optional whitespace, then `=`.
fn next_is_eq(rest: &[u8]) -> bool {
    let mut k = 0;
    while k < rest.len() && rest[k].is_ascii_whitespace() {
        k += 1;
    }
    k < rest.len() && rest[k] == b'='
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

    fn run(src: &str) -> usize {
        check(Path::new("t.sui"), src).len()
    }

    #[test]
    fn flags_inline_style_static_and_dynamic() {
        assert_eq!(run(r#"<p style="color: red">x</p>"#), 1);
        assert_eq!(run(r#"<p style={dyn}>x</p>"#), 1);
        assert_eq!(run(r#"<p style = "x">y</p>"#), 1);
    }

    #[test]
    fn does_not_flag_style_element_or_lookalikes() {
        // `<style>` element is preceded by `<`, not whitespace — not an attr.
        assert_eq!(run(r#"<style>:host{color:red}</style>"#), 0);
        assert_eq!(run(r#"<p data-style="x" styled="y">z</p>"#), 0);
    }

    #[test]
    fn is_warn_and_skips_comments() {
        let f = check(Path::new("t.sui"), r#"<p style="x">y</p>"#);
        assert!(matches!(f[0].severity, Severity::Warn));
        assert_eq!(run(r#"<!-- <p style="x"> --> <sui-card>y</sui-card>"#), 0);
    }
}
