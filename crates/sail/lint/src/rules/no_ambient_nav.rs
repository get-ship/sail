//! `sail-no-ambient-nav` — no location./history. outside deck route signals.

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;

use super::ambient::{finding, gate};
use crate::report::Finding;

pub const RULE_ID: &str = "sail-no-ambient-nav";
const REMEDY: &str = "Use ctx.route / navigate — not ambient location or history.";

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
        if matches!(n, "location" | "history") {
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
    fn flags_location_pathname() {
        assert_eq!(
            check(
                Path::new("apps/studio/src/x.ts"),
                "const p = location.pathname"
            )
            .len(),
            1
        );
    }
}
