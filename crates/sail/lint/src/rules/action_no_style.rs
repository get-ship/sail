//! `sail-action-no-style` — Action bodies may not class or style.
//!
//! `use:` is a node lifetime mix-in API. Shipped registry actions and author
//! Actions share one floor with the template/sx invariants:
//!
//! **Refuse**
//! - `.classList` mutation, `.className` writes, `setAttribute('class'|…)`,
//!   `removeAttribute('class')`
//! - `.style` **except** `setProperty` / `removeProperty` on **`--*`**
//!   custom-property names (string-literal allowlist)
//! - `setAttribute('style'|…)` / `removeAttribute('style')`
//!
//! **Allow**
//! - listeners, focus, measure (RO/IO), DOM structure queries
//! - `el.style.setProperty('--x', v)` / `removeProperty('--x')` (sx bridge)
//! - ARIA / role attribute updates
//!
//! Path scope is decided at dispatch (`is_action_source_path`); this rule
//! assumes every source it sees is an Action body. Bench harness sources
//! under `tests/perf/` are exempt at dispatch (interop class=).

use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    AssignmentTarget, CallExpression, Expression, NewExpression, SimpleAssignmentTarget,
    StaticMemberExpression,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-action-no-style";

const REMEDY: &str = "Actions bridge runtime geometry through CSS custom properties \
     that sx (or a skin) reads: `el.style.setProperty('--row-y', px)` / \
     `removeProperty('--row-y')`. No class, no layout CSS property, no \
     `style=` attribute. Visual state is ARIA/data the skin keys on.";

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

/// `obj.style.setProperty` / `obj.style.removeProperty` callee shape.
fn style_method<'a>(
    expr: &'a CallExpression<'a>,
) -> Option<(&'a str, &'a StaticMemberExpression<'a>)> {
    let Expression::StaticMemberExpression(method) = &expr.callee else {
        return None;
    };
    let name = method.property.name.as_str();
    if name != "setProperty" && name != "removeProperty" {
        return None;
    }
    let Expression::StaticMemberExpression(style) = &method.object else {
        return None;
    };
    if style.property.name.as_str() != "style" {
        return None;
    }
    Some((name, style))
}

fn first_arg_custom_prop_ok(expr: &CallExpression<'_>) -> bool {
    let Some(arg) = expr.arguments.first().and_then(|a| a.as_expression()) else {
        // removeProperty() with no args is nonsense — refuse.
        return false;
    };
    match arg {
        // Literal name must be a custom property. This is the gate that
        // catches `setProperty('height', …)` / `setProperty('left', …)`.
        Expression::StringLiteral(s) => s.value.as_str().starts_with("--"),
        Expression::TemplateLiteral(t) if t.expressions.is_empty() => t
            .quasis
            .first()
            .map(|q| {
                q.value
                    .cooked
                    .as_ref()
                    .map(|c| c.as_str())
                    .unwrap_or(q.value.raw.as_str())
            })
            .is_some_and(|s| s.starts_with("--")),
        // Non-literal (`name`, `KEYS[i]`) — allow. Registry actions fold
        // known `--*` keys through a helper; refusing identifiers would
        // force unrolled literals without catching more real abuse than
        // the literal check already does.
        _ => true,
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
        .is_some_and(|v| names.iter().any(|n| *n == v))
}

impl<'a, 'src> Visit<'a> for ActionStyleVisitor<'src> {
    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        // Sanctioned bridge: el.style.setProperty('--x', …) / removeProperty('--x').
        // Walk arguments only so the intermediate `.style` member is not flagged.
        if let Some((method, _style)) = style_method(expr) {
            if first_arg_custom_prop_ok(expr) {
                for arg in &expr.arguments {
                    if let Some(e) = arg.as_expression() {
                        self.visit_expression(e);
                    }
                }
                return;
            }
            self.report(
                expr.span,
                format!(
                    "actions may only `style.{method}()` a `--*` custom property \
                     (string literal); bare CSS properties reopen layout/style."
                ),
            );
            // Still walk args; skip callee to avoid a duplicate `.style` hit.
            for arg in &expr.arguments {
                if let Some(e) = arg.as_expression() {
                    self.visit_expression(e);
                }
            }
            return;
        }

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
                        "actions do not `{name}('{attr}', …)` — class/style attributes are \
                         the same hatch the template floor closes."
                    ),
                );
            }

            // classList mutation only (reads stay observation).
            if matches!(name, "add" | "remove" | "toggle" | "replace")
                && let Expression::StaticMemberExpression(class_list) = &m.object
                && class_list.property.name.as_str() == "classList"
            {
                self.report(
                    expr.span,
                    format!(
                        "actions do not mutate `.classList.{name}()` — a class is a global \
                         CSS target; stamp ARIA/data and let the skin follow."
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
                "actions do not touch `.style` except `setProperty`/`removeProperty` on a \
                 `--*` custom property — layout CSS stays in skins/sx."
                    .to_string(),
            );
        }

        if name == "className" {
            self.report(
                m.span,
                "actions do not write `.className` — light DOM has no boundary; a class is \
                 a global CSS target."
                    .to_string(),
            );
        }

        if name == "attributeStyleMap" {
            self.report(
                m.span,
                "actions do not touch `.attributeStyleMap` — Typed OM twin of `.style`."
                    .to_string(),
            );
        }

        walk::walk_static_member_expression(self, m);
    }

    fn visit_assignment_expression(&mut self, expr: &oxc_ast::ast::AssignmentExpression<'a>) {
        // `el.className = …` / `el.style = …` via non-member paths already
        // covered by member visit; catch `el.style.left =` left-hand shape
        // with a clearer message (member visit also fires on `.style`).
        if let AssignmentTarget::AssignmentTargetIdentifier(_) = &expr.left {
            // nothing class/style-specific
        } else if let Some(target) = assignment_member_name(&expr.left) {
            if target == "className" {
                self.report(expr.span, "actions do not assign `.className`.".to_string());
            }
        }
        walk::walk_assignment_expression(self, expr);
    }

    fn visit_new_expression(&mut self, expr: &NewExpression<'a>) {
        // Not in the Part A refuse list — leave CSSStyleSheet alone here.
        walk::walk_new_expression(self, expr);
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
        check(Path::new("packages/sail/runtime/src/actions/x.ts"), src)
    }

    #[test]
    fn allows_setproperty_custom_prop() {
        assert!(run("el.style.setProperty('--row-y', y)").is_empty());
        assert!(run("el.style.removeProperty('--row-y')").is_empty());
    }

    #[test]
    fn refuses_style_property_assignment() {
        let f = run("el.style.height = 'auto'");
        assert!(!f.is_empty(), "{f:?}");
        assert!(f.iter().all(|x| x.rule == RULE_ID));
    }

    #[test]
    fn refuses_setproperty_plain_css_name() {
        let f = run("el.style.setProperty('height', '10px')");
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("custom property"), "{}", f[0].message);
    }

    #[test]
    fn allows_dynamic_custom_prop_name() {
        // Helper form used by event-span / rect-box / port-edge.
        assert!(run("el.style.setProperty(name, val)").is_empty());
        assert!(run("el.style.removeProperty(k)").is_empty());
    }

    #[test]
    fn refuses_class_and_style_attributes() {
        assert!(!run("el.setAttribute('class', 'x')").is_empty());
        assert!(!run("el.setAttribute('style', 'color:red')").is_empty());
        assert!(!run("el.removeAttribute('class')").is_empty());
    }

    #[test]
    fn refuses_class_name_and_class_list_writes() {
        assert!(!run("el.className = 'x'").is_empty());
        assert!(!run("el.classList.add('x')").is_empty());
        assert!(!run("el.classList.toggle('on')").is_empty());
    }

    #[test]
    fn allows_class_list_reads_and_aria() {
        assert!(run("el.classList.contains('dark')").is_empty());
        assert!(run("el.setAttribute('aria-pressed', 'true')").is_empty());
        assert!(run("el.setAttribute('role', 'button')").is_empty());
    }
}
