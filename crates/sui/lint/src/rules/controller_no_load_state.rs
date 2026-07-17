//! `sui-controller-no-load-state` — loading/error/ok are RENDER states; branch
//! on them in the template, not the controller.
//!
//! The anti-pattern (and how the surface grows): a controller takes a live
//! source and re-derives `loading`/`error` booleans from its `.kind`, e.g.
//!
//! ```ignore
//! const loading  = derived(() => src().kind === 'loading')
//! const loadError = derived(() => src().kind === 'error' ? src().error.message : '')
//! ```
//!
//! That re-implements, in the ViewModel, what `.sui` already gives for free:
//! `{#while-loading of src}` / `{#while-error err of src}` / `{#if src().kind ===
//! 'ok'}`. The controller's job is the DATA (`unwrap`/`unwrapOr` for the value,
//! `derived` for transforms) — not the load-state flow. Expose the live source
//! and let the template own the states.
//!
//! Flags any `<member>.kind` compared (`==`/`===`/`!=`/`!==`) to a `LiveState`
//! discriminant literal (`'loading'`/`'error'`/`'ok'`) inside a `*.controller.ts`.
//! `unwrap(src)` (the sanctioned value accessor) is NOT flagged — that's the
//! point: read the value in the controller, branch the state in the template.

use std::path::{Path, PathBuf};

use crate::report::{Finding, Severity};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

pub const RULE_ID: &str = "sail-controller-no-load-state";

const KIND_LITERALS: &[&str] = &["loading", "error", "ok"];

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    // Controller-only — templates SHOULD branch on `.kind`. View controllers are
    // `<dir>/controller.ts`; component controllers are `<tag>.controller.ts`.
    let p = path.to_string_lossy();
    if !(p.ends_with("/controller.ts") || p.ends_with(".controller.ts") || p == "controller.ts") {
        return Vec::new();
    }
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }
    let mut visitor = LoadStateVisitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    visitor.findings
}

struct LoadStateVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
}

impl LoadStateVisitor<'_> {
    fn report(&mut self, span: Span) {
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Warn,
            message:
                "controllers must not branch on a live source's `.kind` — loading/error/ok \
                 are render states. Expose the live source and let the template own them via \
                 `{#while-loading of src}` / `{#while-error err of src}` / `{#if src().kind === \
                 'ok'}`. Use `unwrap`/`unwrapOr` for the value in the controller."
                    .to_string(),
            suggest: Some(
                "return the live source from `scope`; branch states in the `.sui` with \
                 {#while-loading}/{#while-error}"
                    .to_string(),
            ),
        });
    }
}

/// True for `<anything>.kind` (a `.kind` property access).
fn is_kind_access(expr: &Expression) -> bool {
    matches!(expr, Expression::StaticMemberExpression(m) if m.property.name.as_str() == "kind")
}

/// True for a `LiveState` discriminant string literal.
fn is_kind_literal(expr: &Expression) -> bool {
    matches!(expr, Expression::StringLiteral(s) if KIND_LITERALS.contains(&s.value.as_str()))
}

impl<'a> Visit<'a> for LoadStateVisitor<'_> {
    fn visit_binary_expression(&mut self, expr: &BinaryExpression<'a>) {
        if matches!(
            expr.operator,
            BinaryOperator::Equality
                | BinaryOperator::Inequality
                | BinaryOperator::StrictEquality
                | BinaryOperator::StrictInequality
        ) && ((is_kind_access(&expr.left) && is_kind_literal(&expr.right))
                || (is_kind_literal(&expr.left) && is_kind_access(&expr.right)))
        {
            self.report(expr.span);
        }
        self.visit_expression(&expr.left);
        self.visit_expression(&expr.right);
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

    fn n(src: &str) -> usize {
        check(Path::new("views/x/controller.ts"), src).len()
    }

    #[test]
    fn flags_kind_loading_and_error() {
        assert_eq!(n("const l = derived(() => src().kind === 'loading')"), 1);
        assert_eq!(n("const e = derived(() => s().kind === 'error' ? x : '')"), 1);
        assert_eq!(n("function f() { if (src().kind !== 'ok') return 1; return 0 }"), 1);
    }

    #[test]
    fn allows_unwrap_value_access() {
        // The sanctioned controller-side value accessor — not flagged.
        assert_eq!(n("const items = derived(() => unwrap(src)?.data ?? [])"), 0);
        assert_eq!(n("const v = unwrapOr(src, EMPTY)"), 0);
    }

    #[test]
    fn ignores_non_controller_files() {
        // Templates compile their own `.kind` branches — only controllers are governed.
        assert_eq!(check(Path::new("page.sui"), "src().kind === 'loading'").len(), 0);
        assert_eq!(check(Path::new("views/x/decorate.ts"), "x.kind === 'ok'").len(), 0);
    }
}
