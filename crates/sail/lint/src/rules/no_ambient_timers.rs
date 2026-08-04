//! `sail-no-ambient-timers` — setTimeout/setInterval/rAF at module top level.
//!
//! Nested inside functions is still a smell but lifecycle-bound helpers often
//! live in mount(); Part-2 flags **module/top-level** and direct controller
//! top-level statements only (conservative).

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_syntax::scope::ScopeFlags;
use oxc_span::SourceType;

use super::ambient::{finding, gate};
use crate::report::Finding;

pub const RULE_ID: &str = "sail-no-ambient-timers";
const REMEDY: &str =
    "Lifecycle-bound timers belong in runtime helpers / mount scopes — not ambient app top-level.";

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
        depth: 0,
    };
    v.visit_program(&ret.program);
    v.findings.sort_by_key(|f| (f.line, f.col));
    v.findings
}

struct Visitor<'a> {
    path: &'a Path,
    source: &'a str,
    findings: Vec<Finding>,
    /// Function/class nesting depth — 0 = module top level.
    depth: u32,
}

fn is_timer_name(name: &str) -> bool {
    matches!(
        name,
        "setTimeout" | "setInterval" | "requestAnimationFrame" | "requestIdleCallback"
    )
}

impl<'a> Visit<'a> for Visitor<'_> {
    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        self.depth += 1;
        walk::walk_function(self, func, flags);
        self.depth -= 1;
    }

    fn visit_arrow_function_expression(&mut self, expr: &ArrowFunctionExpression<'a>) {
        self.depth += 1;
        walk::walk_arrow_function_expression(self, expr);
        self.depth -= 1;
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        self.depth += 1;
        walk::walk_class(self, class);
        self.depth -= 1;
    }

    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        if self.depth == 0 {
            let name = match &expr.callee {
                Expression::Identifier(id) => Some(id.name.as_str()),
                Expression::StaticMemberExpression(m)
                    if matches!(
                        &m.object,
                        Expression::Identifier(id)
                            if matches!(id.name.as_str(), "window" | "globalThis" | "self")
                    ) =>
                {
                    Some(m.property.name.as_str())
                }
                _ => None,
            };
            if let Some(n) = name.filter(|n| is_timer_name(n)) {
                if let Some(f) = finding(
                    self.path,
                    self.source,
                    expr.span.start as usize,
                    RULE_ID,
                    format!("module top-level `{n}` is banned in app sources"),
                    REMEDY,
                ) {
                    self.findings.push(f);
                }
            }
        }
        walk::walk_call_expression(self, expr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_level_raf_errors() {
        assert_eq!(
            check(
                Path::new("apps/studio/src/x.ts"),
                "requestAnimationFrame(() => {})"
            )
            .len(),
            1
        );
    }

    #[test]
    fn inside_function_ok() {
        assert_eq!(
            check(
                Path::new("apps/studio/src/x.ts"),
                "function f() { requestAnimationFrame(() => {}) }"
            )
            .len(),
            0
        );
    }
}
