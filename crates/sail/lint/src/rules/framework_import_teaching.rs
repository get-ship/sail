//! `sail-framework-import-teaching` — import law for app + third-party components package
//! code (SPEC-AMBIENT-SURFACE-1 §2; S5.3 completion / JOB-E3; G2b L2 npm cutover).
//!
//! Law (operator-ratified; G2b supersedes the S5.2 vendor-facade packaging seam):
//! - Framework functions are ambient in **controllers** — authors delete `@ship/*`
//!   value imports from controller bodies (inline `<controller>` / `.controller.ts`).
//! - Type-only imports (`import type`, `import { type X }`) remain free for
//!   `@ship/*` (types are not the ambient value surface).
//! - App-local `@/` imports are not `@ship/*` and never fire this rule.
//! - Compiler-injected ambient imports from `@ship/sail-runtime/internal`
//!   are exempt — `extract_controller_block` prepends them before the purity
//!   hard-wall lints the body; flagging them would Error every ambient use.
//! - **Plain app modules** (`apps/*/src/**` compositions, stores, glue) consume
//!   monorepo surface as `@ship/*` packages (L2 npm model / G2b). Value imports
//!   of package exports are legal there — ambient inject only owns controllers.
//! - Component packages (export sail custom elements) remain legal value imports
//!   from apps (package.json metadata, not a hardcoded list).
//! - **Deep `packages/**` paths (E3 / G2b):** app product sources must not import
//!   monorepo packages by relative/`packages/` path. Use `@ship/<name>` package
//!   exports. Vendor facades are dead — no exemption.
//!
//! Components-package conventions (documented for authors):
//! - Controllers: ambient names only — no `@ship/*` value imports.
//! - Composition / engine modules stay package-local (relative within the package).
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
     Component-package controllers: same law — ambient only; composition stays package-local \
     (relative within the package), not app-style `@ship/*` value imports.";

/// Deep monorepo path dual-path teaching (app product sources only).
/// what / allowed / instead — external audience (G2b).
const DEEP_PACKAGE_MESSAGE: &str = "deep `packages/**` imports are forbidden in app code — import the package \
     as `@ship/<name>` via package exports";

const DEEP_PACKAGE_SUGGEST: &str = "Replace the path into `packages/**` with the package name from its \
     `package.json` (`@ship/…`). Apps consume monorepo packages through \
     node_modules resolution / package exports, not relative source hops.";

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
        // Deep packages dual-path: apps only. No vendor exemption (G2b).
        check_deep_packages: is_app_product_path(path),
        // Ambient `@ship/*` value teaching: controllers only (G2b — plain app
        // modules import packages as packages).
        check_ship_ambient: is_ship_ambient_scope(path),
        findings: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    visitor.findings
}

/// App + third-party components package controllers — framework packages stay out of scope.
fn is_app_framework_import_scope(path: &Path) -> bool {
    let p = path.to_string_lossy().replace('\\', "/");
    // S5.3: L2 components packages are third-party — same import law as apps (no carve-out).
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

/// App product path (not packs). Used to scope the deep-`packages/**` law.
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

/// Controllers own the ambient inject surface. Inline `<controller>` bodies are
/// linted under the owning `.sail` path during compile; paired controllers use
/// `.controller.ts`. Plain app modules are out of ambient teaching (G2b).
fn is_ship_ambient_scope(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if name.ends_with(".controller.ts") || name.ends_with(".controller.tsx") {
        return true;
    }
    // Inline controller region linted under the template path.
    path.extension().and_then(|e| e.to_str()) == Some("sail")
}

fn is_ship_package_specifier(module: &str, from_file: &Path) -> bool {
    let m = module.trim();
    if !m.starts_with("@ship/") {
        return false;
    }
    // Compiler-injected ambient line — not authored.
    if m == COMPILER_INJECT_SPECIFIER {
        return false;
    }
    // Component packages (export sail custom elements) — legal to import from
    // apps. Distinguished by package.json export metadata, not a hardcoded list.
    if is_component_package_specifier(m, from_file) {
        return false;
    }
    true
}

/// Extract npm package name from a bare/specifier (`@scope/name/sub` → `@scope/name`).
fn package_name_of_module(module: &str) -> Option<&str> {
    let m = module.trim();
    if m.is_empty() || m.starts_with('.') || m.starts_with('/') {
        return None;
    }
    if m.starts_with('@') {
        let mut parts = m.split('/');
        let scope = parts.next()?;
        let name = parts.next()?;
        if name.is_empty() {
            return None;
        }
        let end = scope.len() + 1 + name.len();
        Some(&m[..end])
    } else {
        Some(m.split('/').next().unwrap_or(m))
    }
}

/// Resolve `package.json` for `pkg_name` by walking ancestors of `from` for
/// `node_modules/<pkg_name>/package.json`, then (monorepo bake) scanning
/// nearby `packages/components/**/package.json` name fields.
fn resolve_package_json(from: &Path, pkg_name: &str) -> Option<PathBuf> {
    let mut cur = from.parent();
    while let Some(dir) = cur {
        let candidate = dir.join("node_modules").join(pkg_name).join("package.json");
        if candidate.is_file() {
            return Some(candidate);
        }
        cur = dir.parent();
    }
    let mut cur = from.parent();
    while let Some(dir) = cur {
        let components = dir.join("packages/components");
        if components.is_dir() {
            if let Some(found) = find_named_package_json(&components, pkg_name) {
                return Some(found);
            }
        }
        cur = dir.parent();
    }
    None
}

fn find_named_package_json(root: &Path, pkg_name: &str) -> Option<PathBuf> {
    fn walk(dir: &Path, pkg_name: &str, depth: usize) -> Option<PathBuf> {
        if depth > 6 {
            return None;
        }
        let pkg = dir.join("package.json");
        if pkg.is_file() {
            if let Ok(text) = std::fs::read_to_string(&pkg) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                    if v.get("name").and_then(|n| n.as_str()) == Some(pkg_name) {
                        return Some(pkg);
                    }
                }
            }
        }
        let entries = std::fs::read_dir(dir).ok()?;
        for e in entries.flatten() {
            let p = e.path();
            if !p.is_dir() {
                continue;
            }
            let name = e.file_name();
            let n = name.to_string_lossy();
            if n == "node_modules" || n == "dist" || n == "target" || n.starts_with('.') {
                continue;
            }
            if let Some(found) = walk(&p, pkg_name, depth + 1) {
                return Some(found);
            }
        }
        None
    }
    walk(root, pkg_name, 0)
}

/// True when package.json marks a sail components package:
/// - `customElements` string field (CEM path), or
/// - `sailComponents: true`, or
/// - any `exports` target ending in `.compiled.js`.
fn package_json_exports_components(pkg_path: &Path) -> bool {
    let Ok(text) = std::fs::read_to_string(pkg_path) else {
        return false;
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    if v.get("customElements").and_then(|x| x.as_str()).is_some() {
        return true;
    }
    if v.get("sailComponents").and_then(|x| x.as_bool()) == Some(true) {
        return true;
    }
    fn exports_look_like_components(val: &serde_json::Value) -> bool {
        match val {
            serde_json::Value::String(s) => {
                s.ends_with(".compiled.js")
                    || s.ends_with(".custom-elements.json")
                    || s.contains("custom-elements")
            }
            serde_json::Value::Object(map) => map.values().any(exports_look_like_components),
            serde_json::Value::Array(arr) => arr.iter().any(exports_look_like_components),
            _ => false,
        }
    }
    if let Some(exports) = v.get("exports") {
        if exports_look_like_components(exports) {
            return true;
        }
    }
    false
}

/// Component package specs are legal value imports (not ambient runtime).
/// Detection reads package.json metadata at resolve time (SPEC-L2-PACKAGE-1).
fn is_component_package_specifier(module: &str, from_file: &Path) -> bool {
    let m = module.trim();
    let Some(pkg_name) = package_name_of_module(m) else {
        return false;
    };
    if let Some(pkg_json) = resolve_package_json(from_file, pkg_name) {
        return package_json_exports_components(&pkg_json);
    }
    false
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
    check_ship_ambient: bool,
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

    fn report_deep_package(&mut self, span: Span, module: &str) {
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message: format!("{DEEP_PACKAGE_MESSAGE} (got `{module}`)"),
            suggest: Some(DEEP_PACKAGE_SUGGEST.to_string()),
        });
    }
}

impl<'a> Visit<'a> for FrameworkImportVisitor<'_> {
    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        let module = decl.source.value.as_str();

        // Deep packages dual path — packaging boundary (type-only still dual path).
        if self.check_deep_packages && is_deep_packages_specifier(module) {
            self.report_deep_package(decl.span, module);
            return;
        }

        if !self.check_ship_ambient {
            return;
        }

        // Whole-declaration `import type { … }` — free for `@ship/*`.
        if decl.import_kind.is_type() {
            return;
        }
        if !is_ship_package_specifier(module, &self.file) {
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
        // Re-export from deep packages/** is still a dual path.
        let Some(source) = &decl.source else {
            return;
        };
        let module = source.value.as_str();
        if self.check_deep_packages && is_deep_packages_specifier(module) {
            self.report_deep_package(decl.span, module);
        }
    }

    fn visit_export_all_declaration(&mut self, decl: &ExportAllDeclaration<'a>) {
        let module = decl.source.value.as_str();
        if self.check_deep_packages && is_deep_packages_specifier(module) {
            self.report_deep_package(decl.span, module);
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

    fn is_deep_package_teaching(msg: &str) -> bool {
        let lower = msg.to_ascii_lowercase();
        lower.contains("packages")
            && lower.contains("import")
            && (lower.contains("@ship") || lower.contains("package export"))
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
    fn allows_ship_package_import_in_plain_app_src() {
        // G2b: plain app modules consume `@ship/*` as packages.
        let f = lint_at(
            "apps/demo/src/stores/tally.ts",
            "import { defineStore } from '@ship/sail-runtime'\n\
             export const tally = defineStore('tally', () => ({}))\n",
        );
        assert!(
            f.is_empty(),
            "plain app src may import @ship/* packages (G2b L2): {f:?}"
        );
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
        // S5.3: components packages are third-party — no L2 import-law exemption.
        let f = lint_at(
            "packages/components/sail/button/src/button.controller.ts",
            "import { controller, signal } from '@ship/sail-runtime'\n\
             export default controller(() => ({ n: signal(0) }))\n",
        );
        assert_eq!(
            f.len(),
            1,
            "components package must teaching-Error @ship/* value import: {f:?}"
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
        assert_eq!(f.len(), 1, "deep packages path must Error: {f:?}");
        assert_eq!(f[0].severity, Severity::Error);
        assert_eq!(f[0].rule, RULE_ID);
        assert!(
            is_deep_package_teaching(&f[0].message),
            "expected deep-package teaching message; got: {:?}",
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
        assert!(is_deep_package_teaching(&f[0].message), "{f:?}");
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
        assert!(is_deep_package_teaching(&f[0].message), "{f:?}");
    }

    #[test]
    fn vendor_facade_deep_packages_reexport_errors() {
        // G2b: vendor dies — no packaging-seam exemption.
        let f = lint_at(
            "apps/demo/src/vendor/runtime.ts",
            "export * from '../../../../packages/sail/runtime/src/index.ts'\n",
        );
        assert_eq!(
            f.len(),
            1,
            "src/vendor/** is not exempt (G2b facade death): {f:?}"
        );
        assert_eq!(f[0].severity, Severity::Error);
        assert!(is_deep_package_teaching(&f[0].message), "{f:?}");
    }

    #[test]
    fn nested_non_seam_vendor_still_errors() {
        let f = lint_at(
            "apps/demo/src/compositions/vendor/helper.ts",
            "export * from '../../../../../packages/sail/runtime/src/index.ts'\n",
        );
        assert_eq!(f.len(), 1, "deep packages path must Error: {f:?}");
        assert_eq!(f[0].severity, Severity::Error);
        assert_eq!(f[0].rule, RULE_ID);
        assert!(is_deep_package_teaching(&f[0].message), "{f:?}");
    }

    #[test]
    fn allows_ship_package_import_from_app() {
        let f = lint_at(
            "apps/demo/src/compositions/boot.ts",
            "import { signal } from '@ship/sail-runtime'\n\
             export const n = signal(0)\n",
        );
        assert!(
            f.is_empty(),
            "import via @ship/* package must be free in plain app src: {f:?}"
        );
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
        assert!(is_deep_package_teaching(&f[0].message), "{f:?}");
    }

    #[test]
    fn pack_deep_packages_not_app_scoped() {
        // Deep-package law is app-only; components-package monorepo relatives are a separate residual.
        let f = lint_at(
            "packages/components/sail/media/src/sail-media-csv/sail-media-csv.controller.ts",
            "import { createTable } from '../../../../packages/components/sail/primitives/src/index.ts'\n\
             export default controller(() => ({}))\n",
        );
        assert!(
            f.is_empty(),
            "components-package deep packages path is not the app deep-package law: {f:?}"
        );
    }

    #[test]
    fn ambient_teaching_on_inline_sail_controller_path() {
        // Compile lints inline controller bodies under the owning .sail path.
        let f = lint_at(
            "apps/demo/src/import-bad/import-bad.sail",
            "import { controller, signal } from '@ship/sail-runtime'\n\
             export default controller(() => ({ n: signal(0) }))\n",
        );
        assert_eq!(
            f.len(),
            1,
            "inline .sail controller path must ambient-Error: {f:?}"
        );
        assert!(
            f[0].message.to_ascii_lowercase().contains("ambient"),
            "{f:?}"
        );
    }
}
