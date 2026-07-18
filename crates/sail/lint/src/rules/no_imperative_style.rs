//! `sail-no-imperative-style` — controllers do not style. FLOOR: no policy relaxes it.
//!
//! We banned CSS in the template and then wrote CSS in JavaScript. Our own
//! controllers set `.style.opacity`, `.style.translate`, `.style.transition`,
//! `.style.overflow`, `.style.cursor` — every property we tell an author to spend
//! a token for, set imperatively, untokenized, with nothing checking it. That is
//! where a blocked agent goes next, and it was the largest hole in the language.
//!
//! THE ACCESS IS THE VIOLATION, not the write. `el.style.color = 'red'`,
//! `const s = el.style`, and `el.style.setProperty('--x', v)` are ALL errors in a
//! controller. `.setProperty('--x')` was once the sanctioned hatch here; the owner
//! ruling of 2026-07-18 CLOSED it — a custom property set from a controller is
//! still CSS written in JavaScript, and the `(--x)`-declared reflection
//! (inv-dynamic-style-is-declared, 2026-07-17) exists precisely so a controller
//! never has to touch `.style` to move a runtime value into CSS. Banning only the
//! write leaves aliasing as a trivial escape; a rule you can rename your way out
//! of is not a rule, so the ACCESS is what fails.
//!
//! Where a runtime value goes instead — three intents, none through `.style`:
//!
//!   1. A value you own or compute → declare a signal, return it, read `(--x)`:
//!      `const rowY = signal(0)` … `return { rowY }` … `sx="translate-y-(--row-y)"`.
//!      The compiler reflects the signal onto the host as an inline custom
//!      property; no `.style`.
//!   2. Measurement-driven geometry (a dragged node, a measured column) → a
//!      `use:` action. The action OWNS its custom property and is the sanctioned
//!      home for `el.style.setProperty`; actions live in the framework runtime
//!      (`packages/sail/runtime/src/**`), which this rule never lints.
//!   3. A design token (theming) → tokens live in `tokens.css`; in dev, edit the
//!      file and HMR re-themes. A live per-subtree override is a signal read as
//!      `(--x)` on a wrapper — the inline custom property inherits to the subtree.
//!
//! Also banned, because each is the same hole wearing a different hat:
//!   `.classList` writes (a class is a CSS target — see `sail-no-class`)
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
    };
    visitor.visit_program(&ret.program);
    visitor.findings.sort_by_key(|f| (f.line, f.col));
    visitor.findings
}

struct StyleVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
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

impl<'a, 'src> Visit<'a> for StyleVisitor<'src> {
    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        // `el.style.setProperty('--x', …)` is NOT exempt here. This rule only
        // ever runs on controller / pack sources (the framework runtime, where
        // `use:` actions live, is exempt at the dispatch level); inside those,
        // a custom-property write is still `.style` from JavaScript. The
        // `.style` member access below catches it — no special-case needed.
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
                    "Style with sx. For a runtime value, declare a signal and read it in sx (`translate-y-(--row-y)`), or use a `use:` action for measurement-driven geometry — never `.style`.",
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
        // is one rename deep. `.setProperty('--x')` is included: it is `.style`
        // from JavaScript like any other, closed by the 2026-07-18 ruling.
        if name == "style" {
            self.report(
                m.span,
                "controllers do not touch `.style` — a CSS property set from JavaScript \
                 is untokenized CSS that nothing can check (`.setProperty('--x', v)` \
                 included: a custom property set from a controller is still CSS in JS)."
                    .to_string(),
                "Move the value into CSS without `.style` — three intents: \
                 (1) a value you own or compute — declare a signal, return it, read it in sx: \
                 `const rowY = signal(0)` … `return { rowY }` … `sx=\"translate-y-(--row-y)\"` \
                 (the compiler reflects it onto the host). \
                 (2) measurement-driven geometry (a dragged node, a measured column) — a `use:` \
                 action owns the property: `<node use:gridTracks={tracks}>`. \
                 (3) a design TOKEN (theming) — tokens live in tokens.css (edit the file; HMR \
                 re-themes), or for a live per-subtree override declare a signal and read `(--x)` \
                 on a wrapper (the inline custom property inherits to the subtree).",
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
    fn setproperty_of_a_custom_property_is_now_closed_in_a_controller() {
        // The 2026-07-18 ruling: `.setProperty('--x')` is `.style` from
        // JavaScript like any other, and is CLOSED in controller/pack source.
        // (It was legal here before; this is the red/green gate for the closure.)
        assert_eq!(run("el.style.setProperty('--row-y', y)").len(), 1);
        assert_eq!(run("host.style.setProperty('--sail-grid-template', tracks)").len(), 1);
        // A CSS property through the same call was always the hole; still is.
        assert_eq!(run("el.style.setProperty('transform', t)").len(), 1);
        // A name we cannot read is a name we cannot check.
        assert_eq!(run("el.style.setProperty(prop, v)").len(), 1);
    }

    #[test]
    fn the_rewritten_diagnostic_teaches_the_declared_paths_not_the_hatch() {
        // The message must NOT teach `el.style.setProperty('--x', …)` — that was
        // the hatch the ruling closed. It must point at the declared signal /
        // `(--x)` reflection, a `use:` action, and tokens.css.
        let findings = check(Path::new("t.controller.ts"), "el.style.opacity = '0'");
        assert_eq!(findings.len(), 1);
        let suggest = findings[0].suggest.as_deref().unwrap_or("");
        assert!(
            !suggest.contains(".style.setProperty"),
            "the remedy must not send the author back to el.style.setProperty — got: {suggest}"
        );
        assert!(suggest.contains("signal("), "remedy must name the declared signal path: {suggest}");
        assert!(suggest.contains("use:"), "remedy must name the use: action path: {suggest}");
        assert!(suggest.contains("tokens.css"), "remedy must name the token path: {suggest}");
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
        // is observation, not a hatch. (The `sail-no-class` rule prevents the
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
