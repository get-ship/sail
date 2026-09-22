//! `sail-action-host-only` — author actions receive a host node; no god-reach.
//!
//! **Refuse**
//! - bare `document` / `window` / `globalThis` / `self` identifiers used as values
//! - `document.*` / `window.*` member access
//!
//! The action is handed its host. Reach into the ambient tree or the global
//! object is the cross-node / god-object hatch. Framework catalog actions that
//! need document (hotkeys, icon fetch) stay trusted and are not linted here.

use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-action-host-only";

const REMEDY: &str = "Use only the handed host node (and nodes reached from it: \
     `node.querySelector`, listeners on `node`). Ambient `document` / `window` \
     god-reach is forbidden in author actions.";

const GOD: &[&str] = &["document", "window", "globalThis", "self"];

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
}

fn is_god_ident(expr: &Expression<'_>) -> Option<&'static str> {
    match expr {
        Expression::Identifier(id) => {
            let n = id.name.as_str();
            GOD.iter().copied().find(|&g| g == n)
        }
        _ => None,
    }
}

impl<'a> Visit<'a> for Visitor<'_> {
    fn visit_identifier_reference(&mut self, id: &IdentifierReference<'a>) {
        let n = id.name.as_str();
        if GOD.contains(&n) {
            self.report(
                id.span,
                format!(
                    "author actions do not reach ambient `{n}` — handed host only \
                     (no document/window god-reach)"
                ),
            );
        }
    }

    fn visit_static_member_expression(&mut self, m: &StaticMemberExpression<'a>) {
        // Redundant with identifier visit on `document.foo` (identifier fires
        // on `document`), but give a clearer message for member form when the
        // object is god.
        if let Some(god) = is_god_ident(&m.object) {
            self.report(
                m.span,
                format!(
                    "author actions do not access `{god}.{}` — handed host only",
                    m.property.name.as_str()
                ),
            );
            // Still walk into property args? skip object re-walk by not
            // descending into object — walk children of property only via default.
            return;
        }
        walk::walk_static_member_expression(self, m);
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

    /// Fixture: legal — host-only queries.
    #[test]
    fn fixture_legal_pattern_passes() {
        let src = r#"
export const wire: Action = (node) => {
  const root = node as HTMLElement
  const child = root.querySelector('[data-slot]')
  child?.addEventListener('click', () => {})
}
"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    /// Fixture: document/window god-reach fires ERROR.
    #[test]
    fn fixture_god_reach_error() {
        let f = run("document.querySelector('body')");
        assert!(!f.is_empty(), "{f:?}");
        assert!(f.iter().any(|x| x.rule == RULE_ID));
        assert!(f.iter().any(|x| x.message.contains("document")));
        assert!(!run("window.addEventListener('resize', () => {})").is_empty());
        assert!(!run("globalThis.document").is_empty());
    }
}
