//! `sail-no-sx` — Style product host `style=` banned under `strict` policy.
//! Rule id keeps historical `sail-no-sx` (capability key remains `"sx"`).

use std::path::Path;
use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-no-sx";

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
            && next_is_eq(&bytes[i + 5..])
        {
            let (line, col) = offset_to_line_col(source, i);
            findings.push(Finding {
                file: path.to_path_buf(), line, col, rule: RULE_ID,
                severity: Severity::Warn,
                message: "style= (Style product host) is not allowed under the strict policy — compose primitives and use their props; there is no styling escape at this tier.".into(),
                suggest: Some(format!(
                    "Text is already styled: write <h1>-<h6> and <p> plain. For structure use <row>/<column>/<grid>; for surface/tone use a primitive prop.{}",
                    crate::config::grant_hint(path, "sx")
                )),
            });
        }
        i += 1;
        while i < bytes.len() && (bytes[i] & 0xC0) == 0x80 { i += 1; }
    }
    findings
}

fn next_is_eq(rest: &[u8]) -> bool {
    let mut k = 0;
    while k < rest.len() && rest[k].is_ascii_whitespace() { k += 1; }
    k < rest.len() && rest[k] == b'='
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
    fn flags_static_and_dynamic_style_host() {
        assert_eq!(run(r#"<sail-card style="p-4">x</sail-card>"#), 1);
        assert_eq!(run(r#"<sail-card style={cond ? 'p-4' : 'p-2'}>x</sail-card>"#), 1);
        assert_eq!(run(r#"<sail-card style = "p-4">x</sail-card>"#), 1);
    }
    #[test]
    fn does_not_flag_lookalikes_or_props() {
        assert_eq!(run(r#"<sail-card padding="lg" data-sx="x">y</sail-card>"#), 0);
        assert_eq!(run(r#"<sail-card styled="x">y</sail-card>"#), 0);
    }
    #[test]
    fn is_warn_and_skips_comments() {
        let f = check(Path::new("t.sail"), r#"<sail-card style="p-4">x</sail-card>"#);
        assert!(matches!(f[0].severity, Severity::Warn));
        assert_eq!(run(r#"<!-- <sail-card style="p-9"> --> <sail-card>y</sail-card>"#), 0);
    }
}
