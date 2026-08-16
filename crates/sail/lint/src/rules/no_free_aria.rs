//! `sail-no-free-aria` — free `aria-*` / free ARIA `role=` in app `.sail` markup.
//!
//! SPEC-ARIA-1: authors never write role/aria; semantics are authored as
//! semantic props (`label`, `current`, `expanded`, …) and L1 anatomy stamps
//! fixed roles. **Warning this cycle** (Error next).
//!
//! Path-gated like DOMDEC data-* (apps only). Stdlib / packages are exempt —
//! anatomy declarations live there.

use std::path::Path;

use super::ambient::{finding_warn, is_ambient_app_path};
use crate::report::Finding;

pub const RULE_ID: &str = "sail-no-free-aria";

const REMEDY: &str = "Do not author free aria-* or role=. Use semantic props: \
     label, description, expanded, selected, pressed, current, level, live, \
     invalid, hidden. Fixed roles come from native tags or L1 anatomy \
     (navbar/dialog/tabs/…). Design identity that used role= → token=.";

/// Scan .sail source for free aria-* attributes and free role=.
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
        // Skip // line comments (controller prose may mention aria-*)
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
        // Skip <code>…</code> doc samples (prose, not authoring)
        if bytes[i] == b'<' && source[i..].starts_with("<code") {
            if let Some(end) = source[i..].find("</code>") {
                i += end + 7;
                continue;
            }
        }

        // Attribute form: whitespace + aria-name
        if i + 6 <= bytes.len()
            && bytes[i].is_ascii_whitespace()
            && &bytes[i + 1..i + 6] == b"aria-"
            && bytes.get(i + 6).is_some_and(|b| b.is_ascii_alphabetic())
        {
            if let Some(f) = finding_warn(
                path,
                source,
                i + 1,
                RULE_ID,
                "free aria-* is not authoring IR — use a semantic prop",
                REMEDY,
            ) {
                findings.push(f);
            }
            i += 6;
            continue;
        }

        // role= attribute (ARIA role). Design-token misuse is also flagged —
        // migrate to token= / from= / native element.
        if i + 5 <= bytes.len()
            && bytes[i].is_ascii_whitespace()
            && &bytes[i + 1..i + 5] == b"role"
            && bytes
                .get(i + 5)
                .is_some_and(|&b| b == b'=' || b.is_ascii_whitespace())
        {
            if let Some(f) = finding_warn(
                path,
                source,
                i + 1,
                RULE_ID,
                "free role= is not authoring IR — use native tags / L1 anatomy / token=",
                REMEDY,
            ) {
                findings.push(f);
            }
            i += 5;
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
    use crate::report::Severity;

    #[test]
    fn flags_aria_label_in_apps() {
        let f = check(
            Path::new("apps/studio/src/x.sail"),
            r#"<button aria-label="Go">x</button>"#,
        );
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].rule, RULE_ID);
        assert_eq!(f[0].severity, Severity::Warn);
        assert!(f[0].message.contains("aria"));
    }

    #[test]
    fn flags_role_in_apps() {
        let f = check(
            Path::new("apps/sail-site/src/x.sail"),
            r#"<div role="navigation">x</div>"#,
        );
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].severity, Severity::Warn);
    }

    #[test]
    fn allows_semantic_props() {
        let f = check(
            Path::new("apps/studio/src/x.sail"),
            r#"<button label="Go" current="page" expanded={open()}>x</button>"#,
        );
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn skips_stdlib_and_packages() {
        assert!(
            check(
                Path::new("crates/sail/compiler/stdlib/alert/alert.sail"),
                r#"<row role="status" aria-live="polite">x</row>"#,
            )
            .is_empty()
        );
        assert!(
            check(
                Path::new("packages/components/sail/x/src/y.sail"),
                r#"<div role="listbox" aria-label="x">x</div>"#,
            )
            .is_empty()
        );
    }

    #[test]
    fn skips_code_prose() {
        let f = check(
            Path::new("apps/sail-site/routes/sail/elements/page.sail"),
            r#"<p>See <code>role="tab"</code> and <code>aria-selected</code>.</p>"#,
        );
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn skips_comments() {
        let f = check(
            Path::new("apps/studio/src/x.sail"),
            "<!-- aria-label and role= in comments are fine -->\n<button label=\"x\">y</button>",
        );
        assert!(f.is_empty(), "{f:?}");
    }
}
