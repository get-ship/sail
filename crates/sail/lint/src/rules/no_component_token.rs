//! `sail-no-component-token` — L1 component token names are unnameable
//! in unsanctioned app `.sail` attributes (THEMES-1-ENFORCE / THEMES-2 §1).
//!
//! Style `bg(button)` is refused by the Style pipeline (`ThemeOwnedToken`).
//! This rule covers unsanctioned *attribute* sites: attr names that are
//! component-token paths (`button.bg=`, `card.border=`) and attr values
//! that look like a token ref on a free color-like attr.

use std::path::Path;

use super::ambient::{finding, is_ambient_app_path};
use crate::report::Finding;

pub const RULE_ID: &str = "sail-no-component-token";

const REMEDY: &str = "Component tokens are theme-owned; override via theme= or the theme document.";

/// Scan app `.sail` for unsanctioned component-token attribute sites.
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
        // Attr name start after whitespace / tag-name boundary.
        if i + 1 < bytes.len()
            && (bytes[i].is_ascii_whitespace() || bytes[i] == b'<')
            && is_name_start(bytes[i + 1])
        {
            let start = i + 1;
            let mut j = start;
            while j < bytes.len() && is_name_byte(bytes[j]) {
                j += 1;
            }
            if j > start {
                let name = &source[start..j];
                // Skip tags (`<button`) and sanctioned `theme`.
                let after_lt = bytes[i] == b'<';
                if !after_lt && name != "theme" && sail_style::is_theme_owned_token_name(name) {
                    let msg = format!(
                        "component token `{name}` is theme-owned; override via theme= or the theme document"
                    );
                    if let Some(f) = finding(path, source, start, RULE_ID, msg, REMEDY) {
                        findings.push(f);
                    }
                    i = j;
                    continue;
                }
            }
        }
        i += 1;
    }

    findings.sort_by_key(|f| (f.line, f.col));
    findings.dedup_by(|a, b| a.line == b.line && a.col == b.col);
    findings
}

fn is_name_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_name_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_unsanctioned_component_token_attr() {
        let f = check(
            Path::new("apps/demo/src/page.sail"),
            r#"<div button.bg="primary">x</div>"#,
        );
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert!(
            f[0].message.contains("theme-owned") && f[0].message.contains("theme="),
            "{}",
            f[0].message
        );
    }

    #[test]
    fn flags_card_foreground_attr() {
        let f = check(
            Path::new("apps/demo/routes/page.sail"),
            r#"<column card.foreground="primary">x</column>"#,
        );
        assert!(!f.is_empty(), "{f:?}");
        assert!(f[0].message.contains("card.foreground"), "{}", f[0].message);
    }

    #[test]
    fn allows_theme_attr_and_tags() {
        let f = check(
            Path::new("apps/demo/src/page.sail"),
            r#"<column theme="ship"><button>ok</button><card>x</card></column>"#,
        );
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn packages_exempt() {
        let src = r#"<div button.bg="primary">x</div>"#;
        assert!(check(Path::new("packages/components/sail/x.sail"), src).is_empty());
        assert!(check(Path::new("crates/sail/compiler/stdlib/button.sail"), src).is_empty());
    }
}
