//! Compile-time icon role resolve (SPEC-ICONS-1).
//!
//! `<icon name="…">` is a closed role. Unknown names fail with close
//! matches — same teaching shape as every other closed-union error.

use std::path::Path;

use crate::diagnostics::CompileError;
use crate::icons::{RoleRegistry, collect_static_roles, unknown_role_message};
use crate::ir::Program;

const RULE_UNKNOWN_ICON: &str = "sail-unknown-icon";

pub fn check(program: &Program, source_path: Option<&Path>) -> Result<(), CompileError> {
    let mut errs = collect(program, source_path);
    match errs.len() {
        0 => Ok(()),
        _ => Err(errs.remove(0)),
    }
}

pub fn collect(program: &Program, source_path: Option<&Path>) -> Vec<CompileError> {
    let registry = RoleRegistry::load(source_path);
    let mut out = Vec::new();
    for (role, span) in collect_static_roles(&program.template) {
        if registry.contains(&role) {
            continue;
        }
        out.push(
            CompileError::new(unknown_role_message(&role, &registry))
                .with_span(span)
                .with_category(RULE_UNKNOWN_ICON),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;

    fn errs(src: &str) -> Vec<CompileError> {
        let program = parse(src).expect("parse");
        collect(&program, None)
    }

    #[test]
    fn known_role_is_silent() {
        assert!(errs(r#"<icon name="settings"></icon>"#).is_empty());
        assert!(errs(r#"<icon name="search"></icon>"#).is_empty());
        assert!(errs(r#"<button name="submit"></button>"#).is_empty());
    }

    #[test]
    fn unknown_role_lists_close_matches() {
        let got = errs(r#"<icon name="setttings"></icon>"#);
        assert_eq!(got.len(), 1, "{got:?}");
        assert_eq!(got[0].category.as_deref(), Some(RULE_UNKNOWN_ICON));
        assert!(got[0].message.contains("setttings"), "{}", got[0].message);
        assert!(
            got[0].message.contains("settings"),
            "close match missing: {}",
            got[0].message
        );
    }

    #[test]
    fn pack_prefix_is_a_compile_error() {
        let got = errs(r#"<icon name="lucide:settings"></icon>"#);
        assert_eq!(got.len(), 1, "{got:?}");
        assert!(
            got[0].message.contains("roles, not pack prefixes"),
            "{}",
            got[0].message
        );
        assert!(got[0].message.contains("`settings`"), "{}", got[0].message);
    }

    #[test]
    fn dynamic_name_is_not_this_pass() {
        // I4 enumerates closed unions; unbounded expr is a later gate.
        assert!(errs(r#"<icon name={expr()}></icon>"#).is_empty());
    }
}
