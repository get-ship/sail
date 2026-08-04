//! `sail-no-ambient-events` — no window/document addEventListener in app sources.

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;

use super::ambient::{finding, gate};
use crate::report::Finding;

pub const RULE_ID: &str = "sail-no-ambient-events";

const REMEDY: &str = "Use element on: handlers or a catalog use: action; do not bind \
     window/document listeners in app modules.";

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

fn is_window_or_document(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::Identifier(id) => matches!(id.name.as_str(), "window" | "document" | "self"),
        Expression::StaticMemberExpression(m) if m.property.name.as_str() == "document" => {
            matches!(
                &m.object,
                Expression::Identifier(id) if matches!(id.name.as_str(), "window" | "globalThis" | "self")
            )
        }
        _ => false,
    }
}

impl<'a> Visit<'a> for Visitor<'_> {
    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        if let Expression::StaticMemberExpression(m) = &expr.callee {
            let meth = m.property.name.as_str();
            if matches!(meth, "addEventListener" | "removeEventListener")
                && is_window_or_document(&m.object)
            {
                if let Some(f) = finding(
                    self.path,
                    self.source,
                    expr.span.start as usize,
                    RULE_ID,
                    format!("app sources do not call `{meth}` on window/document"),
                    REMEDY,
                ) {
                    self.findings.push(f);
                }
            }
        }
        oxc_ast_visit::walk::walk_call_expression(self, expr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn run(src: &str) -> usize {
        check(Path::new("apps/studio/src/x.ts"), src).len()
    }

    #[test]
    fn flags_window_listener() {
        assert_eq!(run("window.addEventListener('keydown', () => {})"), 1);
    }

    #[test]
    fn allow_comment() {
        let src = "// sail-allow sail-no-ambient-events\nwindow.addEventListener('x', () => {})";
        assert_eq!(run(src), 0);
    }

    #[test]
    fn packages_out_of_scope() {
        assert_eq!(
            check(Path::new("packages/foo/src/x.ts"), "window.addEventListener('x',()=>{})").len(),
            0
        );
    }

    #[test]
    fn path_gate() {
        let _ = PathBuf::from("apps/studio/src/x.ts");
        assert!(super::super::ambient::is_ambient_app_path(Path::new(
            "apps/studio/routes/edit/page.sail"
        )));
    }
}
