//! `sail-no-ambient-dom` — app sources do not reach into the ambient document.
//!
//! Controllers and app modules under `apps/*/src/**` must not invent a parallel
//! DOM API: `document.createElement`, `document.querySelector` / `querySelectorAll`,
//! and other `document.*` members used to mutate hosts or inject paint. Host
//! properties bind through Sail (`prop:tree={table}`, signals, sx) — not by
//! fishing elements out of the ambient tree and writing them.
//!
//! **Allowlist (narrow):** static `document.querySelector('meta[name=…]')` for
//! shell bootstrap (project root / deck app route). The framework injects those
//! meta tags; there is no alternate authoring API yet. Any other `document.*`
//! access is an Error.
//!
//! Pair with `sail-no-imperative-style` on the same surface — that rule bans
//! `.style` / `cssText` / classList writes; this rule closes the "side module
//! does `createElement` + `querySelector` to paint" escape hatch.

use std::path::{Path, PathBuf};

use crate::report::{Finding, Severity};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

pub const RULE_ID: &str = "sail-no-ambient-dom";

const REMEDY: &str = "Bind host state through Sail (prop:/signals/sx), not ambient document \
     lookup. Shell bootstrap may read `document.querySelector('meta[name=…]')` only. \
     Measurement-driven geometry belongs in a `use:` action in the runtime package.";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    // SPEC-DOMDEC-1 path gate (apps only). Controllers historically always ran
    // this rule; keep controllers in-scope via .controller.ts under apps or
    // the ambient gate for apps/*/src|routes.
    use super::ambient::{is_ambient_app_path, is_sail_allowed};
    let p = path.to_string_lossy();
    let is_controller = p.ends_with(".controller.ts");
    if !is_ambient_app_path(path) && !is_controller {
        return Vec::new();
    }
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }
    let mut visitor = AmbientDomVisitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    // Apply sail-allow after visit.
    visitor.findings.retain(|f| !is_sail_allowed(source, f.line, RULE_ID));
    visitor.findings.sort_by_key(|f| (f.line, f.col));
    visitor.findings
}

struct AmbientDomVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
}

impl AmbientDomVisitor<'_> {
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

/// `document` or `window.document` / `globalThis.document`.
fn is_document_expr(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::Identifier(id) => id.name.as_str() == "document",
        Expression::StaticMemberExpression(m)
            if m.property.name.as_str() == "document" =>
        {
            matches!(
                &m.object,
                Expression::Identifier(id)
                    if matches!(id.name.as_str(), "window" | "globalThis" | "self")
            )
        }
        _ => false,
    }
}

/// Static selector that only reads a shell bootstrap meta tag.
fn is_meta_bootstrap_selector(sel: &str) -> bool {
    let t = sel.trim();
    // Accept both quote styles: meta[name="x"] / meta[name='x']
    t.starts_with("meta[name=") || t.starts_with("meta[name =")
}

impl<'a> Visit<'a> for AmbientDomVisitor<'_> {
    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        if let Expression::StaticMemberExpression(m) = &expr.callee
            && is_document_expr(&m.object)
        {
            let name = m.property.name.as_str();
            match name {
                "createElement" | "createElementNS" | "createDocumentFragment" => {
                    self.report(
                        expr.span,
                        format!(
                            "app sources do not call `document.{name}(...)` — inventing DOM \
                             nodes outside the template is the side-module paint hatch."
                        ),
                    );
                }
                "querySelector" | "querySelectorAll" => {
                    let allowed = expr
                        .arguments
                        .first()
                        .and_then(|a| a.as_expression())
                        .and_then(|e| match e {
                            Expression::StringLiteral(s) => Some(s.value.as_str()),
                            _ => None,
                        })
                        .is_some_and(is_meta_bootstrap_selector);
                    if !allowed {
                        self.report(
                            expr.span,
                            format!(
                                "app sources do not call `document.{name}(...)` to find hosts — \
                                 bind identity through prop:/signals, not ambient lookup \
                                 (meta[name=…] bootstrap reads are the only exception)."
                            ),
                        );
                    }
                }
                "getElementById" | "getElementsByClassName" | "getElementsByTagName"
                | "getElementsByName" | "getElementsByTagNameNS" => {
                    self.report(
                        expr.span,
                        format!(
                            "app sources do not call `document.{name}(...)` — ambient document \
                             lookup is the host-mutation hatch."
                        ),
                    );
                }
                _ => {
                    // Any other `document.foo(...)` call (write / adopt / execCommand …).
                    self.report(
                        expr.span,
                        format!(
                            "app sources do not call `document.{name}(...)` — keep document \
                             access out of app modules."
                        ),
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
        // `document.foo` access (not a call) — THE ACCESS is the violation so
        // aliasing (`const d = document.body`) cannot escape. Calls are reported
        // in visit_call_expression; here we still flag the member so
        // `document.documentElement` / `.body` / `.styleSheets` refuse even
        // without a following call. Skip when parent is a call we already
        // handle? Visit walks both — call reports on CallExpression span,
        // member reports on property access. Double-report on
        // `document.querySelector(...)` would fire member + call. Prefer
        // reporting only non-method member access here for root/document
        // properties, not methods that will be call-visited.
        if is_document_expr(&m.object) {
            let name = m.property.name.as_str();
            // Method names are reported at the call site with a better message.
            let methodish = matches!(
                name,
                "createElement"
                    | "createElementNS"
                    | "createDocumentFragment"
                    | "querySelector"
                    | "querySelectorAll"
                    | "getElementById"
                    | "getElementsByClassName"
                    | "getElementsByTagName"
                    | "getElementsByName"
                    | "getElementsByTagNameNS"
                    | "addEventListener"
                    | "removeEventListener"
                    | "dispatchEvent"
                    | "write"
                    | "writeln"
                    | "open"
                    | "close"
                    | "execCommand"
                    | "adoptNode"
                    | "importNode"
            );
            if !methodish {
                self.report(
                    m.span,
                    format!(
                        "app sources do not touch `document.{name}` — ambient document access \
                         is the side-module hatch (meta bootstrap querySelector is the only \
                         narrow exception)."
                    ),
                );
            }
        }
        oxc_ast_visit::walk::walk_static_member_expression(self, m);
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
        check(Path::new("apps/studio/src/foo.ts"), src)
            .into_iter()
            .map(|f| f.message)
            .collect()
    }

    #[test]
    fn create_element_any_tag_errors() {
        assert_eq!(run("document.createElement('div')").len(), 1);
        assert_eq!(run("document.createElement('style')").len(), 1);
        assert_eq!(run("document.createElement(tag)").len(), 1);
    }

    #[test]
    fn query_selector_errors_except_meta_bootstrap() {
        assert_eq!(run("document.querySelector('[data-x]')").len(), 1);
        assert_eq!(run("document.querySelectorAll('[data-x]')").len(), 1);
        assert_eq!(run("document.querySelector(sel)").len(), 1);
        // Bootstrap meta — allowed.
        assert!(
            run(r#"document.querySelector('meta[name="sail-project"]')"#).is_empty(),
            "meta bootstrap must be allowed"
        );
        assert!(
            run(r#"document.querySelector('meta[name=\"deck-app-route\"]')"#).is_empty()
                || run(r#"document.querySelector('meta[name="deck-app-route"]')"#).is_empty()
        );
    }

    #[test]
    fn document_root_member_access_errors() {
        assert_eq!(run("document.documentElement.setAttribute('data-theme', 'x')").len(), 1);
        assert_eq!(run("const b = document.body").len(), 1);
    }

    #[test]
    fn typeof_document_guard_is_clean() {
        assert!(run("if (typeof document === 'undefined') return").is_empty());
    }

    #[test]
    fn plain_studio_module_with_csstext_path_is_caught_by_sibling_rule() {
        // Dispatch-level coverage lives in lib tests; here we only own ambient DOM.
        assert_eq!(run("el.style.cssText = 'x'").len(), 0);
    }
}
