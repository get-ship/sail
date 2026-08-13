//! `sail-no-data-attr` — no `data-*` attributes in app `.sail` markup (SPEC-DOMDEC-1).
//!
//! All data-* (static or bound) are Errors. Selectors that target `[data-…]`
//! are also Errors. Tests should use role / name / text / authored id.
//! Compiler-emitted `data-sail-host` (L1 part identity) is framework-only —
//! authors never write it; this rule refuses authored stamps in app sources.

use std::path::Path;

use super::ambient::{finding, is_ambient_app_path};
use crate::report::Finding;

pub const RULE_ID: &str = "sail-no-data-attr";

const REMEDY: &str = "Do not use data-* in .sail markup. Prefer role, accessible name, \
     visible text, or authored id. Event wiring: on: handlers + closed-over state. \
     data-sail-host is compiler-only L1 part identity — never author it.";

const THEME_SUBSTRATE: &[&str] = &["data-theme", "data-product", "data-mode"];
const THEME_SUBSTRATE_REMEDY: &str = "Do not author theme substrate as data-*. Use theme= (theme id) \
     and mode= (light|dark). Override tokens in the theme document, \
     not data-theme / data-product / data-mode.";

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
            let is_host = source[i..].starts_with("[data-sail-host");
            let substrate = THEME_SUBSTRATE
                .iter()
                .copied()
                .find(|n| source[i + 1..].starts_with(n));
            let (msg, remedy) = if is_host {
                (
                    "data-sail-host selector is banned — compiler-only part identity; use role/name/text",
                    REMEDY,
                )
            } else if let Some(name) = substrate {
                (
                    match name {
                        "data-theme" => {
                            "data-theme is theme substrate — use theme= (theme id), not data-theme"
                        }
                        "data-product" => {
                            "data-product is theme substrate — use theme= (theme id), not data-product"
                        }
                        _ => {
                            "data-mode is theme substrate — use theme= (theme id) and mode= (light|dark)"
                        }
                    },
                    THEME_SUBSTRATE_REMEDY,
                )
            } else {
                (
                    "data-* selector is banned (style/test channel bypass)",
                    REMEDY,
                )
            };
            if let Some(f) = finding(path, source, i, RULE_ID, msg, remedy) {
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
            let is_host = source[i + 1..].starts_with("data-sail-host");
            let substrate = THEME_SUBSTRATE
                .iter()
                .copied()
                .find(|n| source[i + 1..].starts_with(n));
            let (msg, remedy) = if is_host {
                (
                    "data-sail-host is compiler-only part identity — not an author API",
                    REMEDY,
                )
            } else if let Some(name) = substrate {
                (
                    match name {
                        "data-theme" => {
                            "data-theme is theme substrate — use theme= (theme id), not data-theme"
                        }
                        "data-product" => {
                            "data-product is theme substrate — use theme= (theme id), not data-product"
                        }
                        _ => {
                            "data-mode is theme substrate — use theme= (theme id) and mode= (light|dark)"
                        }
                    },
                    THEME_SUBSTRATE_REMEDY,
                )
            } else {
                ("data-* attributes are banned in app .sail markup", REMEDY)
            };
            if let Some(f) = finding(path, source, i + 1, RULE_ID, msg, remedy) {
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
    fn flags_theme_substrate_and_teaches_theme_eq() {
        for (src, needle) in [
            (r#"<column data-theme="ship">x</column>"#, "data-theme"),
            (r#"<column data-product="sail">x</column>"#, "data-product"),
            (r#"<column data-mode="dark">x</column>"#, "data-mode"),
        ] {
            let f = check(Path::new("apps/sail-site/src/x.sail"), src);
            assert_eq!(f.len(), 1, "{needle}: {f:?}");
            assert_eq!(f[0].rule, RULE_ID);
            assert!(
                f[0].message.contains("theme=") && f[0].message.contains(needle),
                "{needle} message must teach theme=: {}",
                f[0].message
            );
            if let Some(s) = &f[0].suggest {
                assert!(
                    s.contains("theme="),
                    "{needle} remedy must name theme=: {s}"
                );
            }
        }
    }

    #[test]
    fn flags_authored_data_sail_host() {
        let f = check(
            Path::new("apps/studio/src/x.sail"),
            r#"<button data-sail-host="button">Save</button>"#,
        );
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert!(
            f[0].message.contains("compiler-only part identity"),
            "expected teach, got: {}",
            f[0].message
        );
    }

    #[test]
    fn flags_data_sail_host_selector() {
        let f = check(
            Path::new("apps/sail-site/src/x.sail"),
            r#".x [data-sail-host="tab"] { color: red }"#,
        );
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(
            f[0].message.contains("compiler-only part identity"),
            "expected teach, got: {}",
            f[0].message
        );
    }

    #[test]
    fn allows_component_tags_without_data_attrs() {
        let f = check(
            Path::new("apps/studio/src/x.sail"),
            r#"<edit-surface grow style={style.h(full)} />"#,
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

    #[test]
    fn packages_and_stdlib_paths_are_exempt() {
        let src = r#"<button data-sail-host="button">x</button>"#;
        assert!(
            check(Path::new("packages/components/sail/button.sail"), src).is_empty(),
            "packages exempt"
        );
        assert!(
            check(Path::new("crates/sail/compiler/stdlib/button.sail"), src).is_empty(),
            "stdlib exempt (compiler stamps, never authors)"
        );
    }
}
