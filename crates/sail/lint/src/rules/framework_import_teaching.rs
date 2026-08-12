//! `sail-framework-import-teaching` — import law for app + third-party pack
//! code (SPEC-AMBIENT-SURFACE-1 §2; S5.3 completion / JOB-E3).
//!
//! Law (operator-ratified; S5.3 supersedes the S5.1 L2 pack carve-out):
//! - Framework functions are ambient — authors delete `@ship/*` value imports.
//! - Fires on app controllers (`.controller.ts` / inline controller under
//!   `apps/**`), plain app TypeScript under `apps/*/src/**` (via
//!   `APP_SOURCE_RULES`), and pack controllers under `packages/components/**`
//!   (packs are third-party, not framework).
//! - Type-only imports (`import type`, `import { type X }`) remain free for
//!   `@ship/*` (types are not the ambient value surface).
//! - App-local `@/` imports are not `@ship/*` and never fire this rule.
//! - Compiler-injected ambient imports from `@ship/sail-runtime/internal`
//!   are exempt — `extract_controller_block` prepends them before the purity
//!   hard-wall lints the body; flagging them would Error every ambient use.
//! - **Deep `packages/**` dual paths (S5.3 / E3):** app product sources must
//!   not import monorepo packages by relative/`packages/` path. Wrap the lib
//!   in an app-local vendor facade under `src/vendor/` (or `@/vendor/…`).
//!   `apps/*/src/vendor/**` is the legal packaging seam and is exempt.
//!
//! Pack conventions (documented for authors):
//! - Controllers: ambient names only — no `@ship/*` value imports.
//! - Composition / engine modules stay pack-local (relative within the pack).
//! - Cross-pack composition prefers pack-relative paths, not app-style
//!   `@ship/*` value imports (same teaching when the rule sees a controller).
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
const MESSAGE: &str = "framework functions are ambient — delete this import from `@ship/*`";

const SUGGEST: &str = "Drop the `@ship/*` value import. Ambient names (signal, controller, \
     query, store ops, theme, …) need no import; app-local modules use `@/`. \
     Pack controllers: same law — ambient only; composition stays pack-local \
     (relative within the pack), not app-style `@ship/*` value imports.";

/// Deep monorepo path dual-path teaching (app product sources only).
const FACADE_MESSAGE: &str =
    "deep `packages/**` imports are forbidden in app code — wrap the lib in \
     an app-local vendor facade under `src/vendor/`, then import that facade";

const FACADE_SUGGEST: &str =
    "Add `src/vendor/<name>.ts` that re-exports the package, then import that \
     facade (relative path, or `@/vendor/<name>` once mapped). Vendor facades \
     under `apps/*/src/vendor/**` are the only legal hop into `packages/**` \
     from an app.";

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
        // Deep packages dual-path: apps only; vendor facades are the legal seam.
        check_deep_packages: is_app_product_path(path) && !is_vendor_facade_path(path),
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

/// App product path (not packs). Used to scope the deep-`packages/**` facade law.
fn is_app_product_path(path: &Path) -> bool {
    let p = path.to_string_lossy().replace('\\', "/");
    if p.contains("/packages/") || p.starts_with("packages/") {
        return false;
    }
    if p.contains("/crates/") || p.starts_with("crates/") {
        return false;
    }
    p.contains("/apps/") || p.starts_with("apps/")
}

/// `apps/*/src/vendor/**` — legal packaging seam (S5.2 REVISE / S5.3 E3).
/// Only this documented path is exempt; nested dirs named `vendor` elsewhere
/// (e.g. `src/compositions/vendor/`) are not.
fn is_vendor_facade_path(path: &Path) -> bool {
    let p = path.to_string_lossy().replace('\\', "/");
    p.contains("/src/vendor/")
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

/// Relative or bare path that hops into the monorepo `packages/` tree.
fn is_deep_packages_specifier(module: &str) -> bool {
    let m = module.trim().replace('\\', "/");
    if m.is_empty() || m.starts_with("@") {
        // Bare package names (`@ship/…`, npm) are not deep path dual paths.
        return false;
    }
    m.contains("/packages/") || m.starts_with("packages/")
}

struct FrameworkImportVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    check_deep_packages: bool,
    findings: Vec<Finding>,
}

impl FrameworkImportVisitor<'_> {
    fn report_ship(&mut self, span: Span, module: &str) {
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

    fn report_facade(&mut self, span: Span, module: &str) {
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message: format!("{FACADE_MESSAGE} (got `{module}`)"),
            suggest: Some(FACADE_SUGGEST.to_string()),
        });
    }
}

impl<'a> Visit<'a> for FrameworkImportVisitor<'_> {
    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        let module = decl.source.value.as_str();

        // Deep packages dual path — packaging boundary (type-only still dual path).
        if self.check_deep_packages && is_deep_packages_specifier(module) {
            self.report_facade(decl.span, module);
            return;
        }

        // Whole-declaration `import type { … }` — free for `@ship/*`.
        if decl.import_kind.is_type() {
            return;
        }
        if !is_ship_package_specifier(module) {
            return;
        }

        // Side-effect import with no bindings — still a framework import.
        let Some(specifiers) = &decl.specifiers else {
            self.report_ship(decl.span, module);
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
        self.report_ship(decl.span, module);
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        // Re-export from deep packages/** is still a dual path outside vendor.
        let Some(source) = &decl.source else {
            return;
        };
        let module = source.value.as_str();
        if self.check_deep_packages && is_deep_packages_specifier(module) {
            self.report_facade(decl.span, module);
        }
    }

    fn visit_export_all_declaration(&mut self, decl: &ExportAllDeclaration<'a>) {
        let module = decl.source.value.as_str();
        if self.check_deep_packages && is_deep_packages_specifier(module) {
            self.report_facade(decl.span, module);
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

    fn lint_at(path: &str, src: &str) -> Vec<Finding> {
        check(Path::new(path), src)
    }

    fn is_facade_teaching(msg: &str) -> bool {
        let lower = msg.to_ascii_lowercase();
        lower.contains("packages")
            && (lower.contains("facade") || lower.contains("vendor"))
            && lower.contains("import")
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
        assert_eq!(
            f.len(),
            1,
            "pack must teaching-Error @ship/* value import: {f:?}"
        );
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

    #[test]
    fn errors_on_deep_packages_import_in_app_src() {
        let f = lint_at(
            "apps/demo/src/compositions/boot.ts",
            "import { signal } from '../../../../packages/sail/runtime/src/index.ts'\n\
             export const n = signal(0)\n",
        );
        assert_eq!(f.len(), 1, "deep packages path must facade-teach: {f:?}");
        assert_eq!(f[0].severity, Severity::Error);
        assert_eq!(f[0].rule, RULE_ID);
        assert!(
            is_facade_teaching(&f[0].message),
            "expected facade teaching message; got: {:?}",
            f[0].message
        );
        assert!(f[0].message.contains("packages/sail/runtime"), "{f:?}");
    }

    #[test]
    fn errors_on_deep_packages_import_in_app_controller() {
        let f = lint_at(
            "apps/demo/src/widget/widget.controller.ts",
            "import { createTable } from '../../../../packages/components/sail/primitives/src/index.ts'\n\
             export default controller(() => ({}))\n",
        );
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(is_facade_teaching(&f[0].message), "{f:?}");
    }

    #[test]
    fn errors_on_type_only_deep_packages_import() {
        // Dual path is packaging, not ambient — type-only still forbidden.
        let f = lint_at(
            "apps/demo/src/stores/tally.ts",
            "import type { Signal } from '../../../../packages/sail/runtime/src/index.ts'\n\
             export type T = Signal<number>\n",
        );
        assert_eq!(f.len(), 1, "type-only deep packages still dual path: {f:?}");
        assert!(is_facade_teaching(&f[0].message), "{f:?}");
    }

    #[test]
    fn allows_vendor_facade_deep_packages_reexport() {
        let f = lint_at(
            "apps/demo/src/vendor/runtime.ts",
            "export * from '../../../../packages/sail/runtime/src/index.ts'\n",
        );
        assert!(
            f.is_empty(),
            "src/vendor/** is the legal packaging seam: {f:?}"
        );
    }

    #[test]
    fn nested_non_seam_vendor_still_facade_errors() {
        // Only apps/*/src/vendor/** is exempt — a nested dir named vendor is not.
        let f = lint_at(
            "apps/demo/src/compositions/vendor/helper.ts",
            "export * from '../../../../../packages/sail/runtime/src/index.ts'\n",
        );
        assert_eq!(
            f.len(),
            1,
            "nested non-seam vendor path must still facade-Error: {f:?}"
        );
        assert_eq!(f[0].severity, Severity::Error);
        assert_eq!(f[0].rule, RULE_ID);
        assert!(is_facade_teaching(&f[0].message), "{f:?}");
    }

    #[test]
    fn app_root_vendor_not_seam() {
        let f = lint_at(
            "apps/demo/vendor/runtime.ts",
            "export * from '../../../packages/sail/runtime/src/index.ts'\n",
        );
        assert_eq!(
            f.len(),
            1,
            "apps/*/vendor (no src/) is not the legal seam: {f:?}"
        );
        assert!(is_facade_teaching(&f[0].message), "{f:?}");
    }

    #[test]
    fn allows_relative_vendor_import_from_app() {
        let f = lint_at(
            "apps/demo/src/compositions/boot.ts",
            "import { signal } from '../vendor/runtime'\n\
             export const n = signal(0)\n",
        );
        assert!(f.is_empty(), "import via vendor facade must be free: {f:?}");
    }

    #[test]
    fn errors_on_export_named_from_deep_packages() {
        let f = lint_at(
            "apps/demo/src/compositions/boot.ts",
            "export { signal } from '../../../../packages/sail/runtime/src/index.ts'\n",
        );
        assert_eq!(
            f.len(),
            1,
            "export {{ x }} from deep packages must Error: {f:?}"
        );
        assert_eq!(f[0].rule, RULE_ID);
        assert!(is_facade_teaching(&f[0].message), "{f:?}");
    }

    #[test]
    fn pack_deep_packages_not_facade_scoped() {
        // Facade law is app-only; pack monorepo relatives are a separate residual.
        let f = lint_at(
            "packages/components/sail/media/src/sail-media-csv/sail-media-csv.controller.ts",
            "import { createTable } from '../../../../packages/components/sail/primitives/src/index.ts'\n\
             export default controller(() => ({}))\n",
        );
        assert!(
            f.is_empty(),
            "pack deep packages path is not the app facade law: {f:?}"
        );
    }
}
