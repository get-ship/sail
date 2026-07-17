//! `sui-no-script` — a raw `<script>` block is the footgun the SFC bans.
//!
//! In the single-file component, imperative logic has exactly one sanctioned
//! home: the `<controller>` block (constrained, lockdown-linted, type-checked).
//! A bare `<script>` is "run arbitrary code, anywhere" — the exact hole the
//! controller discipline exists to close. Flag it and point authors at
//! `<controller>`.
//!
//! Scans the raw source (pre-preprocess), skips HTML comments. `<controller>`,
//! `<scope>`, and the residual `<style>` are their own sanctioned blocks and
//! are not touched here.
use crate::report::{Finding, Severity};
use std::path::Path;

pub const RULE_ID: &str = "sail-no-script";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let bytes = source.as_bytes();
    let mut findings = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        // Only slice `source[i..]` when `i` is at a `<` (always a char boundary);
        // this keeps the scan UTF-8-safe across multi-byte template content.
        if bytes[i] != b'<' {
            i += 1;
            continue;
        }
        if source[i..].starts_with("<!--") {
            match source[i..].find("-->") {
                Some(end) => {
                    i += end + 3;
                    continue;
                }
                None => break,
            }
        }
        if source[i + 1..].starts_with("script") {
            let after = source[i + 1..].as_bytes().get("script".len()).copied();
            if matches!(after, Some(b) if b == b'>' || b == b'/' || b.is_ascii_whitespace()) {
                let (line, col) = offset_to_line_col(source, i);
                findings.push(Finding {
                    file: path.to_path_buf(),
                    line,
                    col,
                    rule: RULE_ID,
                    severity: Severity::Error,
                    message: "raw <script> is banned — imperative logic lives in the \
                              <controller> block (one constrained, type-checked home). A bare \
                              <script> is arbitrary code with no guardrails."
                        .to_string(),
                    suggest: Some(
                        "Move the code into a <controller> block: `<controller>\n  import … \n  \
                         export function onX() { … }\n</controller>`. Imports are allowed there."
                            .to_string(),
                    ),
                });
            }
        }
        i += 1;
    }
    findings
}

fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
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
