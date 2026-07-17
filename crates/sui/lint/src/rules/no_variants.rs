//! `sui-no-variants` — the `<variants>` block is banned under the `strict` policy.
//!
//! `full` (primitive-authoring) blesses `<variants>` — the CVA-style prop→style
//! table a primitive uses to map its own props to `:host` styling. The `strict`
//! app tier removes it: apps compose finished primitives and drive them with
//! props; they do not author new variant surfaces. A `<variants>` block in app
//! code means the app is re-implementing a primitive — a missing primitive to
//! add upstream, not app styling.
//!
//! Natural severity `Warn`, so an unconfigured / `full` package compiles
//! silently (those override it to Off); `strict` promotes it to a hard `Error`.
//! Skips HTML comments; advances char-safely.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-no-variants";

const TAG: &str = "variants";

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
        // A `<variants>` / `<variants …>` opening tag — the name must end here so
        // `<variants-picker>` (a hypothetical component) is not caught.
        if bytes[i] == b'<' && source[i + 1..].starts_with(TAG) {
            let after = bytes.get(i + 1 + TAG.len()).copied();
            if matches!(after, Some(b) if b == b'>' || b == b'/' || b.is_ascii_whitespace()) {
                let (line, col) = offset_to_line_col(source, i);
                findings.push(Finding {
                    file: path.to_path_buf(),
                    line,
                    col,
                    rule: RULE_ID,
                    severity: Severity::Warn,
                    message:
                        "<variants> is not allowed under the strict policy — apps compose primitives \
                         and drive them with props; they do not author variant surfaces."
                            .to_string(),
                    suggest: Some(format!(
                        "Set the primitive's prop instead (tone / size / variant). If no prop fits, \
                         that's a missing primitive variant to add upstream — not one to declare in \
                         app code.{}",
                        crate::config::grant_hint(path, "variants")
                    )),
                });
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

    fn run(src: &str) -> usize {
        check(Path::new("t.sui"), src).len()
    }

    #[test]
    fn flags_the_variants_block() {
        assert_eq!(run("<variants>\n  tone=success: bg:tone-success-bg\n</variants>"), 1);
        assert_eq!(run(r#"<variants class="x">"#), 1);
    }

    #[test]
    fn does_not_flag_lookalikes_or_comments() {
        // a longer tag name that merely starts with "variants"
        assert_eq!(run("<variants-picker>x</variants-picker>"), 0);
        assert_eq!(run("<!-- <variants> --> <sui-card>y</sui-card>"), 0);
    }

    #[test]
    fn is_warn() {
        let f = check(Path::new("t.sui"), "<variants>tone=x: bg:surface-1</variants>");
        assert!(matches!(f[0].severity, Severity::Warn));
    }
}
