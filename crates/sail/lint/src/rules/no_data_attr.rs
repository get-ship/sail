//! `sail-no-data-attr` — no `data-*` attributes in app `.sail` markup (SPEC-DOMDEC-1).
//!
//! All data-* (static or bound) are Errors. Selectors that target `[data-…]`
//! are also Errors. Tests should use role / name / text / authored id.

use std::path::Path;

use super::ambient::{finding, is_ambient_app_path};
use crate::report::Finding;

pub const RULE_ID: &str = "sail-no-data-attr";

const REMEDY: &str = "Do not use data-* in .sail markup. Prefer role, accessible name, \
     visible text, or authored id. Event wiring: on: handlers + closed-over state.";

/// Scan .sail source for data-* attributes and [data- selectors.
pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    if !is_ambient_app_path(path) {
        return Vec::new();
    }
    if path.extension().and_then(|e| e.to_str()) != Some("sail") {
        return Vec::new();
    }

    let mut findings = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        // Skip HTML comments <!-- ... -->
        if bytes[i] == b'<' && source[i..].starts_with("<!--") {
            if let Some(end) = source[i..].find("-->") {
                i += end + 3;
                continue;
            }
        }
        // Skip // line comments (controller prose may mention data-*)
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'/') {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        // Skip /* block comments */
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
            if let Some(end) = source[i + 2..].find("*/") {
                i += 2 + end + 2;
                continue;
            }
        }
        // [data- selector
        if i + 6 <= bytes.len() && &bytes[i..i + 6] == b"[data-" {
            if let Some(f) = finding(
                path,
                source,
                i,
                RULE_ID,
                "data-* selector is banned (style/test channel bypass)",
                REMEDY,
            ) {
                findings.push(f);
            }
            i += 6;
            continue;
        }
        // Attribute form: whitespace + data-name
        if i + 6 <= bytes.len()
            && bytes[i].is_ascii_whitespace()
            && &bytes[i + 1..i + 6] == b"data-"
            && bytes.get(i + 6).is_some_and(|b| b.is_ascii_alphabetic())
        {
            if let Some(f) = finding(
                path,
                source,
                i + 1,
                RULE_ID,
                "data-* attributes are banned in app .sail markup",
                REMEDY,
            ) {
                findings.push(f);
            }
            i += 6;
            continue;
        }
        i += 1;
    }

    findings.sort_by_key(|f| (f.line, f.col));
    findings.dedup_by(|a, b| a.line == b.line && a.col == b.col);
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_data_attr() {
        let f = check(
            Path::new("apps/studio/src/x.sail"),
            r#"<button data-action="go">x</button>"#,
        );
        assert!(!f.is_empty(), "{f:?}");
    }

    #[test]
    fn allows_component_tags_without_data_attrs() {
        let f = check(
            Path::new("apps/studio/src/x.sail"),
            r#"<edit-surface grow sx={style.h(full)} />"#,
        );
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn does_not_eat_tag_names() {
        // Critical regression: never flag or rewrite edit-surface / sail-studio-chrome.
        let f = check(
            Path::new("apps/studio/routes/_layout.sail"),
            r#"<sail-studio-chrome></sail-studio-chrome>
<edit-surface grow />"#,
        );
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn flags_data_selector() {
        let f = check(
            Path::new("apps/studio/src/x.sail"),
            r#".foo [data-selected] { color: red }"#,
        );
        assert!(!f.is_empty());
    }
}
