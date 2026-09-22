//! `sail-controller-navigation` — route via `navigate()`, not raw history.
//!
//! Bans, anywhere inside a `*.controller.ts` file:
//! - `history.pushState(…)` / `window.history.pushState(…)`
//! - `history.replaceState(…)` / `window.history.replaceState(…)`
//!
//! These are hand-rolled SPA routing. The framework affordance is
//! `navigate(path)` (`@ship/sail-runtime`), which routes through the host
//! `RouterPort` — history + path matching + mounting + the route-change
//! notification — so a single call actually switches routes from any route
//! bundle. See ADR `unified-deck-router`.
//!
//! A deliberate full-page navigation (`window.location.href = …`,
//! `location.assign(…)`) is intentionally NOT flagged — a full reload (e.g.
//! after onboarding) is a legitimate, distinct intent from SPA routing.

use std::path::{Path, PathBuf};

use crate::report::{Finding, Severity};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

pub const RULE_ID: &str = "sail-controller-navigation";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }
    let mut visitor = NavVisitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    visitor.findings
}

struct NavVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
}

impl NavVisitor<'_> {
    fn report(&mut self, span: Span, method: &str) {
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message: format!(
                "controllers must not drive SPA navigation via `history.{method}(…)`. \
                 Call `navigate(path)` from @ship/sail-runtime — it routes through the host \
                 router (history + match + mount). For a deliberate full-page reload, \
                 `window.location` is fine."
            ),
            suggest: Some(
                "navigate(path)  // import { navigate } from '@ship/sail-runtime'".to_string(),
            ),
        });
    }
}

impl<'a> Visit<'a> for NavVisitor<'_> {
    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        if let Expression::StaticMemberExpression(m) = &expr.callee {
            let prop = m.property.name.as_str();
            if prop == "pushState" || prop == "replaceState" {
                self.report(expr.span, prop);
            }
        }
        self.visit_expression(&expr.callee);
        for arg in &expr.arguments {
            if let Some(e) = arg.as_expression() {
                self.visit_expression(e);
            }
        }
    }
}

fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let mut line = 1usize;
    let mut col = 1usize;
    let cap = offset.min(bytes.len());
    let mut i = 0;
    while i < cap {
        if bytes[i] == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
        i += 1;
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn flags_history_pushstate() {
        let src = "function go(p: string) { history.pushState({}, '', p) }";
        let f = check(Path::new("x.controller.ts"), src);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].rule, "sail-controller-navigation");
    }

    #[test]
    fn flags_window_history_replacestate() {
        let src = "window.history.replaceState({}, '', '/x')";
        let f = check(Path::new("x.controller.ts"), src);
        assert_eq!(f.len(), 1);
    }

    #[test]
    fn allows_window_location_full_reload() {
        // Deliberate full navigation is a distinct, legitimate intent.
        let src = "window.location.href = '/setup'";
        let f = check(Path::new("x.controller.ts"), src);
        assert!(f.is_empty());
    }

    #[test]
    fn allows_navigate_call() {
        let src = "navigate('/agents/42')";
        let f = check(Path::new("x.controller.ts"), src);
        assert!(f.is_empty());
    }
}
