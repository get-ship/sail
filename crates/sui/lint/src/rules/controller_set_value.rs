//! `sui-controller-set-value` — controllers do not hand-roll
//! `setAttribute("value", …)`.
//!
//! From #177 (arcade-game agent feedback): writing
//! `el.setAttribute("value", x)` on a non-form element (`<span>`,
//! `<div>`, etc.) is silently invisible — `value` is not a rendered
//! attribute on those elements. On a form element (`<input>`,
//! `<select>`, `<textarea>`) it does set the displayed value but
//! bypasses the reactive system, so updates don't track and the
//! signal proxy never sees the write.
//!
//! Either case, the canonical SUI helper is the right call:
//! - non-form / text content → `bindText(el, () => x)`
//! - form inputs            → `bindValue(el, signal)`
//!
//! The lint is intentionally narrow — it flags the exact two-argument
//! `setAttribute("value", …)` shape only. Other `setAttribute` calls
//! (aria-*, data-*, custom attrs) stay free; SUI does not own them.

use std::path::{Path, PathBuf};

use crate::report::{Finding, Severity};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

pub const RULE_ID: &str = "sui-controller-set-value";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let ret = Parser::new(&allocator, source, source_type).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }

    let mut visitor = SetValueVisitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    visitor.findings
}

struct SetValueVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
}

impl<'src> SetValueVisitor<'src> {
    fn report(&mut self, span: Span) {
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message: "controllers do not call `setAttribute(\"value\", …)` — silent on \
                non-form elements; bypasses reactivity on form ones."
                .to_string(),
            suggest: Some(
                "For text content: `bindText(el, () => x)`. \
                 For form inputs: `bindValue(el, signal)`. \
                 Both are exported from @ship/sui-runtime."
                    .to_string(),
            ),
        });
    }
}

impl<'a, 'src> Visit<'a> for SetValueVisitor<'src> {
    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        if is_set_attribute_value(expr) {
            self.report(expr.span);
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

/// Match `<expr>.setAttribute("value", <expr>)` (exactly 2 args, first is
/// the string literal `"value"`). The check is method-call shape only —
/// it doesn't try to type-check the receiver as an Element. Authors who
/// wrap setAttribute in their own helper bypass the lint, which is fine:
/// that's an explicit choice, not the silent footgun.
fn is_set_attribute_value(expr: &CallExpression<'_>) -> bool {
    let Expression::StaticMemberExpression(m) = &expr.callee else { return false };
    if m.property.name.as_str() != "setAttribute" {
        return false;
    }
    if expr.arguments.len() != 2 {
        return false;
    }
    let Some(first) = expr.arguments.first().and_then(|a| a.as_expression()) else {
        return false;
    };
    if let Expression::StringLiteral(s) = first
        && s.value.as_str() == "value"
    {
        return true;
    }
    false
}

/// Convert a byte offset into the source string into a 1-based
/// `(line, col)` pair.
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
    fn flags_set_attribute_value_on_span() {
        let src = r#"
            export default controller<Scope>({
              mount(state, host) {
                const span = host.shadowRoot!.querySelector('span')!
                span.setAttribute('value', '42')
                return () => {}
              },
            })
        "#;
        let findings = check(Path::new("foo.controller.ts"), src);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, RULE_ID);
        assert!(findings[0].message.contains("silent on non-form"));
    }

    #[test]
    fn does_not_flag_set_attribute_aria_label() {
        let src = r#"
            export default controller<Scope>({
              mount(state, host) {
                host.setAttribute('aria-label', 'hi')
                return () => {}
              },
            })
        "#;
        assert_eq!(check(Path::new("foo.controller.ts"), src).len(), 0);
    }

    #[test]
    fn does_not_flag_set_attribute_value_with_extra_args() {
        // Wrapped helpers or chained call forms aren't matched — author
        // opted out explicitly.
        let src = r#"
            export default controller<Scope>({
              mount(state, host) {
                host.setAttribute('value', x, /* meta */ true)
                return () => {}
              },
            })
        "#;
        assert_eq!(check(Path::new("foo.controller.ts"), src).len(), 0);
    }
}
