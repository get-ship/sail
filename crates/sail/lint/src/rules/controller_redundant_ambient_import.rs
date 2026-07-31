//! `sail-controller-redundant-ambient-import` — value imports of ambient
//! runtime idents inside a controller are redundant.
//!
//! The ambient vocabulary (`controller`, `signal`, `derived`, `effect`,
//! `mount`, `onCleanup`, `batch`, `untrack`) is injected by the compiler for
//! any idents a `<controller>` body references without binding them
//! (`runtime_ambient::RUNTIME_AMBIENT` + `preprocess::wrap_controller_body`).
//! An explicit `import { controller, signal } from '@ship/sail-runtime'` is
//! cargo-cult and teaches the wrong model.
//!
//! Type-only imports (`import type { ControllerScope }`, `import { type
//! Signal }`) are fine — types are not ambient-injected. Non-ambient values
//! (`query`, `mutation`, `form`, `useStore`, `filteredList`, `timeline`, …)
//! still need a real import.
//!
//! Only the **public** runtime specifier is flagged. The compiler injects
//! ambient imports from `@ship/sail-runtime/internal` during
//! `extract_controller_block` / emit; linting that post-inject source would
//! warn on every controller that uses ambient idents. Authored cargo-cult
//! always writes the public package.
//!
//! Severity: Warn (surfaced by LSP / CLI; does not hard-fail compile until
//! the teaching surface is fully clean and we promote it).

use std::path::{Path, PathBuf};

use crate::report::{Finding, Severity};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

pub const RULE_ID: &str = "sail-controller-redundant-ambient-import";

/// Authored ambient idents — mirrors `sail_compiler::runtime_ambient::RUNTIME_AMBIENT`
/// authored names (excluding the compiler-only `__invalidate`). Duplicated
/// here so `sail-lint` stays free of a `sail-compiler` dependency.
const AMBIENT_VALUE_IDENTS: &[&str] = &[
    "batch",
    "controller",
    "derived",
    "effect",
    "mount",
    "onCleanup",
    "signal",
    "untrack",
];

/// Public authoring package only. Compiler ambient inject uses
/// `@ship/sail-runtime/internal` (`runtime_ambient::RUNTIME_SPECIFIER`);
/// those injected lines must not trip this teaching rule.
const RUNTIME_SPECIFIERS: &[&str] = &["@ship/sail-runtime"];

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }

    let mut visitor = AmbientImportVisitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    visitor.findings
}

struct AmbientImportVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
}

impl AmbientImportVisitor<'_> {
    fn report(&mut self, span: Span, names: &[String]) {
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        let list = names.join(", ");
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Warn,
            message: format!(
                "redundant ambient import of `{list}` — these idents are in \
                 the controller ambient vocabulary and need no import. Keep \
                 type-only imports (`ControllerScope`, `Signal`, …) and \
                 non-ambient values (`query`, `form`, `useStore`, …)."
            ),
            suggest: Some(
                "Drop the ambient value names from the import (or the whole \
                 import if nothing non-ambient remains)."
                    .to_string(),
            ),
        });
    }
}

impl<'a> Visit<'a> for AmbientImportVisitor<'_> {
    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        // Whole-declaration `import type { … }` — every binding is type-only.
        if decl.import_kind.is_type() {
            return;
        }
        let spec = decl.source.value.as_str();
        if !RUNTIME_SPECIFIERS.contains(&spec) {
            return;
        }
        let Some(specifiers) = &decl.specifiers else {
            return;
        };
        let mut hit: Vec<String> = Vec::new();
        for sp in specifiers {
            match sp {
                ImportDeclarationSpecifier::ImportSpecifier(s) => {
                    // `import { type X }` — type-only specifier.
                    if s.import_kind.is_type() {
                        continue;
                    }
                    // imported name (not local alias) is what ambient cares about
                    let imported = match &s.imported {
                        ModuleExportName::IdentifierName(id) => id.name.as_str(),
                        ModuleExportName::IdentifierReference(id) => id.name.as_str(),
                        ModuleExportName::StringLiteral(lit) => lit.value.as_str(),
                    };
                    if AMBIENT_VALUE_IDENTS.contains(&imported) {
                        hit.push(imported.to_string());
                    }
                }
                // `import controller from '…'` — not how the runtime is consumed
                ImportDeclarationSpecifier::ImportDefaultSpecifier(_)
                | ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {}
            }
        }
        if !hit.is_empty() {
            self.report(decl.span, &hit);
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

    fn lint(src: &str) -> Vec<Finding> {
        check(Path::new("page.sail"), src)
    }

    #[test]
    fn flags_controller_signal_import() {
        let f = lint(
            "import { controller, signal } from '@ship/sail-runtime'\n\
             export default controller(() => ({ n: signal(0) }))\n",
        );
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert!(f[0].message.contains("controller"));
        assert!(f[0].message.contains("signal"));
    }

    #[test]
    fn allows_type_only() {
        let f = lint(
            "import type { ControllerScope } from '@ship/sail-runtime'\n\
             export default controller((_ctx: ControllerScope<any>) => ({ n: signal(0) }))\n",
        );
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn allows_inline_type_specifier() {
        let f = lint(
            "import { type ControllerScope, query } from '@ship/sail-runtime'\n\
             export default controller(() => ({ q: query(() => 1) }))\n",
        );
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn allows_non_ambient_values() {
        let f = lint(
            "import { query, useStore } from '@ship/sail-runtime'\n\
             export default controller(() => ({ q: query(() => 1) }))\n",
        );
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn mixed_import_flags_only_ambient() {
        let f = lint(
            "import { controller, query } from '@ship/sail-runtime'\n\
             export default controller(() => ({ q: query(() => 1) }))\n",
        );
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("`controller`"), "{f:?}");
        // Only ambient names land in the hit list — query is mentioned as a
        // keep-example in the prose, not as a flagged import.
        assert!(
            f[0].message
                .starts_with("redundant ambient import of `controller`"),
            "{f:?}"
        );
    }

    #[test]
    fn ignores_compiler_injected_internal_specifier() {
        // `extract_controller_block` / `wrap_controller_body` prepend ambient
        // imports from `@ship/sail-runtime/internal`. The rule must not treat
        // those as authored cargo-cult or every ambient-using controller warns.
        let f = lint(
            "import { controller, signal } from '@ship/sail-runtime/internal'\n\
             export default controller(() => ({ n: signal(0) }))\n",
        );
        assert!(
            f.is_empty(),
            "injected /internal ambient must not warn: {f:?}"
        );
    }
}
