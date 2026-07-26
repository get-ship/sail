//! `sail-controller-empty` — an empty controller is cargo-cult; omit the block.
//!
//! Flags `export default controller(() => ({}))` and equivalent no-op bodies
//! that only return an empty object expression — no statements, no lifecycle
//! side effects (`mount` / `effect` / …), no non-empty returned scope.
//!
//! Rationale: `<controller>` is optional. A `.sail` file with only a template
//! (and optional `<scope>`) is valid. An empty setup function that returns `{}`
//! contributes nothing to the template's vocabulary and confuses readers into
//! thinking a controller is always required.
//!
//! **Not flagged:** a controller that returns properties, or one whose block
//! body has any statement other than a bare `return {}` (e.g. a `mount(...)`
//! call that returns `{}` still has work to do).
//!
//! Severity: Error (compile-hard via the controller lint gate).

use std::path::{Path, PathBuf};

use crate::report::{Finding, Severity};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

pub const RULE_ID: &str = "sail-controller-empty";

/// Lint a controller source (inline `<controller>` body or `.controller.ts`).
pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }

    let mut visitor = EmptyVisitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    visitor.findings
}

struct EmptyVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
}

impl EmptyVisitor<'_> {
    fn report(&mut self, span: Span) {
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message: "empty controller — omit the `<controller>` block. A \
                      controller is only needed when it returns state, handlers, \
                      or runs lifecycle work; `controller(() => ({}))` is a no-op."
                .to_string(),
            suggest: Some(
                "Delete the entire `<controller>…</controller>` block (or the \
                 paired controller module). Keep the template (and optional \
                 `<scope>`) only."
                    .to_string(),
            ),
        });
    }
}

impl<'a> Visit<'a> for EmptyVisitor<'_> {
    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        if is_controller_callee(&expr.callee)
            && let Some(setup) = expr.arguments.first().and_then(|a| a.as_expression())
            && is_empty_setup(setup)
        {
            self.report(expr.span);
        }
        // Continue walking so nested calls are also examined (rare for this
        // rule, but cheap and keeps Visit consistent).
        self.visit_expression(&expr.callee);
        for arg in &expr.arguments {
            if let Some(e) = arg.as_expression() {
                self.visit_expression(e);
            }
        }
    }
}

/// `controller` or `controller<…>` (TSInstantiationExpression / call on ident).
fn is_controller_callee(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::Identifier(id) => id.name.as_str() == "controller",
        // controller<Scope>(…)
        Expression::TSInstantiationExpression(te) => is_controller_callee(&te.expression),
        // (controller)(…) — rare parens
        Expression::ParenthesizedExpression(p) => is_controller_callee(&p.expression),
        _ => false,
    }
}

/// True when the setup function body is a pure no-op that returns `{}`.
fn is_empty_setup(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::ArrowFunctionExpression(arrow) => {
            if arrow.expression {
                // `() => ({})` — body is a single expression statement whose
                // expression is the returned value.
                arrow
                    .body
                    .statements
                    .first()
                    .and_then(|s| match s {
                        Statement::ExpressionStatement(es) => Some(&es.expression),
                        _ => None,
                    })
                    .is_some_and(is_empty_object_expr)
            } else {
                is_empty_block_body(&arrow.body.statements)
            }
        }
        Expression::FunctionExpression(fun) => fun
            .body
            .as_ref()
            .is_some_and(|b| is_empty_block_body(&b.statements)),
        Expression::ParenthesizedExpression(p) => is_empty_setup(&p.expression),
        _ => false,
    }
}

/// Block body is empty-controller only when every statement is either empty
/// or a `return {}` / `return {};`. Any other statement (const, mount call,
/// effect, …) means the controller is doing work.
fn is_empty_block_body(stmts: &[Statement<'_>]) -> bool {
    if stmts.is_empty() {
        // `() => {}` — returns undefined, still a no-op controller.
        return true;
    }
    for s in stmts {
        match s {
            Statement::EmptyStatement(_) => {}
            Statement::ReturnStatement(r) => match &r.argument {
                None => {} // bare `return;` — no-op
                Some(arg) if is_empty_object_expr(arg) => {}
                Some(_) => return false,
            },
            _ => return false,
        }
    }
    // Pure empty blocks and blocks that only return {} are empty controllers.
    true
}

fn is_empty_object_expr(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::ObjectExpression(obj) => obj.properties.is_empty(),
        Expression::ParenthesizedExpression(p) => is_empty_object_expr(&p.expression),
        // `({}) as const` / type assertion — still empty payload.
        Expression::TSAsExpression(a) => is_empty_object_expr(&a.expression),
        Expression::TSSatisfiesExpression(s) => is_empty_object_expr(&s.expression),
        Expression::TSTypeAssertion(t) => is_empty_object_expr(&t.expression),
        Expression::TSNonNullExpression(n) => is_empty_object_expr(&n.expression),
        _ => false,
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

    fn lint(src: &str) -> Vec<Finding> {
        check(Path::new("page.sail"), src)
    }

    #[test]
    fn flags_expression_empty_object() {
        let f = lint("export default controller(() => ({}))\n");
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
    }

    #[test]
    fn flags_block_return_empty() {
        let f = lint("export default controller(() => { return {} })\n");
        assert_eq!(f.len(), 1, "{f:?}");
    }

    #[test]
    fn flags_empty_block() {
        let f = lint("export default controller(() => {})\n");
        assert_eq!(f.len(), 1, "{f:?}");
    }

    #[test]
    fn allows_returned_state() {
        let f = lint(
            "export default controller(() => {\n  const n = signal(0)\n  return { n }\n})\n",
        );
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn allows_mount_with_empty_return() {
        let f = lint(
            "export default controller(() => {\n  mount((h) => { h.setAttribute('role', 'list') })\n  return {}\n})\n",
        );
        assert!(f.is_empty(), "mount side-effect is real work: {f:?}");
    }

    #[test]
    fn allows_expression_with_props() {
        let f = lint("export default controller(() => ({ count: signal(0) }))\n");
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn flags_typed_ctx_empty() {
        let f = lint(
            "export default controller((_ctx: ControllerScope<Scope>) => ({}))\n",
        );
        assert_eq!(f.len(), 1, "{f:?}");
    }
}
