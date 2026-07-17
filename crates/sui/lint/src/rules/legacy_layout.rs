//! `sui-legacy-layout` — the `<sui-stack>` / `<sui-cluster>` / `<sui-grid>` /
//! `<sui-center>` web-component primitives are superseded by the language-level
//! layout keywords (`<column>` / `<row>` / `<grid>` / `<center>`), which lower
//! to plain divs + token CSS with no shadow-DOM cost. This rule flags a legacy
//! layout element and names its keyword replacement.
//!
//! Severity `Warn` — advisory during the migration so existing builds don't
//! break mid-sweep. Once the tree is converted it graduates to `Error` (via the
//! floor policy) to prevent backsliding, and the primitives can be deleted.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-legacy-layout";

/// Legacy element → the keyword that replaces it, with a note on any prop shift.
const MAP: &[(&str, &str, &str)] = &[
    ("sui-stack", "column", "vertical flow; same gap/align props"),
    ("sui-cluster", "row", "horizontal flow that wraps; same gap/align/justify props"),
    ("sui-grid", "grid", "same gap/min/flow props"),
    ("sui-center", "center", "same gap prop"),
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
        if bytes[i] == b'<' {
            if let Some(&(legacy, keyword, note)) = MAP.iter().find(|&&(tag, _, _)| {
                source[i + 1..].starts_with(tag)
                    && matches!(
                        bytes.get(i + 1 + tag.len()),
                        Some(b) if b.is_ascii_whitespace() || *b == b'>' || *b == b'/'
                    )
            }) {
                let (line, col) = offset_to_line_col(source, i);
                findings.push(Finding {
                    file: path.to_path_buf(),
                    line,
                    col,
                    rule: RULE_ID,
                    severity: Severity::Warn,
                    message: format!(
                        "<{legacy}> is superseded by the <{keyword}> layout keyword ({note})"
                    ),
                    suggest: Some(format!(
                        "Replace <{legacy}> … </{legacy}> with <{keyword}> … </{keyword}> — the \
                         keyword lowers to a div with the same token CSS, no shadow DOM."
                    )),
                });
                i += 1 + legacy.len();
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
    for (i, b) in source.bytes().enumerate() {
        if i == offset {
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

    fn run(src: &str) -> Vec<String> {
        check(Path::new("t.sui"), src).into_iter().map(|f| f.message).collect()
    }

    #[test]
    fn flags_each_legacy_layout_element() {
        assert!(run(r#"<sui-stack gap="3">x</sui-stack>"#)[0].contains("<column>"));
        assert!(run(r#"<sui-cluster>x</sui-cluster>"#)[0].contains("<row>"));
        assert!(run(r#"<sui-grid>x</sui-grid>"#)[0].contains("<grid>"));
        assert!(run(r#"<sui-center>x</sui-center>"#)[0].contains("<center>"));
    }

    #[test]
    fn leaves_the_new_keywords_and_other_primitives_alone() {
        assert!(run(r#"<column gap="3"><sui-card>x</sui-card><row>y</row></column>"#).is_empty());
    }

    #[test]
    fn does_not_match_similar_tags_or_comments() {
        assert!(run(r#"<sui-stackable>x</sui-stackable>"#).is_empty());
        assert!(run(r#"<!-- <sui-stack> --> <sui-card>x</sui-card>"#).is_empty());
    }
}
