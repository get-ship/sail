//! `sail-action-signature` — author action exports must match the Action shape.
//!
//! Enforced shape: `(node, param?) => { update?, destroy? } | void`
//!
//! Lightweight AST check (not full tsc):
//! - Every **value** export that is a function / arrow must have 1 or 2 params.
//! - Type-only exports are ignored.
//! - Re-exports are ignored (defining module is linted at its path).
//!
//! A zero-arg export or a 3+-arg export is an ERROR naming the Action signature.

use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-action-signature";

const REMEDY: &str = "Action signature is `(node: Element, param?) => { update?, destroy? } | void`. \
     Import `type { Action } from '@ship/sail-runtime'` and annotate the export.";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }
    let mut visitor = Visitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    visitor.findings.sort_by_key(|f| (f.line, f.col));
    visitor.findings
}

struct Visitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
}

impl Visitor<'_> {
    fn report(&mut self, span: Span, message: String) {
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message,
            suggest: Some(REMEDY.to_string()),
        });
    }

    fn check_fn_params(&mut self, span: Span, name: &str, params: &FormalParameters<'_>) {
        let n = params.items.len() + usize::from(params.rest.is_some());
        if n == 0 || n > 2 {
            self.report(
                span,
                format!(
                    "author action `{name}` must match `(node, param?) => …` — found {n} parameter(s)"
                ),
            );
        }
    }

    fn check_expr_as_action(&mut self, span: Span, name: &str, expr: &Expression<'_>) {
        match expr {
            Expression::ArrowFunctionExpression(a) => {
                self.check_fn_params(span, name, &a.params);
            }
            Expression::FunctionExpression(f) => {
                self.check_fn_params(span, name, &f.params);
            }
            Expression::ParenthesizedExpression(p) => {
                self.check_expr_as_action(span, name, &p.expression);
            }
            Expression::TSAsExpression(a) => {
                self.check_expr_as_action(span, name, &a.expression);
            }
            Expression::TSSatisfiesExpression(a) => {
                self.check_expr_as_action(span, name, &a.expression);
            }
            Expression::TSTypeAssertion(a) => {
                self.check_expr_as_action(span, name, &a.expression);
            }
            // Non-function exports (types, constants) are not actions.
            _ => {}
        }
    }
}

impl<'a> Visit<'a> for Visitor<'_> {
    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        if decl.export_kind == ImportOrExportKind::Type {
            return;
        }
        if let Some(Declaration::VariableDeclaration(v)) = &decl.declaration {
            for d in &v.declarations {
                if let (BindingPattern::BindingIdentifier(id), Some(init)) = (&d.id, &d.init) {
                    self.check_expr_as_action(id.span, id.name.as_str(), init);
                }
            }
        }
        if let Some(Declaration::FunctionDeclaration(f)) = &decl.declaration {
            if let Some(id) = &f.id {
                self.check_fn_params(id.span, id.name.as_str(), &f.params);
            }
        }
    }

    fn visit_export_default_declaration(&mut self, decl: &ExportDefaultDeclaration<'a>) {
        match &decl.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(f) => {
                let name = f
                    .id
                    .as_ref()
                    .map(|i| i.name.as_str())
                    .unwrap_or("default");
                self.check_fn_params(f.span, name, &f.params);
            }
            ExportDefaultDeclarationKind::ArrowFunctionExpression(a) => {
                self.check_fn_params(a.span, "default", &a.params);
            }
            ExportDefaultDeclarationKind::FunctionExpression(f) => {
                let name = f
                    .id
                    .as_ref()
                    .map(|i| i.name.as_str())
                    .unwrap_or("default");
                self.check_fn_params(f.span, name, &f.params);
            }
            other => {
                // export default <expr>
                if let Some(expr) = other.as_expression() {
                    self.check_expr_as_action(decl.span, "default", expr);
                }
            }
        }
    }
}

fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str) -> Vec<Finding> {
        check(Path::new("apps/demo/src/actions/x.ts"), src)
    }

    /// Fixture: legal Action signature.
    #[test]
    fn fixture_legal_pattern_passes() {
        let src = r#"
import type { Action } from '@ship/sail-runtime'
export const autofocus: Action<void> = (node) => {
  ;(node as HTMLElement).focus()
}
export const withParam: Action<string> = (node, label) => {
  node.setAttribute('aria-label', label ?? '')
  return { update(l) { node.setAttribute('aria-label', l ?? '') } }
}
"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    /// Fixture: wrong arity fires ERROR.
    #[test]
    fn fixture_signature_error() {
        let f = run("export const bad = () => {}");
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].rule, RULE_ID);
        assert!(f[0].message.contains("(node, param?)"), "{}", f[0].message);

        let f2 = run("export function worse(a, b, c) { return }");
        assert!(!f2.is_empty());
    }
}
