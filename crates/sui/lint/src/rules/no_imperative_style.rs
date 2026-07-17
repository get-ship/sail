//! `sui-no-imperative-style` — controllers do not style. FLOOR: no policy relaxes it.
//!
//! We banned CSS in the template and then wrote CSS in JavaScript. Our own
//! controllers set `.style.opacity`, `.style.translate`, `.style.transition`,
//! `.style.overflow`, `.style.cursor` — every property we tell an author to spend
//! a token for, set imperatively, untokenized, with nothing checking it. That is
//! where a blocked agent goes next, and it was the largest hole in the language.
//!
//! THE ACCESS IS THE VIOLATION, not the write. `el.style.color = 'red'` and
//! `const s = el.style` are both errors, because banning only the write leaves
//! aliasing as a trivial escape and a rule you can rename your way out of is not
//! a rule. The single sanctioned form is a CUSTOM PROPERTY:
//!
//! ```text
//! el.style.setProperty('--row-y', y)       // legal — a value, not a design
//! el.style.transform = `translateY(${y})`  // ERROR
//! ```
//!
//! Why the custom property is not a hatch: a value that comes from measurement or
//! from the user (a dragged node, a resized column) is DATA, not design. There is
//! no token for 342px and there should not be. The property itself stays in sx —
//! `sx="translate-y-(--row-y)"` — where it is auditable and tokenized; only the
//! number crosses from JS. That is the platform's own mechanism, used with the
//! grain.
//!
//! Also banned, because each is the same hole wearing a different hat:
//!   `.classList` (a class is a CSS target — see `sui-no-class`)
//!   `setAttribute('style' | 'class', …)`
//!   `adoptedStyleSheets`, `new CSSStyleSheet`, `createElement('style')`

use std::path::{Path, PathBuf};

use crate::report::{Finding, Severity};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

pub const RULE_ID: &str = "sail-no-imperative-style";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }
    let mut visitor = StyleVisitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
        sanctioned: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    visitor.findings.sort_by_key(|f| (f.line, f.col));
    visitor.findings
}

struct StyleVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
    /// Spans of `.style` accesses that ARE the sanctioned custom-property call —
    /// recorded on the way down so the member visit does not also flag them.
    sanctioned: Vec<Span>,
}

impl<'src> StyleVisitor<'src> {
    fn report(&mut self, span: Span, message: String, suggest: &str) {
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message,
            suggest: Some(suggest.to_string()),
        });
    }
}

/// `el.style.setProperty('--x', …)` — the one legal shape. The property name must
/// be a literal custom property: `setProperty(prop, …)` with a computed name is
/// refused, because a name we cannot read is a name we cannot check.
fn is_custom_property_set(expr: &CallExpression<'_>) -> Option<Span> {
    let Expression::StaticMemberExpression(call_member) = &expr.callee else {
        return None;
    };
    if call_member.property.name.as_str() != "setProperty" {
        return None;
    }
    let Expression::StaticMemberExpression(style) = &call_member.object else {
        return None;
    };
    if style.property.name.as_str() != "style" {
        return None;
    }
    let first = expr.arguments.first()?.as_expression()?;
    let Expression::StringLiteral(name) = first else {
        return None;
    };
    if !name.value.starts_with("--") {
        return None;
    }
    Some(style.span)
}

impl<'a, 'src> Visit<'a> for StyleVisitor<'src> {
    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        if let Some(style_span) = is_custom_property_set(expr) {
            self.sanctioned.push(style_span);
            // Only the arguments need walking; the callee IS the sanctioned form.
            for arg in &expr.arguments {
                if let Some(e) = arg.as_expression() {
                    self.visit_expression(e);
                }
            }
            return;
        }

        if let Expression::StaticMemberExpression(m) = &expr.callee {
            let name = m.property.name.as_str();

            // setAttribute('style'|'class', …) — the same hole, spelled differently.
            if name == "setAttribute"
                && let Some(Expression::StringLiteral(attr)) =
                    expr.arguments.first().and_then(|a| a.as_expression())
                && matches!(attr.value.as_str(), "style" | "class")
            {
                let attr = attr.value.to_string();
                self.report(
                    expr.span,
                    format!("controllers do not `setAttribute('{attr}', …)` — that is CSS with a JavaScript accent."),
                    "Style with sx. For a runtime value use `el.style.setProperty('--x', v)` and read it in sx: `translate-y-(--x)`.",
                );
            }

            // document.createElement('style')
            if name == "createElement"
                && let Some(Expression::StringLiteral(tag)) =
                    expr.arguments.first().and_then(|a| a.as_expression())
                && tag.value.as_str() == "style"
            {
                self.report(
                    expr.span,
                    "controllers do not inject a `<style>` element.".to_string(),
                    "A component's styling is sx. The framework's own sheet is generated, not injected at runtime.",
                );
            }

            // classList mutation: `el.classList.add('x')` / `.remove` /
            // `.toggle` / `.replace` — a styling hatch through the back
            // door. Reads (`.contains`, `.item`, `.length`) are allowed —
            // they surface a value the host set, not an authoring branch.
            if name == "add" || name == "remove" || name == "toggle" || name == "replace" {
                // Callee shape: `... .classList.<method>(...)` — chain
                // through two static member expressions.
                if let Expression::StaticMemberExpression(class_list) = &m.object
                    && class_list.property.name.as_str() == "classList"
                {
                    self.report(
                        expr.span,
                        format!(
                            "controllers do not mutate `.classList.{name}()` — light DOM                              has no boundary, so a class is a global CSS target."
                        ),
                        "Visual state is sx keyed on ARIA (`selected:`, `expanded:`, `disabled:`).                          Stamp the semantics and the style follows. Reads                          (`.contains`, `.item`, `.length`) are observation, not an                          authoring hatch.",
                    );
                }
            }
        }

        self.visit_expression(&expr.callee);
        for arg in &expr.arguments {
            if let Some(e) = arg.as_expression() {
                self.visit_expression(e);
            }
        }
    }

    fn visit_static_member_expression(&mut self, m: &StaticMemberExpression<'a>) {
        let name = m.property.name.as_str();

        // THE ACCESS is the violation. `const s = el.style` must fail, or the ban
        // is one rename deep.
        if name == "style" && !self.sanctioned.contains(&m.span) {
            self.report(
                m.span,
                "controllers do not touch `.style` — a CSS property set from JavaScript \
                 is untokenized CSS that nothing can check."
                    .to_string(),
                "Style with sx. A value that comes from measurement or from the user is DATA, \
                 not design: set it as a custom property — `el.style.setProperty('--row-y', y)` \
                 — and read it in sx: `sx=\"translate-y-(--row-y)\"`. The property stays where it \
                 can be audited; only the number crosses.",
            );
        }

        // `.classList` reads (e.g. `document.documentElement.classList.contains("dark")`
        // to detect the host theme) are observation — they surface a value
        // the host already set, which is the seam the language respects.
        // WRITES to classList (`add`/`remove`/`toggle`/`replace`) are still
        // banned: they are a styling hatch through the back door. The exact
        // match is in `visit_call_expression` below; the `classList`
        // member access itself is not flagged here.

        if name == "adoptedStyleSheets" {
            self.report(
                m.span,
                "controllers do not assign `adoptedStyleSheets`.".to_string(),
                "A component's styling is sx; the framework's sheet is generated at build.",
            );
        }

        oxc_ast_visit::walk::walk_static_member_expression(self, m);
    }

    fn visit_new_expression(&mut self, expr: &NewExpression<'a>) {
        if let Expression::Identifier(id) = &expr.callee
            && id.name.as_str() == "CSSStyleSheet"
        {
            self.report(
                expr.span,
                "controllers do not construct a `CSSStyleSheet`.".to_string(),
                "A component's styling is sx; the framework's sheet is generated at build.",
            );
        }
        oxc_ast_visit::walk::walk_new_expression(self, expr);
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

    fn run(src: &str) -> Vec<String> {
        check(Path::new("t.controller.ts"), src).into_iter().map(|f| f.message).collect()
    }

    #[test]
    fn a_css_property_written_from_js_is_untokenized_css() {
        assert_eq!(run("el.style.opacity = '0.5'").len(), 1);
        assert_eq!(run("row.style.transform = `translateY(${y}px)`").len(), 1);
        assert_eq!(run("host.style.setProperty('padding', '13px')").len(), 1);
    }

    #[test]
    fn the_access_is_the_violation_so_aliasing_cannot_escape() {
        // A ban you can rename your way out of is not a ban.
        assert_eq!(run("const s = el.style; s.color = 'red'").len(), 1);
        assert_eq!(run("Object.assign(el.style, { color: 'red' })").len(), 1);
    }

    #[test]
    fn a_custom_property_is_data_not_design_and_is_the_one_way_through() {
        assert!(run("el.style.setProperty('--row-y', y)").is_empty());
        assert!(run("host.style.setProperty('--sui-grid-template', tracks)").is_empty());
        // …but only a CUSTOM property. A CSS property through the same call is the
        // same hole.
        assert_eq!(run("el.style.setProperty('transform', t)").len(), 1);
        // A name we cannot read is a name we cannot check.
        assert_eq!(run("el.style.setProperty(prop, v)").len(), 1);
    }

    #[test]
    fn the_other_hats_the_same_hole_wears() {
        // `.classList.add/remove/toggle/replace` — the styling hatch.
        assert_eq!(run("el.classList.add('active')").len(), 1);
        assert_eq!(run("el.classList.remove('active')").len(), 1);
        assert_eq!(run("el.classList.toggle('active')").len(), 1);
        assert_eq!(run("el.classList.replace('a', 'b')").len(), 1);
        assert_eq!(run("el.setAttribute('style', 'color:red')").len(), 1);
        assert_eq!(run("el.setAttribute('class', 'active')").len(), 1);
        assert_eq!(run("root.adoptedStyleSheets = [sheet]").len(), 1);
        assert_eq!(run("const s = new CSSStyleSheet()").len(), 1);
        assert_eq!(run("document.createElement('style')").len(), 1);
    }

    #[test]
    fn classlist_reads_are_observation_not_an_authoring_hatch() {
        // `el.classList.contains` / `.item` / `.length` — READING a class
        // is observation, not a hatch. (The `sui-no-class` rule prevents the
        // author from writing `class=` from inside a component; the seams
        // are read here on purpose.)
        assert!(run("if (el.classList.contains('active')) doThing()").is_empty());
        assert!(run("const x = el.classList.item(0)").is_empty());
        assert!(run("const n = el.classList.length").is_empty());
    }

    #[test]
    fn a_controller_that_does_not_style_is_clean() {
        assert!(run("const n = signal(0); el.addEventListener('click', () => n.set(1))").is_empty());
        assert!(run("el.setAttribute('aria-expanded', 'true')").is_empty());
        assert!(run("document.createElement('div')").is_empty());
    }
}
