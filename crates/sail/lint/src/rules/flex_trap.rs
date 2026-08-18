//! `sail-flex-trap` — `size(full)` is a DTCG CSS trap.
//!
//! Language pointer only: use `size(grow)` (share leftover) or `size(max)`
//! (stretch the containing block). Suggest-only FixPlan; nothing auto-applies.

use std::path::Path;

use crate::report::{Finding, Severity};
use crate::rules::layout;

pub const RULE_ID: &str = "sail-flex-trap";

pub const MESSAGE: &str = "`size(full)` is a DTCG CSS trap. Use `size(grow)` or `size(max)`.";

pub const SUGGEST: &str = "size(grow) shares leftover; size(max) stretches the containing block.";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for offset in size_full_sites(source) {
        let (line, col) = layout::offset_to_line_col(source, offset);
        findings.push(Finding {
            file: path.to_path_buf(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message: MESSAGE.into(),
            suggest: Some(SUGGEST.into()),
        });
    }
    findings
}

fn size_full_sites(source: &str) -> Vec<usize> {
    let bytes = source.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if !source.is_char_boundary(i) {
            i += 1;
            continue;
        }
        if bytes[i] == b'<' && source[i..].starts_with("<!--") {
            if let Some(end) = source[i..].find("-->") {
                i += end + 3;
                continue;
            }
        }
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'/') {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
            if let Some(end) = source[i + 2..].find("*/") {
                i += 2 + end + 2;
                continue;
            }
        }
        if source[i..].starts_with("size(full)") && is_word_start(bytes, i) {
            out.push(i);
            i += "size(full)".len();
            continue;
        }
        i += utf8_step(source, i);
    }
    out
}

fn is_word_start(bytes: &[u8], i: usize) -> bool {
    if i == 0 {
        return true;
    }
    !bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_'
}

fn utf8_step(source: &str, i: usize) -> usize {
    source[i..]
        .chars()
        .next()
        .map(|c| c.len_utf8())
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn run(src: &str) -> Vec<Finding> {
        check(Path::new("t.sail"), src)
    }

    #[test]
    fn flags_size_full() {
        let f = run(r#"<row><column style({ size(full) })>x</column></row>"#);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert_eq!(f[0].message, MESSAGE);
        assert!(f[0].suggest.as_deref().unwrap().contains("size(grow)"));
        assert!(f[0].suggest.as_deref().unwrap().contains("size(max)"));
    }

    #[test]
    fn quoted_trap_in_comment_is_ignored() {
        let f = run(r#"
<!--
        <column style({ size(full) })>x</column>
-->
<row>
  <column style({ size(grow) })>grow</column>
</row>
"#);
        assert!(f.is_empty(), "{f:?}");
    }
}
