//! Gate tests for SPEC-AMBIENT-SURFACE-1 §2 — the import law.
//!
//! Spike S5.1 red gate. Calls product surfaces only (`compile`,
//! `sail_lint::lint_controller_source`, `sail_lint::lint_app_source_source`).
//! Implementer must not modify, skip, or weaken these tests.
//!
//! Law (operator-ratified; S5.3 supersedes the S5.1 L2 pack carve-out):
//! - `import … from '@ship/<anything>'` in **app** code (`.sail` controller
//!   regions or `apps/*/src/**` TS) is a **teaching Error** that names the
//!   ambient model and tells the author to delete the import.
//! - Teaching shape (SPEC): *"framework functions are ambient — delete this
//!   import."*
//! - Pack code under `packages/components/sail/**` is **not** exempt (S5.3):
//!   packs are third-party, same import law as apps.
//! - App-local `@/` imports remain legal and must not fire this teaching.
//!
//! Severity: Error (fails `compile` hard-wall + `check_all`). Not a soft Warn.

use std::path::{Path, PathBuf};

use sail_compiler::{CompileMode, CompileOpts, ComponentMeta, compile};
use sail_lint::{Severity, lint_app_source_source, lint_controller_source};

/// SPEC teaching: names ambient + tells the author to drop the framework import.
fn is_framework_import_teaching(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    let names_ambient = lower.contains("ambient");
    let names_framework_or_delete = lower.contains("framework") || lower.contains("delete");
    let names_import = lower.contains("import");
    names_ambient && names_framework_or_delete && names_import
}

fn has_teaching_error(findings: &[sail_lint::Finding]) -> bool {
    findings
        .iter()
        .any(|f| f.severity == Severity::Error && is_framework_import_teaching(&f.message))
}

fn component_opts(name: &str, path: &str) -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: Some(format!("./{name}.sail.controller.ts")),
        mode: CompileMode::Component,
        source_path: Some(PathBuf::from(path)),
        component_meta: Some(ComponentMeta {
            name: name.to_string(),
        }),
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

// ── 1. Controller / .sail app code ──────────────────────────────────────────

/// Classic cargo-cult: value-import runtime primitives that are already ambient.
const CONTROLLER_WITH_RUNTIME_IMPORT: &str = r#"
<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller(() => {
  const n = signal(0)
  return { n }
})
</controller>
<div>{n()}</div>
"#;

/// Non-historical ambient surface still imported — must also teach (full table).
/// Keep the body free of `query(...)` call forms so query-key lowering cannot
/// mask the import-law Error once the teaching lands.
const CONTROLLER_WITH_QUERY_IMPORT: &str = r#"
<controller>
import { query } from '@ship/sail-runtime'
export default controller(() => {
  const q = query
  return { q }
})
</controller>
<div></div>
"#;

/// Tokens / other `@ship/*` packages are still framework surface for app code.
const CONTROLLER_WITH_TOKENS_IMPORT: &str = r#"
<controller>
import { applyTheme } from '@ship/tokens'
export default controller(() => {
  const apply = () => applyTheme('sail')
  return { apply }
})
</controller>
<div></div>
"#;

#[test]
fn runtime_import_in_app_controller_is_teaching_error() {
    let path = Path::new("apps/demo/src/widget/widget.controller.ts");
    let src = "import { controller, signal } from '@ship/sail-runtime'\n\
               export default controller(() => ({ n: signal(0) }))\n";
    let findings = lint_controller_source(path, src);
    assert!(
        has_teaching_error(&findings),
        "app controller value-import from @ship/sail-runtime must be an Error \
         teaching ambient (SPEC §2). findings: {findings:?}"
    );
}

#[test]
fn query_import_in_app_controller_is_teaching_error() {
    // After ambient-table completion, query is ambient too — importing it is
    // the same law, not a leftover "still needs import" exception.
    let path = Path::new("apps/demo/routes/home/page.controller.ts");
    let src = "import { query } from '@ship/sail-runtime'\n\
               export default controller(() => ({ q: query }))\n";
    let findings = lint_controller_source(path, src);
    assert!(
        has_teaching_error(&findings),
        "import {{ query }} from @ship/sail-runtime must teach ambient, not stay legal. \
         findings: {findings:?}"
    );
}

#[test]
fn any_ship_package_import_in_app_controller_is_teaching_error() {
    let path = Path::new("apps/demo/src/widget/widget.controller.ts");
    let src = "import { applyTheme } from '@ship/tokens'\n\
               export default controller(() => ({ apply: () => applyTheme('sail') }))\n";
    let findings = lint_controller_source(path, src);
    assert!(
        has_teaching_error(&findings),
        "import from @ship/tokens (any @ship/*) in app controller must teach ambient. \
         findings: {findings:?}"
    );
}

#[test]
fn framework_import_fails_compile_with_teaching_error() {
    // Hard wall: Error-severity teaching must fail compile (same path as purity).
    let err = compile(
        CONTROLLER_WITH_RUNTIME_IMPORT,
        component_opts("import-bad", "apps/demo/src/import-bad/import-bad.sail"),
    )
    .expect_err(
        "app controller with @ship/sail-runtime value import must fail compile \
         (SPEC-AMBIENT-SURFACE-1 §2 teaching Error, not soft warn)",
    );
    assert!(
        is_framework_import_teaching(&err.message)
            || err
                .category
                .as_deref()
                .is_some_and(|c| c.contains("import") || c.contains("ambient")),
        "compile error must teach framework-import ambient law; got category={:?} message={}",
        err.category,
        err.message
    );
    assert!(
        is_framework_import_teaching(&err.message),
        "compile error message must name ambient + delete/framework import; got: {}",
        err.message
    );
}

#[test]
fn query_import_fails_compile_with_teaching_error() {
    let err = compile(
        CONTROLLER_WITH_QUERY_IMPORT,
        component_opts("query-import", "apps/demo/routes/home/page.sail"),
    )
    .expect_err("query import from @ship/* must fail compile under the import law");
    assert!(
        is_framework_import_teaching(&err.message),
        "expected ambient teaching on query import; got: {}",
        err.message
    );
}

#[test]
fn tokens_import_fails_compile_with_teaching_error() {
    let err = compile(
        CONTROLLER_WITH_TOKENS_IMPORT,
        component_opts(
            "tokens-import",
            "apps/demo/src/theme-panel/theme-panel.sail",
        ),
    )
    .expect_err("@ship/tokens import in app code must fail compile under the import law");
    assert!(
        is_framework_import_teaching(&err.message),
        "expected ambient teaching on @ship/tokens import; got: {}",
        err.message
    );
}

// ── 2. Plain app TypeScript under apps/*/src ────────────────────────────────

#[test]
fn app_src_ship_import_is_teaching_error() {
    let path = Path::new("apps/demo/src/stores/tally.ts");
    let src = "import { defineStore } from '@ship/sail-runtime'\n\
               export const tally = defineStore('tally', () => ({ n: signal(0) }))\n";
    let findings = lint_app_source_source(path, src);
    assert!(
        has_teaching_error(&findings),
        "apps/*/src/** value-import from @ship/* must be teaching Error. findings: {findings:?}"
    );
}

// ── 3. Legal escapes ────────────────────────────────────────────────────────

#[test]
fn at_slash_app_local_import_is_not_framework_teaching() {
    let path = Path::new("apps/demo/src/widget/widget.controller.ts");
    let src = "import { tally } from '@/stores/tally'\n\
               export default controller(() => ({ tally: useStore(tally) }))\n";
    let findings = lint_controller_source(path, src);
    let bad: Vec<_> = findings
        .iter()
        .filter(|f| is_framework_import_teaching(&f.message))
        .collect();
    assert!(
        bad.is_empty(),
        "@/ app-local imports are legal (SPEC-PROJECT-STRUCTURE-1); must not fire \
         framework-import teaching. got: {bad:?}"
    );
}

#[test]
fn component_pack_ship_import_is_teaching_error() {
    // S5.3 supersedes S5.1 L2 exemption: packs are third-party, not framework.
    // Operator doctrine — no import-law carve-out for packages/components.
    let path = Path::new("packages/components/sail/button/src/button.controller.ts");
    let src = "import { controller, signal } from '@ship/sail-runtime'\n\
               export default controller(() => ({ n: signal(0) }))\n";
    let findings = lint_controller_source(path, src);
    assert!(
        has_teaching_error(&findings),
        "component-pack @ship/* value imports must teaching-Error (S5.3). findings: {findings:?}"
    );
}
