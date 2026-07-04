//! `sui-no-sx` — the `sx=` styling attribute is banned under the STRICTEST policy.
//!
//! `floor` and `stripped` bless `sx` (the token styling surface). The strictest
//! policy removes even that: "here are the primitives — compose them, use their
//! props, no styling escape at all." It's the maximal "make the wrong thing
//! unrepresentable" tier for app authors.
//!
//! Natural severity `Warn` so an unconfigured / floor / stripped package keeps
//! compiling silently (those policies override it to Off); `Policy::Strictest`
//! promotes it to a hard `Error`. Catches both static `sx="…"` and dynamic
//! `sx={…}`. Skips HTML comments; advances char-safely.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sui-no-sx";

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
        // An `sx=` attribute: `sx` preceded by whitespace, followed by an
        // (optionally spaced) `=`. Both `sx="…"` and `sx={…}` match.
        if i > 0
            && bytes[i - 1].is_ascii_whitespace()
            && source[i..].starts_with("sx")
            && next_is_eq(&bytes[i + 2..])
        {
            let (line, col) = offset_to_line_col(source, i);
            findings.push(Finding {
                file: path.to_path_buf(),
                line,
                col,
                rule: RULE_ID,
                severity: Severity::Warn,
                message:
                    "sx= is not allowed under the strictest policy — compose primitives and use \
                     their props; there is no styling escape at this tier."
                        .to_string(),
                suggest: Some(
                    "Text is already styled: write <h1>-<h6> and <p> plain — they take the type \
                     scale automatically. For structure use <sui-stack>/<sui-cluster>/<sui-grid>; \
                     for surface/tone use a primitive prop (padding, tone, size, variant). If \
                     nothing fits, that's a missing primitive to add, not a reason to keep sx."
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

/// After the `sx` name: optional whitespace, then `=`.
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
    fn flags_static_and_dynamic_sx() {
        assert_eq!(run(r#"<sui-card sx="p:4">x</sui-card>"#), 1);
        assert_eq!(run(r#"<sui-card sx={cond ? 'p:4' : 'p:2'}>x</sui-card>"#), 1);
        assert_eq!(run(r#"<sui-card sx = "p:4">x</sui-card>"#), 1);
    }

    #[test]
    fn does_not_flag_lookalikes_or_props() {
        assert_eq!(run(r#"<sui-card padding="lg" data-sx="x">y</sui-card>"#), 0);
        assert_eq!(run(r#"<sui-card sxy="x">y</sui-card>"#), 0);
    }

    #[test]
    fn is_warn_and_skips_comments() {
        let f = check(Path::new("t.sui"), r#"<sui-card sx="p:4">x</sui-card>"#);
        assert!(matches!(f[0].severity, Severity::Warn));
        assert_eq!(run(r#"<!-- <sui-card sx="p:9"> --> <sui-card>y</sui-card>"#), 0);
    }
}
