//! `sail-style-concern` — retired. Retirement is a compile error (`RetiredSetter`).
//!
//! The rule id stays pinned (HARDENING-1 WI6). The body is a no-op: the
//! language no longer warn-and-keeps absorbed property setters.

use std::path::Path;

use crate::report::Finding;

pub const RULE_ID: &str = "sail-style-concern";

pub fn check(_path: &Path, _source: &str) -> Vec<Finding> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn warn_mode_is_gone() {
        let src = r#"<div style({ border({ width: 1, color: primary }) })>x</div>"#;
        let findings = check(&PathBuf::from("t.sail"), src);
        assert!(
            findings.is_empty(),
            "retirement is compile error, not lint: {findings:?}"
        );
    }
}
