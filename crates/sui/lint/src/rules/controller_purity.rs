//! `sui-controller-purity` — controllers do not write raw HTML.
//!
//! Bans, anywhere inside a `*.controller.ts` file:
//! - `<expr>.innerHTML  = …`
//! - `<expr>.outerHTML  = …`
//! - `<expr>.insertAdjacentHTML(…)` calls
//!
//! Dynamic markup belongs in the `.sui` template via `{#if}/{#for}/{#match}`
//! and the binding primitives (`bindAttr`/`bindText`/`bindClass`/`bindValue`).
//! Controllers own state, refs, lifecycle, and async I/O.

use std::path::{Path, PathBuf};

use crate::report::{Finding, Severity};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

pub const RULE_ID: &str = "sui-controller-purity";

/// Lint a controller `.ts` source. Returns findings ordered by source
/// position. Parse errors are silently swallowed — the SUI compiler
/// (and tsc) own parse-error reporting; we only care about clean,
/// well-formed sources.
pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let ret = Parser::new(&allocator, source, source_type).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }

    let mut visitor = PurityVisitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    visitor.findings
}

struct PurityVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
}

impl<'src> PurityVisitor<'src> {
    fn report(&mut self, span: Span, message: String) {
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message,
            suggest: Some(
                "Move dynamic markup into the .sui template ({#if}/{#for}/{#match}). \
                 For HTML/SVG-blob cases, use bindHtml()."
                    .to_string(),
            ),
        });
    }
}

impl<'a, 'src> Visit<'a> for PurityVisitor<'src> {
    fn visit_assignment_expression(&mut self, expr: &AssignmentExpression<'a>) {
        // <expr>.innerHTML = … / <expr>.outerHTML = …
        if let Some(prop) = static_member_property(&expr.left)
            && (prop == "innerHTML" || prop == "outerHTML") {
                self.report(
                    expr.span,
                    format!(
                        "controllers do not write `.{prop}`. Render dynamic markup from \
                         the .sui template, or use bindHtml() for HTML-blob cases."
                    ),
                );
            }
        // Continue into both sides so chained patterns are also caught.
        oxc_ast_visit::walk::walk_assignment_target(self, &expr.left);
        self.visit_expression(&expr.right);
    }

    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        // <expr>.insertAdjacentHTML(...)
        if let Expression::StaticMemberExpression(m) = &expr.callee
            && m.property.name.as_str() == "insertAdjacentHTML" {
                self.report(
                    expr.span,
                    "controllers do not call `.insertAdjacentHTML(...)`. Render dynamic \
                     markup from the .sui template, or use bindHtml() for HTML-blob cases."
                        .to_string(),
                );
            }
        // Continue walking arguments + callee descendants for nested cases.
        self.visit_expression(&expr.callee);
        for arg in &expr.arguments {
            if let Some(e) = arg.as_expression() {
                self.visit_expression(e);
            }
        }
    }
}

/// Extract the property name from an `AssignmentTarget` if the target
/// is a static member expression (`x.foo`, `x.y.foo`, etc.).
///
/// Returns `None` for other shapes (identifiers, computed members,
/// destructuring targets, etc.).
fn static_member_property<'a>(target: &AssignmentTarget<'a>) -> Option<&'a str> {
    let inner = target.as_member_expression()?;
    if let MemberExpression::StaticMemberExpression(m) = inner {
        Some(m.property.name.as_str())
    } else {
        None
    }
}

/// Convert a byte offset into the source string into a 1-based
/// `(line, col)` pair. Used to populate `Finding.line` / `Finding.col`,
/// which the LSP and CLI both consume.
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
