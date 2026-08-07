//! `sail-no-inline-style` — raw CSS `style="…"` banned; product host is tag `style(…)`.

use std::path::Path;
use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-no-inline-style";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let bytes = source.as_bytes();
    let mut findings = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if source[i..].starts_with("<!--") {
            match source[i..].find("-->") {
                Some(end) => { i += end + 3; continue; }
                None => break,
            }
        }
        if i > 0
            && bytes[i - 1].is_ascii_whitespace()
            && source[i..].starts_with("style")
            && next_is_quoted_eq(&bytes[i + 5..])
        {
            let (line, col) = offset_to_line_col(source, i);
            findings.push(Finding {
                file: path.to_path_buf(), line, col, rule: RULE_ID,
                severity: Severity::Warn,
                message: "inline style=\"…\" is raw CSS and bypasses the token surface — it is not allowed in any `.sail`. Use Style tag call `style({ box: …, paint: … })` or a primitive prop; raw CSS values never compile.".into(),
                suggest: Some("Use Style tag call `style({ box: …, paint: … })` on the open tag, or a primitive prop. There is no raw-CSS escape.".into()),
            });
        }
        i += 1;
        while i < bytes.len() && (bytes[i] & 0xC0) == 0x80 { i += 1; }
    }
    findings
}

fn next_is_quoted_eq(rest: &[u8]) -> bool {
    let mut k = 0;
    while k < rest.len() && rest[k].is_ascii_whitespace() { k += 1; }
    if k >= rest.len() || rest[k] != b'=' { return false; }
    k += 1;
    while k < rest.len() && rest[k].is_ascii_whitespace() { k += 1; }
    k < rest.len() && (rest[k] == b'"' || rest[k] == b'\'')
}

fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize; let mut col = 1usize;
    for (idx, b) in source.bytes().enumerate() {
        if idx == offset { break; }
        if b == b'\n' { line += 1; col = 1; } else { col += 1; }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(src: &str) -> usize { check(Path::new("t.sail"), src).len() }
    #[test]
    fn flags_raw_css_style_string_only() {
        assert_eq!(run(r#"<p style="color: red">x</p>"#), 1);
        assert_eq!(run(r#"<p style = "x">y</p>"#), 1);
        assert_eq!(run(r#"<p style={style.fg(primary)}>x</p>"#), 0);
        assert_eq!(run(r#"<p style={dyn}>x</p>"#), 0);
    }
    #[test]
    fn does_not_flag_style_element_or_lookalikes() {
        assert_eq!(run(r#"<style>:host{color:red}</style>"#), 0);
        assert_eq!(run(r#"<p data-style="x" styled="y">z</p>"#), 0);
    }
    #[test]
    fn is_warn_and_skips_comments() {
        let f = check(Path::new("t.sail"), r#"<p style="x">y</p>"#);
        assert!(matches!(f[0].severity, Severity::Warn));
        assert_eq!(run(r#"<!-- <p style="x"> --> <sail-card>y</sail-card>"#), 0);
    }
}
