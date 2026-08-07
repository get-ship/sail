//! `sail-controller-import-convention` — controllers import only from
//! conventional dirs + runtime/stdlib packages (SAIL-SYNTAX-FREEZE §1 / §13).
//!
//! Allowed value-import sources:
//! - `@ship/*` packages (runtime, tokens, deck, studio-shell, …)
//! - relative paths that resolve under `src/actions/`, `src/compositions/`,
//!   or `src/serverfns/` (ACTIONS-1 + composition + D4 conventions)
//! - relative `.sail` modules (Scope / type surface of a partial)
//!
//! Type-only imports (`import type`, `import { type X }`) are free — they
//! carry no runtime coupling.
//!
//! Severity: **Error** (freeze: arbitrary module paths are a compile error
//! naming the convention).

use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-controller-import-convention";

const MESSAGE: &str = "controller import is outside the freeze convention — value \
     imports must come from `@ship/*` packages, relative `.sail` modules, or \
     `src/actions/` · `src/compositions/` · `src/serverfns/`. Type-only imports \
     (`import type`) are free anywhere.";

const SUGGEST: &str = "Move the module under `src/actions/`, `src/compositions/`, \
     or `src/serverfns/`, import it from an `@ship/*` package, or switch to \
     `import type` if only types are needed.";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }

    let mut visitor = ImportVisitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    visitor.findings
}

struct ImportVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
}

impl ImportVisitor<'_> {
    fn report(&mut self, span: Span, module: &str) {
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message: format!("{MESSAGE} (got `{module}`)"),
            suggest: Some(SUGGEST.to_string()),
        });
    }
}

impl<'a> Visit<'a> for ImportVisitor<'_> {
    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        // Whole-declaration `import type` — free.
        if decl.import_kind.is_type() {
            return;
        }
        let module = decl.source.value.as_str();

        // Side-effect import with no bindings — refuse (no type-only escape).
        let Some(specifiers) = &decl.specifiers else {
            self.report(decl.span, module);
            return;
        };

        // If every named specifier is type-only, the import is free.
        let mut any_value = false;
        for sp in specifiers {
            match sp {
                ImportDeclarationSpecifier::ImportSpecifier(s) => {
                    if !s.import_kind.is_type() {
                        any_value = true;
                        break;
                    }
                }
                // default / namespace are always value bindings
                ImportDeclarationSpecifier::ImportDefaultSpecifier(_)
                | ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                    any_value = true;
                    break;
                }
            }
        }
        if !any_value {
            return;
        }

        if is_allowed_value_module(module, &self.file) {
            return;
        }
        self.report(decl.span, module);
    }
}

fn is_allowed_value_module(module: &str, controller_path: &Path) -> bool {
    let m = module.trim();
    if m.is_empty() {
        return false;
    }

    // Workspace / product packages.
    if m.starts_with("@ship/") {
        return true;
    }

    // Relative or absolute file path.
    if m.starts_with('.') || m.starts_with('/') {
        // .sail partials (Scope types often imported as value-less; if value,
        // still the language surface for partials).
        if m.ends_with(".sail") {
            return true;
        }

        // Resolve against the controller file's directory when possible.
        let resolved = if m.starts_with('.') {
            controller_path
                .parent()
                .map(|dir| normalize_join(dir, m))
                .unwrap_or_else(|| PathBuf::from(m))
        } else {
            PathBuf::from(m)
        };
        let resolved_str = resolved.to_string_lossy().replace('\\', "/");
        if is_conventional_dir(&resolved_str) {
            return true;
        }
        // Also accept the bare specifier form (tests pass synthetic paths).
        if is_conventional_dir(m) {
            return true;
        }
        return false;
    }

    // Bare package names outside @ship/* are arbitrary third-party imports.
    false
}

fn is_conventional_dir(p: &str) -> bool {
    p.contains("/src/actions/")
        || p.contains("/src/compositions/")
        || p.contains("/src/serverfns/")
        || p.starts_with("src/actions/")
        || p.starts_with("src/compositions/")
        || p.starts_with("src/serverfns/")
        || p.contains("/actions/") && p.contains("/src/")
        // path segment exact: …/src/actions
        || p.ends_with("/src/actions")
        || p.ends_with("/src/compositions")
        || p.ends_with("/src/serverfns")
}

fn normalize_join(base: &Path, rel: &str) -> PathBuf {
    let mut out = base.to_path_buf();
    for part in rel.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                let _ = out.pop();
            }
            p => out.push(p),
        }
    }
    out
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

    fn lint_at(path: &str, src: &str) -> Vec<Finding> {
        check(Path::new(path), src)
    }

    #[test]
    fn allows_ship_package() {
        let f = lint_at(
            "apps/demo/src/page.controller.ts",
            "import { query } from '@ship/sail-runtime'\n\
             export default controller(() => ({ q: query(() => 1) }))\n",
        );
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn allows_type_only_anywhere() {
        let f = lint_at(
            "apps/demo/src/page.controller.ts",
            "import type { Foo } from '../helpers/foo'\n\
             import { type Bar } from './bar-types'\n\
             export default controller(() => ({}))\n",
        );
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn allows_actions_dir() {
        let f = lint_at(
            "apps/demo/src/routes/page.controller.ts",
            "import { autofocus } from '../actions/autofocus'\n\
             export default controller(() => ({}))\n",
        );
        // ../actions from routes → src/actions
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn allows_compositions_dir() {
        let f = lint_at(
            "apps/demo/src/page.controller.ts",
            "import { listModel } from './compositions/list-model'\n\
             export default controller(() => ({}))\n",
        );
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn allows_relative_sail() {
        let f = lint_at(
            "apps/demo/src/page.controller.ts",
            "import type { Scope } from './page.sail'\n\
             export default controller(() => ({}))\n",
        );
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn refuses_arbitrary_relative() {
        let f = lint_at(
            "apps/demo/src/page.controller.ts",
            "import { createVm } from './page-vm'\n\
             export default controller(() => ({}))\n",
        );
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert_eq!(f[0].severity, Severity::Error);
        assert!(f[0].message.contains("page-vm"), "{}", f[0].message);
    }

    #[test]
    fn refuses_third_party_package() {
        let f = lint_at(
            "apps/demo/src/page.controller.ts",
            "import lodash from 'lodash'\n\
             export default controller(() => ({}))\n",
        );
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("lodash"), "{}", f[0].message);
    }

    #[test]
    fn refuses_side_effect_import() {
        let f = lint_at(
            "apps/demo/src/page.controller.ts",
            "import './polyfill'\n\
             export default controller(() => ({}))\n",
        );
        assert_eq!(f.len(), 1, "{f:?}");
    }
}
