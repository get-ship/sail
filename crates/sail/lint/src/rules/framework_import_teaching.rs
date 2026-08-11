//! `sail-framework-import-teaching` — value imports from `@ship/*` in **app**
//! and **third-party pack** code are a teaching Error (SPEC-AMBIENT-SURFACE-1 §2).
//!
//! Law (operator-ratified; S5.3 supersedes the S5.1 L2 pack carve-out):
//! - Framework functions are ambient — authors delete `@ship/*` value imports.
//! - Fires on app controllers (`.controller.ts` / inline controller under
//!   `apps/**`), plain app TypeScript under `apps/*/src/**` (via
//!   `APP_SOURCE_RULES`), and pack controllers under `packages/components/**`
//!   (packs are third-party, not framework).
//! - Type-only imports (`import type`, `import { type X }`) remain free.
//! - App-local `@/` imports are not `@ship/*` and never fire this rule.
//! - Compiler-injected ambient imports from `@ship/sail-runtime/internal`
//!   are exempt — `extract_controller_block` prepends them before the purity
//!   hard-wall lints the body; flagging them would Error every ambient use.
//!
//! Severity: Error (fails `compile` hard-wall + `check_all`).

use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-framework-import-teaching";

/// SPEC teaching shape: names ambient + delete/framework + import.
const MESSAGE: &str =
    "framework functions are ambient — delete this import from `@ship/*`";

const SUGGEST: &str =
    "Drop the `@ship/*` value import. Ambient names (signal, controller, \
     query, store ops, theme, …) need no import; app-local modules use `@/`.";

/// Compiler ambient inject specifier (`runtime_ambient::RUNTIME_SPECIFIER`).
/// Post-inject controller bodies always carry this line; never treat it as
/// authored cargo-cult.
const COMPILER_INJECT_SPECIFIER: &str = "@ship/sail-runtime/internal";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    if !is_app_framework_import_scope(path) {
        return Vec::new();
    }

    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }

    let mut visitor = FrameworkImportVisitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    visitor.findings
}

/// App + third-party pack controllers — framework packages stay out of scope.
fn is_app_framework_import_scope(path: &Path) -> bool {
    let p = path.to_string_lossy().replace('\\', "/");
    // S5.3: L2 packs are third-party — same import law as apps (no carve-out).
    if p.contains("/packages/components/") || p.starts_with("packages/components/") {
        return true;
    }
    // Framework / other packages (runtime, tokens sources, deck, …).
    if p.contains("/packages/") || p.starts_with("packages/") {
        return false;
    }
    if p.contains("/crates/") || p.starts_with("crates/") {
        return false;
    }
    // App surfaces: controllers, routes, src under apps/*.
    p.contains("/apps/") || p.starts_with("apps/")
}

fn is_ship_package_specifier(module: &str) -> bool {
    let m = module.trim();
    if !m.starts_with("@ship/") {
        return false;
    }
    // Compiler-injected ambient line — not authored.
    if m == COMPILER_INJECT_SPECIFIER {
        return false;
    }
    true
}

struct FrameworkImportVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
}

impl FrameworkImportVisitor<'_> {
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

impl<'a> Visit<'a> for FrameworkImportVisitor<'_> {
    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        // Whole-declaration `import type { … }` — free.
        if decl.import_kind.is_type() {
            return;
        }
        let module = decl.source.value.as_str();
        if !is_ship_package_specifier(module) {
            return;
        }

        // Side-effect import with no bindings — still a framework import.
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
        self.report(decl.span, module);
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

    fn lint_at(path: &str, src: &str) -> Vec<Finding> {
        check(Path::new(path), src)
    }

    #[test]
    fn errors_on_runtime_import_in_app_controller() {
        let f = lint_at(
            "apps/demo/src/widget/widget.controller.ts",
            "import { controller, signal } from '@ship/sail-runtime'\n\
             export default controller(() => ({ n: signal(0) }))\n",
        );
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].severity, Severity::Error);
        assert_eq!(f[0].rule, RULE_ID);
        let lower = f[0].message.to_ascii_lowercase();
        assert!(lower.contains("ambient"), "{f:?}");
        assert!(
            lower.contains("framework") || lower.contains("delete"),
            "{f:?}"
        );
        assert!(lower.contains("import"), "{f:?}");
    }

    #[test]
    fn errors_on_any_ship_package() {
        let f = lint_at(
            "apps/demo/src/widget/widget.controller.ts",
            "import { applyTheme } from '@ship/tokens'\n\
             export default controller(() => ({}))\n",
        );
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("@ship/tokens"), "{f:?}");
    }

    #[test]
    fn errors_on_app_src_define_store() {
        let f = lint_at(
            "apps/demo/src/stores/tally.ts",
            "import { defineStore } from '@ship/sail-runtime'\n\
             export const tally = defineStore('tally', () => ({}))\n",
        );
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].severity, Severity::Error);
    }

    #[test]
    fn allows_type_only() {
        let f = lint_at(
            "apps/demo/src/widget/widget.controller.ts",
            "import type { Signal } from '@ship/sail-runtime'\n\
             export default controller(() => ({ n: signal(0) }))\n",
        );
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn allows_inline_type_specifiers_only() {
        let f = lint_at(
            "apps/demo/src/widget/widget.controller.ts",
            "import { type Signal } from '@ship/sail-runtime'\n\
             export default controller(() => ({ n: signal(0) }))\n",
        );
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn allows_at_slash_app_local() {
        let f = lint_at(
            "apps/demo/src/widget/widget.controller.ts",
            "import { tally } from '@/stores/tally'\n\
             export default controller(() => ({ tally: useStore(tally) }))\n",
        );
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn pack_controllers_get_teaching_error() {
        // S5.3: packs are third-party — no L2 import-law exemption.
        let f = lint_at(
            "packages/components/sail/button/src/button.controller.ts",
            "import { controller, signal } from '@ship/sail-runtime'\n\
             export default controller(() => ({ n: signal(0) }))\n",
        );
        assert_eq!(f.len(), 1, "pack must teaching-Error @ship/* value import: {f:?}");
        assert_eq!(f[0].severity, Severity::Error);
        assert_eq!(f[0].rule, RULE_ID);
    }

    #[test]
    fn ignores_compiler_injected_internal() {
        let f = lint_at(
            "apps/demo/src/widget/widget.controller.ts",
            "import { controller, signal } from '@ship/sail-runtime/internal'\n\
             export default controller(() => ({ n: signal(0) }))\n",
        );
        assert!(
            f.is_empty(),
            "compiler inject must not trip teaching: {f:?}"
        );
    }

    #[test]
    fn ignores_non_app_paths() {
        let f = lint_at(
            "page.controller.ts",
            "import { signal } from '@ship/sail-runtime'\n\
             export default controller(() => ({ n: signal(0) }))\n",
        );
        assert!(f.is_empty(), "path-gated to apps/**: {f:?}");
    }
}
