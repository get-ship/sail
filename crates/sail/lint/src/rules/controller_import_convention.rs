//! `sail-controller-import-convention` — controller value imports are
//! restricted to allowed surfaces.
//!
//! Allowed (apps):
//! - `@ship/*`
//! - relative `.sail`
//! - relative paths under `src/actions/`, `src/compositions/`, `src/serverfns/`
//!
//! Allowed (component packs, structural — pack root has `customElements` in
//! package.json): pack-relative modules + pack `dependencies` /
//! `peerDependencies`.
//!
//! Type-only imports are free. Severity: Error.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sail-controller-import-convention";

// Product-facing: name the law, name the path. No process jargon, no strategy essay.
const MESSAGE: &str = "disallowed controller import — value imports must be \
     `@ship/*`, a relative `.sail` module, or under `src/actions/`, \
     `src/compositions/`, or `src/serverfns/`";

const SUGGEST: &str = "Move the module under `src/actions/`, `src/compositions/`, \
     or `src/serverfns/`; import from `@ship/*`; or use `import type` if only \
     types are needed.";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }

    let pack = pack_context_for(path);
    let mut visitor = ImportVisitor {
        source,
        file: path.to_path_buf(),
        pack,
        findings: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    visitor.findings
}

struct ImportVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    pack: Option<PackContext>,
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

        if is_allowed_value_module(module, &self.file, self.pack.as_ref()) {
            return;
        }
        self.report(decl.span, module);
    }
}

/// Resolved pack envelope for one controller path (if any).
#[derive(Debug, Clone)]
struct PackContext {
    root: PathBuf,
    /// Package names from `dependencies` + `peerDependencies`.
    declared_deps: HashSet<String>,
}

fn is_allowed_value_module(module: &str, controller_path: &Path, pack: Option<&PackContext>) -> bool {
    let m = module.trim();
    if m.is_empty() {
        return false;
    }

    // Workspace / product packages — always legal (apps and packs).
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

        // Pack envelope: any relative module that stays under the pack root.
        if let Some(pack) = pack {
            if path_is_under(&resolved, &pack.root) {
                return true;
            }
        }
        return false;
    }

    // Bare package names: L3 refuses; packs allow declared deps only.
    if let Some(pack) = pack {
        if let Some(name) = package_name_of_specifier(m) {
            if pack.declared_deps.contains(name) {
                return true;
            }
        }
    }
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

/// True when `candidate` is `root` or a descendant (path-component aware).
fn path_is_under(candidate: &Path, root: &Path) -> bool {
    let cand = normalize_path_components(candidate);
    let root = normalize_path_components(root);
    cand.starts_with(&root)
}

fn normalize_path_components(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                let _ = out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// `lodash` from `lodash/fp`; `@codemirror/state` from `@codemirror/state` or
/// a future subpath. Returns `None` for empty / malformed.
fn package_name_of_specifier(module: &str) -> Option<&str> {
    let m = module.trim();
    if m.is_empty() || m.starts_with('.') || m.starts_with('/') {
        return None;
    }
    if m.starts_with('@') {
        // @scope/name[/subpath…]
        let mut parts = m.split('/');
        let scope = parts.next()?;
        let name = parts.next()?;
        if name.is_empty() {
            return None;
        }
        // Return the longest prefix matching `@scope/name` without allocating:
        // slice back into `m`.
        let end = scope.len() + 1 + name.len();
        Some(&m[..end])
    } else {
        Some(m.split('/').next().unwrap_or(m))
    }
}

/// Structural pack root: first ancestor `package.json` that declares
/// `customElements` (CEM marker). First non-pack `package.json` stops the
/// walk (npm package boundary) so apps never inherit a parent workspace pack.
fn pack_context_for(path: &Path) -> Option<PackContext> {
    let mut cur = path.parent();
    while let Some(dir) = cur {
        let pkg_path = dir.join("package.json");
        if pkg_path.is_file() {
            return load_pack_context(dir, &pkg_path);
        }
        cur = dir.parent();
    }
    None
}

fn load_pack_context(root: &Path, pkg_path: &Path) -> Option<PackContext> {
    let text = std::fs::read_to_string(pkg_path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    // `customElements` is the pack identity CEM generation already requires.
    if value.get("customElements").and_then(|v| v.as_str()).is_none() {
        return None;
    }
    let mut declared_deps = HashSet::new();
    for key in ["dependencies", "peerDependencies"] {
        if let Some(obj) = value.get(key).and_then(|v| v.as_object()) {
            for name in obj.keys() {
                declared_deps.insert(name.clone());
            }
        }
    }
    Some(PackContext {
        root: root.to_path_buf(),
        declared_deps,
    })
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
    use std::fs;
    use std::path::Path;

    fn lint_at(path: &str, src: &str) -> Vec<Finding> {
        check(Path::new(path), src)
    }

    /// Lay out a minimal pack: `package.json` (with `customElements` + deps),
    /// optional co-located module, and a controller path under `src/`.
    fn write_pack(deps: &str, controller_rel: &str) -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let root = tmp.path();
        fs::write(
            root.join("package.json"),
            format!(
                r#"{{
                  "name": "@ship/test-pack",
                  "customElements": "dist/custom-elements.json",
                  "dependencies": {{ {deps} }}
                }}"#
            ),
        )
        .unwrap();
        fs::write(root.join("sail.config.json"), r#"{ "sx": true }"#).unwrap();
        let ctrl = root.join(controller_rel);
        if let Some(parent) = ctrl.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        // Touch a co-located engine module so relative resolve is realistic.
        if let Some(parent) = ctrl.parent() {
            fs::write(parent.join("engine.ts"), "export const x = 1\n").ok();
            fs::write(parent.join("sail-widget.internals.ts"), "export const y = 1\n").ok();
        }
        (tmp, ctrl)
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

    // ── PACK-IMPORTS-1 envelope (both directions) ─────────────────────────

    #[test]
    fn pack_allows_relative_internal_module() {
        let (_tmp, ctrl) = write_pack(
            r#""@ship/sail-runtime": "workspace:*""#,
            "src/sail-widget/sail-widget.controller.ts",
        );
        let src = "import { buildEvent } from './sail-widget.internals'\n\
                   export default controller(() => ({}))\n";
        let f = check(&ctrl, src);
        assert!(
            f.is_empty(),
            "pack-internal relative must be allowed, got {f:?}"
        );
    }

    #[test]
    fn pack_allows_relative_engine_module() {
        let (_tmp, ctrl) = write_pack(
            r#""@ship/sail-runtime": "workspace:*""#,
            "src/sail-widget/sail-widget.controller.ts",
        );
        let src = "import { createVm } from './engine'\n\
                   export default controller(() => ({}))\n";
        let f = check(&ctrl, src);
        assert!(
            f.is_empty(),
            "pack-relative engine module must be allowed, got {f:?}"
        );
    }

    #[test]
    fn pack_allows_declared_vendor() {
        let (_tmp, ctrl) = write_pack(
            r#""@codemirror/state": "^6.0.0", "mermaid": "^11.0.0""#,
            "src/sail-code/sail-code.controller.ts",
        );
        let src = "import { EditorState } from '@codemirror/state'\n\
                   import mermaid from 'mermaid'\n\
                   export default controller(() => ({}))\n";
        let f = check(&ctrl, src);
        assert!(
            f.is_empty(),
            "declared pack vendor deps must be allowed, got {f:?}"
        );
    }

    #[test]
    fn pack_refuses_undeclared_vendor() {
        let (_tmp, ctrl) = write_pack(
            r#""@ship/sail-runtime": "workspace:*""#,
            "src/sail-code/sail-code.controller.ts",
        );
        let src = "import { EditorState } from '@codemirror/state'\n\
                   export default controller(() => ({}))\n";
        let f = check(&ctrl, src);
        assert_eq!(f.len(), 1, "undeclared vendor must hard-error, got {f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert!(
            f[0].message.contains("@codemirror/state"),
            "{}",
            f[0].message
        );
        assert!(
            f[0].suggest
                .as_deref()
                .is_some_and(|s| s.contains("`@ship/*`") || s.contains("src/")),
            "suggest missing: {:?}",
            f[0].suggest
        );
    }

    #[test]
    fn pack_refuses_relative_escape_outside_root() {
        let (_tmp, ctrl) = write_pack(
            r#""@ship/sail-runtime": "workspace:*""#,
            "src/sail-widget/sail-widget.controller.ts",
        );
        // ../../.. leaves the pack root.
        let src = "import { secrets } from '../../../outside/secrets'\n\
                   export default controller(() => ({}))\n";
        let f = check(&ctrl, src);
        assert_eq!(
            f.len(),
            1,
            "relative escape outside pack root must hard-error, got {f:?}"
        );
        assert!(f[0].message.contains("secrets"), "{}", f[0].message);
    }

    #[test]
    fn non_pack_project_refuses_codemirror_vendor() {
        // App path with no pack package.json — hard error, plain wording.
        let f = lint_at(
            "apps/demo/src/page.controller.ts",
            "import { EditorState } from '@codemirror/state'\n\
             export default controller(() => ({}))\n",
        );
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].rule, RULE_ID);
        assert_eq!(f[0].severity, Severity::Error);
        assert!(
            f[0].message.contains("@codemirror/state"),
            "{}",
            f[0].message
        );
        assert!(
            f[0].message.contains("disallowed controller import"),
            "product diagnostic missing: {}",
            f[0].message
        );
        assert!(
            !f[0].message.contains("freeze"),
            "must not leak process jargon: {}",
            f[0].message
        );
    }

    #[test]
    fn package_name_of_specifier_scoped_and_subpath() {
        assert_eq!(
            package_name_of_specifier("@codemirror/state"),
            Some("@codemirror/state")
        );
        assert_eq!(
            package_name_of_specifier("@codemirror/state/dist/index.js"),
            Some("@codemirror/state")
        );
        assert_eq!(package_name_of_specifier("mermaid"), Some("mermaid"));
        assert_eq!(package_name_of_specifier("lodash/fp"), Some("lodash"));
        assert_eq!(package_name_of_specifier("./local"), None);
    }

    #[test]
    fn app_package_json_without_custom_elements_is_not_pack() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::write(
            root.join("package.json"),
            r#"{"name":"@ship/demo","dependencies":{"@codemirror/state":"^6.0.0"}}"#,
        )
        .unwrap();
        let ctrl = root.join("src/page.controller.ts");
        fs::create_dir_all(ctrl.parent().unwrap()).unwrap();
        let src = "import { EditorState } from '@codemirror/state'\n\
                   export default controller(() => ({}))\n";
        let f = check(&ctrl, src);
        assert_eq!(
            f.len(),
            1,
            "app package.json without customElements must not get pack envelope: {f:?}"
        );
    }
}
