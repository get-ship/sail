//! `sail-action-no-network` — author actions do not network.
//!
//! **Refuse** ambient `fetch`, `XMLHttpRequest`, `WebSocket`, `EventSource`
//! (identifier references). Network I/O belongs in `query` / `mutation`
//! (controller), not element behavior.

use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-action-no-network";

const REMEDY: &str = "Use `query()` / `mutation()` in a controller — not ambient \
     fetch/XHR/WebSocket inside an action.";

const FORBIDDEN: &[&str] = &["fetch", "XMLHttpRequest", "WebSocket", "EventSource"];

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

impl<'a> Visit<'a> for Visitor<'_> {
    fn visit_identifier_reference(&mut self, id: &IdentifierReference<'a>) {
        let n = id.name.as_str();
        if FORBIDDEN.contains(&n) {
            self.report(
                id.span,
                format!("author actions do not use ambient `{n}` (no network)"),
            );
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

    /// Fixture: legal — no network.
    #[test]
    fn fixture_legal_pattern_passes() {
        let src = r#"
export const listen: Action = (node) => {
  const on = () => {}
  node.addEventListener('click', on)
  return { destroy: () => node.removeEventListener('click', on) }
}
"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    /// Fixture: fetch/XHR/WS/EventSource fire ERROR.
    #[test]
    fn fixture_network_error() {
        let f = run("await fetch('/api')");
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].rule, RULE_ID);
        assert!(f[0].message.contains("fetch"));
        assert!(!run("new WebSocket(url)").is_empty());
        assert!(!run("new EventSource(url)").is_empty());
        assert!(!run("new XMLHttpRequest()").is_empty());
    }
}
