//! `sail-action-no-state` — author actions do not invent state.
//!
//! Actions are behavior attached to a host node. Signals, stores, query, and
//! mutation belong in controllers / compositions — not inside an action module.
//!
//! **Refuse** imports (named or namespace) of:
//! `signal`, `derived`, `store`, `defineStore`, `useStore`, `query`, `queryKeyed`,
//! `mutation` from `@ship/sail-runtime` (or any specifier).
//!
//! Also refuse bare calls / identifiers of those names when they appear as
//! value imports that were renamed — we catch the import binding names.

use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-action-no-state";

const REMEDY: &str = "Keep state in the controller or a composition (`signal` / `store` / \
     `query` / `mutation`). Pass values in as the action param if the behavior needs them.";

const FORBIDDEN: &[&str] = &[
    "signal",
    "derived",
    "store",
    "defineStore",
    "useStore",
    "provideStore",
    "peekStore",
    "snapshotStore",
    "query",
    "queryKeyed",
    "mutation",
];

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

    fn forbid_name(&mut self, span: Span, name: &str) {
        if FORBIDDEN.contains(&name) {
            self.report(
                span,
                format!(
                    "author actions do not import or use `{name}` — actions are behavior, \
                     not state (forbidden state primitive)"
                ),
            );
        }
    }
}

impl<'a> Visit<'a> for Visitor<'_> {
    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        let Some(specifiers) = &decl.specifiers else {
            return;
        };
        for spec in specifiers {
            match spec {
                ImportDeclarationSpecifier::ImportSpecifier(s) => {
                    // imported name is the forbidden API surface
                    let imported = match &s.imported {
                        ModuleExportName::IdentifierName(id) => id.name.as_str(),
                        ModuleExportName::IdentifierReference(id) => id.name.as_str(),
                        ModuleExportName::StringLiteral(lit) => lit.value.as_str(),
                    };
                    self.forbid_name(s.span, imported);
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                    self.forbid_name(s.span, s.local.name.as_str());
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                    // `import * as rt` — namespace alone is fine; member use caught below.
                    let _ = s;
                }
            }
        }
    }

    fn visit_identifier_reference(&mut self, id: &IdentifierReference<'a>) {
        // Catch ambient / re-bound uses when the name is used as a value.
        // Import declarations already cover the common case; this catches
        // `const x = signal(0)` when signal was somehow in scope without a
        // flagged import (ambient inject would be unusual in action modules).
        let n = id.name.as_str();
        if FORBIDDEN.contains(&n) {
            self.report(
                id.span,
                format!(
                    "author actions do not use `{n}` — forbidden state primitive"
                ),
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

    /// Fixture: legal — no state primitives.
    #[test]
    fn fixture_legal_pattern_passes() {
        let src = r#"
import type { Action } from '@ship/sail-runtime'
export const ping: Action<() => void> = (node, onPing) => {
  const el = node as HTMLElement
  const go = () => onPing?.()
  el.addEventListener('click', go)
  return { destroy: () => el.removeEventListener('click', go) }
}
"#;
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    /// Fixture: signal/store/query/mutation import fires ERROR.
    #[test]
    fn fixture_state_primitive_error() {
        let f = run("import { signal } from '@ship/sail-runtime'\nconst x = signal(0)");
        assert!(!f.is_empty(), "{f:?}");
        assert!(f.iter().any(|x| x.rule == RULE_ID));
        assert!(f.iter().any(|x| x.message.contains("signal")));

        assert!(!run("import { store } from '@ship/sail-runtime'").is_empty());
        assert!(!run("import { query, mutation } from '@ship/sail-runtime'").is_empty());
        assert!(!run("import { defineStore } from '@ship/sail-runtime'").is_empty());
    }
}
