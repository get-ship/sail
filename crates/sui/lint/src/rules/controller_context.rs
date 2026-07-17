//! `sui-controller-context` — read host facts from the seam, not `window`.
//!
//! Bans, anywhere inside a `*.controller.ts` file, reads of the ambient
//! host-context globals:
//! - `window.__shipProjectId` / `__shipProjectId`
//! - `window.__shipSessionId` / `__shipSessionId`
//! - `window.__shipMode`       / `__shipMode`
//!
//! These were a leak — host facts stashed on `window` and fished out
//! ad-hoc. The runtime exposes them through the `HostContext` seam
//! (`hostContext()` from `@ship/sail-runtime`); the daemon mode is on the
//! `<meta name="ship-mode">` tag. Reaching for the globals re-creates the
//! leak the seam exists to close.
//!
//! Not flagged: `window.ship` (the intentional public shell API) and
//! `window.__shipCanvas` (a deliberate e2e/QA probe handle).

use std::path::{Path, PathBuf};

use crate::report::{Finding, Severity};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

pub const RULE_ID: &str = "sail-controller-context";

const BANNED: &[&str] = &["__shipProjectId", "__shipSessionId", "__shipMode"];

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }
    let mut visitor = ContextVisitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    visitor.findings
}

struct ContextVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
}

impl ContextVisitor<'_> {
    fn report(&mut self, span: Span, prop: &str) {
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message: format!(
                "controllers must not read host facts from `window.{prop}`. Use the \
                 HostContext seam — `hostContext()` from @ship/sail-runtime (or the \
                 `<meta name=\"ship-mode\">` tag for daemon mode)."
            ),
            suggest: Some(
                "import { hostContext } from '@ship/sail-runtime'  // hostContext().projectId / .sessionId"
                    .to_string(),
            ),
        });
    }
}

impl<'a> Visit<'a> for ContextVisitor<'_> {
    fn visit_static_member_expression(&mut self, m: &StaticMemberExpression<'a>) {
        // `window.__shipX` / `globalThis.__shipX` (and bare via the object).
        let prop = m.property.name.as_str();
        if BANNED.contains(&prop) {
            self.report(m.span, prop);
        }
        self.visit_expression(&m.object);
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        // Bare `__shipX` reference (no `window.` prefix).
        if BANNED.contains(&ident.name.as_str()) {
            self.report(ident.span, ident.name.as_str());
        }
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

    #[test]
    fn flags_window_ship_project_id() {
        let f = check(Path::new("x.controller.ts"), "const p = window.__shipProjectId");
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].rule, "sail-controller-context");
    }

    #[test]
    fn flags_bare_ship_mode() {
        let f = check(Path::new("x.controller.ts"), "if (__shipMode === 'Development') {}");
        assert_eq!(f.len(), 1);
    }

    #[test]
    fn allows_window_ship_api() {
        // The intentional public shell API is fine.
        let f = check(Path::new("x.controller.ts"), "window.ship.navigate('/x')");
        assert!(f.is_empty());
    }

    #[test]
    fn allows_canvas_probe() {
        // The deliberate e2e probe handle is not host context.
        let f = check(Path::new("x.controller.ts"), "window.__shipCanvas = {}");
        assert!(f.is_empty());
    }

    #[test]
    fn allows_host_context_seam() {
        let f = check(Path::new("x.controller.ts"), "const p = hostContext().projectId");
        assert!(f.is_empty());
    }
}
