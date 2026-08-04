//! `sail-no-ambient-storage` — no localStorage/sessionStorage/indexedDB/cookie.

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;

use super::ambient::{finding, gate};
use crate::report::Finding;

pub const RULE_ID: &str = "sail-no-ambient-storage";
const REMEDY: &str = "Use a persisted signal (or other ctx primitive) — not localStorage/sessionStorage.";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    if !gate(path) {
        return Vec::new();
    }
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }
    let mut v = Visitor {
        path,
        source,
        findings: Vec::new(),
    };
    v.visit_program(&ret.program);
    v.findings.sort_by_key(|f| (f.line, f.col));
    v.findings
}

struct Visitor<'a> {
    path: &'a Path,
    source: &'a str,
    findings: Vec<Finding>,
}

impl<'a> Visit<'a> for Visitor<'_> {
    fn visit_identifier_reference(&mut self, id: &IdentifierReference<'a>) {
        let n = id.name.as_str();
        if matches!(n, "localStorage" | "sessionStorage" | "indexedDB") {
            if let Some(f) = finding(
                self.path,
                self.source,
                id.span.start as usize,
                RULE_ID,
                format!("app sources do not use ambient `{n}`"),
                REMEDY,
            ) {
                self.findings.push(f);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_session_storage() {
        assert_eq!(
            check(
                Path::new("apps/studio/src/x.ts"),
                "sessionStorage.getItem('k')"
            )
            .len(),
            1
        );
    }

    #[test]
    fn typeof_guard_still_hits_identifier() {
        // Spec: ban ambient storage access; typeof still mentions the id.
        // Callers should use a platform primitive instead of typeof guards.
        assert!(
            !check(
                Path::new("apps/studio/src/x.ts"),
                "if (typeof sessionStorage === 'undefined') {}"
            )
            .is_empty()
        );
    }
}
