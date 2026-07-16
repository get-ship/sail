//! `sui-controller-no-ambient-authority` — data flows through `ship`, not
//! raw network/eval.
//!
//! Flags, anywhere inside a `*.controller.ts` file, use of ambient authority:
//! - `fetch(...)`, `navigator.sendBeacon(...)`
//! - `new XMLHttpRequest()`, `new WebSocket(...)`, `new EventSource(...)`
//! - `eval(...)`, dynamic `import(...)`
//! - `document.cookie`
//!
//! Rationale (ADR unified-deck-router's sibling, the secure-execution model):
//! view code is untrusted (agent-authored) and runs on the highest-privilege
//! surface. All I/O must go through the `ship` SDK (the only sanctioned,
//! policy-gated path); ambient network/eval is exactly what the Deck SES
//! Compartment denies at runtime (proven in #165 — confined views run with
//! `fetch`/`window` undefined). This rule is the author-time mirror.
//!
//! **Severity: Error** (security), scoped to the CONFINED surface — view
//! controllers (a `views/` tree). First-party chrome component controllers
//! (`apps/shell/src/components`, `packages/components` → any `/components/`
//! path) run UNCONFINED in the host realm and legitimately use `fetch` /
//! lazy-`import()`, so they are out of scope.
//!
//! Promotion to Error was gated on every confined-surface view controller being
//! migrated off raw ambient authority — now true (zero violations across the
//! confined surface, verified by the repo-wide scan). Error hard-fails compile
//! on any regression. Note `fetch` is flagged only as a bare call or on a global
//! (`window`/`globalThis`/`self`); `ship.git.fetch({})` is a sanctioned SDK op,
//! not ambient authority.

use std::path::{Path, PathBuf};

use crate::report::{Finding, Severity};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

pub const RULE_ID: &str = "sui-controller-no-ambient-authority";

const BANNED_CALLS: &[&str] = &["fetch", "eval"];
const BANNED_CTORS: &[&str] = &["XMLHttpRequest", "WebSocket", "EventSource"];

/// The confined surface this rule governs: view controllers (a `views/` tree,
/// incl. `.ship/views/`). Chrome component controllers (`/components/`) run
/// unconfined in the host realm and legitimately use fetch/lazy-import, so they
/// are out of scope. A view's own `_components/` (underscore) stays in scope —
/// it's part of the confined view, and `/components/` does not match it.
fn is_confined_surface(path: &Path) -> bool {
    let p = path.to_string_lossy();
    p.contains("views/") && !p.contains("/components/")
}

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    // Only the confined surface is governed — unconfined chrome may use ambient
    // authority freely.
    if !is_confined_surface(path) {
        return Vec::new();
    }
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }
    let mut visitor = AmbientVisitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    visitor.findings
}

struct AmbientVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
}

impl AmbientVisitor<'_> {
    fn report(&mut self, span: Span, what: &str) {
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message: format!(
                "controllers must not use ambient authority (`{what}`). All I/O goes \
                 through the `ship` SDK — the only sanctioned, policy-gated path. Raw \
                 network/eval is denied at runtime by the Deck secure-execution Compartment."
            ),
            suggest: Some("use a ship.<domain>.<op>() call".to_string()),
        });
    }
}

impl<'a> Visit<'a> for AmbientVisitor<'_> {
    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        match &expr.callee {
            // bare `fetch(...)` / `eval(...)`
            Expression::Identifier(id) if BANNED_CALLS.contains(&id.name.as_str()) => {
                self.report(expr.span, id.name.as_str())
            }
            // `window.fetch(...)` / `navigator.sendBeacon(...)` — but ONLY when the
            // object is a global. A member `.fetch` on something else is a
            // domain method, not ambient authority — `ship.git.fetch({})` is a
            // sanctioned SDK op that happens to be named "fetch".
            Expression::StaticMemberExpression(m) => {
                let p = m.property.name.as_str();
                let on_global = matches!(
                    &m.object,
                    Expression::Identifier(o)
                        if matches!(o.name.as_str(), "window" | "globalThis" | "self" | "navigator")
                );
                if on_global && (p == "fetch" || p == "sendBeacon") {
                    self.report(expr.span, p)
                }
            }
            _ => {}
        }
        self.visit_expression(&expr.callee);
        for arg in &expr.arguments {
            if let Some(e) = arg.as_expression() {
                self.visit_expression(e)
            }
        }
    }

    fn visit_import_expression(&mut self, expr: &ImportExpression<'a>) {
        // dynamic import(...)
        self.report(expr.span, "import()");
        self.visit_expression(&expr.source);
    }

    fn visit_new_expression(&mut self, expr: &NewExpression<'a>) {
        if let Expression::Identifier(id) = &expr.callee
            && BANNED_CTORS.contains(&id.name.as_str())
        {
            self.report(expr.span, id.name.as_str());
        }
        self.visit_expression(&expr.callee);
        for arg in &expr.arguments {
            if let Some(e) = arg.as_expression() {
                self.visit_expression(e)
            }
        }
    }

    fn visit_static_member_expression(&mut self, m: &StaticMemberExpression<'a>) {
        // document.cookie
        if m.property.name.as_str() == "cookie" {
            if let Expression::Identifier(obj) = &m.object
                && obj.name.as_str() == "document"
            {
                self.report(m.span, "document.cookie");
            }
        }
        self.visit_expression(&m.object);
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

    // A confined-surface path (view controller) — where the rule applies.
    fn n(src: &str) -> usize {
        check(Path::new("views/git/controller.ts"), src).len()
    }

    #[test]
    fn flags_fetch() {
        assert_eq!(n("fetch('/api/v1/x')"), 1);
    }

    #[test]
    fn flags_new_xhr_and_websocket() {
        assert_eq!(n("const x = new XMLHttpRequest()"), 1);
        assert_eq!(n("const w = new WebSocket('wss://x')"), 1);
    }

    #[test]
    fn flags_dynamic_import_and_eval() {
        assert_eq!(n("const m = import('./x.js')"), 1);
        assert_eq!(n("eval('2+2')"), 1);
    }

    #[test]
    fn flags_document_cookie() {
        assert_eq!(n("const c = document.cookie"), 1);
    }

    #[test]
    fn allows_ship_calls() {
        assert_eq!(n("ship.git.overview()"), 0);
    }

    #[test]
    fn error_severity_on_confined_surface() {
        let f = check(Path::new("views/git/controller.ts"), "fetch('/x')");
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].severity, Severity::Error);
    }

    #[test]
    fn allows_sdk_methods_named_fetch() {
        // `ship.git.fetch({})` is a domain op, NOT ambient authority — only bare
        // `fetch(` or a fetch on a global (window/globalThis/self) is flagged.
        assert_eq!(n("const f = () => ship.git.fetch({})"), 0);
        assert_eq!(n("repo.fetch()"), 0);
        // …but the real ambient forms still fire.
        assert_eq!(n("window.fetch('/x')"), 1);
        assert_eq!(n("globalThis.fetch('/x')"), 1);
        assert_eq!(n("navigator.sendBeacon('/x')"), 1);
    }

    #[test]
    fn skips_unconfined_chrome_component_controllers() {
        // First-party chrome runs unconfined and legitimately uses fetch/import.
        let chrome = "apps/shell/src/components/sui-x/sui-x.controller.ts";
        assert_eq!(check(Path::new(chrome), "fetch('/x'); import('./y.js')").len(), 0);
        let pack = "packages/components/ui-components/src/sui-y/sui-y.controller.ts";
        assert_eq!(check(Path::new(pack), "fetch('/z')").len(), 0);
    }

    #[test]
    fn governs_a_views_private_component() {
        // `_components` (underscore) inside a view stays in scope.
        let priv_c = "views/dash/_components/sui-tile/sui-tile.controller.ts";
        assert_eq!(check(Path::new(priv_c), "fetch('/x')").len(), 1);
    }
}
