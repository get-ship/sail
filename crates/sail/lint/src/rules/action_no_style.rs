//! `sail-action-no-style` — author action modules may not class, style, or write layout.
//!
//! **Action floor (ACTIONS-1):** ERROR. Applies to app `src/actions/**` and
//! catalog stamps (`packages/sail/runtime/src/actions/*`). Same law.
//! `sail-no-imperative-style` is the CSSOM floor (access is the crime);
//! this rule still refuses class/layout writes.
//!
//! **Refuse**
//! - any `.style` access (including `setProperty` / `removeProperty` / CSS vars)
//! - `.classList` mutation, `.className` writes
//! - `setAttribute` / `removeAttribute` for `class` | `style`
//! - layout-affecting property writes (`scrollTop` / `scrollLeft` / `cssText`)
//!
//! **Allow**
//! - listeners, focus, measure (RO/IO), DOM structure queries on the host
//! - ARIA / role attribute updates
//!
//! Path scope is decided at dispatch (`is_action_source_path`).

use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    AssignmentTarget, CallExpression, Expression, SimpleAssignmentTarget, StaticMemberExpression,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-action-no-style";

const REMEDY: &str = "Actions must not paint or lay out. Put visuals in Style / \
     `bind(signal)` / the element's `<controller>`. ARIA/data attributes and host \
     listeners are the sanctioned surface. There is no trusted catalog caste.";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }
    let mut visitor = ActionStyleVisitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    visitor.findings.sort_by_key(|f| (f.line, f.col));
    visitor.findings
}

struct ActionStyleVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
}

impl<'src> ActionStyleVisitor<'src> {
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
}

fn attr_name_is(expr: &CallExpression<'_>, names: &[&str]) -> bool {
    expr.arguments
        .first()
        .and_then(|a| a.as_expression())
        .and_then(|e| match e {
            Expression::StringLiteral(s) => Some(s.value.as_str()),
            _ => None,
        })
        .is_some_and(|v| names.contains(&v))
}

impl<'a, 'src> Visit<'a> for ActionStyleVisitor<'src> {
    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        if let Expression::StaticMemberExpression(m) = &expr.callee {
            let name = m.property.name.as_str();

            if (name == "setAttribute" || name == "removeAttribute")
                && attr_name_is(expr, &["style", "class"])
            {
                let attr = expr.arguments[0]
                    .as_expression()
                    .and_then(|e| match e {
                        Expression::StringLiteral(s) => Some(s.value.as_str()),
                        _ => None,
                    })
                    .unwrap_or("?");
                self.report(
                    expr.span,
                    format!(
                        "author actions do not `{name}('{attr}', …)` — class/style attributes \
                         reopen the paint hatch the floor closes"
                    ),
                );
            }

            if matches!(name, "add" | "remove" | "toggle" | "replace")
                && let Expression::StaticMemberExpression(class_list) = &m.object
                && class_list.property.name.as_str() == "classList"
            {
                self.report(
                    expr.span,
                    format!(
                        "author actions do not mutate `.classList.{name}()` — stamp ARIA/data \
                         and let sx/skin follow"
                    ),
                );
            }

            // Any style.* call (setProperty, removeProperty, getPropertyValue, …).
            if let Expression::StaticMemberExpression(style) = &m.object
                && style.property.name.as_str() == "style"
            {
                self.report(
                    expr.span,
                    format!(
                        "author actions do not call `.style.{name}(…)` — no style, no CSS \
                         custom-property bridge in author modules"
                    ),
                );
            }
        }

        walk::walk_call_expression(self, expr);
    }

    fn visit_static_member_expression(&mut self, m: &StaticMemberExpression<'a>) {
        let name = m.property.name.as_str();

        if name == "style" {
            self.report(
                m.span,
                "author actions do not touch `.style` — paint and layout stay in sx/tokens \
                 (framework catalog may bridge CSS vars; author modules may not)"
                    .to_string(),
            );
        }

        if name == "className" {
            self.report(
                m.span,
                "author actions do not write `.className` — a class is a global CSS target"
                    .to_string(),
            );
        }

        if name == "attributeStyleMap" || name == "cssText" {
            self.report(
                m.span,
                format!("author actions do not touch `.{name}` — CSSOM twin of the style hatch"),
            );
        }

        walk::walk_static_member_expression(self, m);
    }

    fn visit_assignment_expression(&mut self, expr: &oxc_ast::ast::AssignmentExpression<'a>) {
        if let Some(target) = assignment_member_name(&expr.left) {
            if matches!(target, "className" | "cssText" | "scrollTop" | "scrollLeft") {
                self.report(
                    expr.span,
                    format!(
                        "author actions do not assign `.{target}` — layout/paint stays out of \
                         the action body"
                    ),
                );
            }
        }
        walk::walk_assignment_expression(self, expr);
    }
}

fn assignment_member_name<'a>(target: &'a AssignmentTarget<'a>) -> Option<&'a str> {
    let simple = match target {
        AssignmentTarget::AssignmentTargetIdentifier(_) => return None,
        other => other.as_simple_assignment_target()?,
    };
    match simple {
        SimpleAssignmentTarget::StaticMemberExpression(m) => Some(m.property.name.as_str()),
        _ => None,
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
    use std::path::Path;

    fn run(src: &str) -> Vec<Finding> {
        check(Path::new("apps/demo/src/actions/x.ts"), src)
    }

    /// Fixture: legal author action — listeners + ARIA only.
    #[test]
    fn fixture_legal_pattern_passes() {
        let src = r#"
import type { Action } from '@ship/sail-runtime'
export const pressHint: Action<void> = (node) => {
  const el = node as HTMLElement
  const on = () => el.setAttribute('aria-pressed', 'true')
  el.addEventListener('click', on)
  return { destroy: () => el.removeEventListener('click', on) }
}
"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    /// Fixture: style/class/layout writes fire ERROR.
    #[test]
    fn fixture_style_class_layout_error() {
        assert!(!run("el.style.height = 'auto'").is_empty());
        assert!(!run("el.style.setProperty('--row-y', y)").is_empty());
        assert!(!run("el.classList.add('x')").is_empty());
        assert!(!run("el.setAttribute('class', 'x')").is_empty());
        assert!(!run("el.scrollTop = 0").is_empty());
        let f = run("el.style.setProperty('--x', v)");
        assert_eq!(f[0].rule, RULE_ID);
        assert!(f[0].message.contains("author actions"), "{}", f[0].message);
    }
}
