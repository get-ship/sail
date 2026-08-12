//! SPIKE S5.3 / JOB-E3 — deep `packages/**` imports in app code are a facade
//! teaching Error.
//!
//! Law: apps needing monorepo / third-party surface wrap it in `src/vendor/*`
//! and import that facade. Direct relative hops into `packages/**` are
//! dual-path cargo-cult and fail the import law hard-wall.
//!
//! Vendor facades themselves (`apps/*/src/vendor/**`) remain the legal seam.
//!
//! Product surface: `sail_lint::lint_app_source_source` /
//! `sail_lint::lint_controller_source` (same rule as framework-import teaching).

use std::path::PathBuf;

use sail_lint::{Finding, Severity, lint_app_source_source, lint_controller_source};

const RULE_ID: &str = "sail-framework-import-teaching";

fn is_facade_teaching(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    lower.contains("packages")
        && (lower.contains("facade") || lower.contains("vendor"))
        && lower.contains("import")
}

fn facade_error(findings: &[Finding]) -> bool {
    findings.iter().any(|f| {
        f.severity == Severity::Error && f.rule == RULE_ID && is_facade_teaching(&f.message)
    })
}

#[test]
fn app_src_deep_packages_import_is_facade_teaching_error() {
    let path = PathBuf::from("apps/demo/src/compositions/boot.ts");
    let source = r#"
import { signal } from '../../../../packages/sail/runtime/src/index.ts'
export const n = signal(0)
"#;
    let findings = lint_app_source_source(&path, source);
    assert!(
        facade_error(&findings),
        "deep packages/** import in app src must facade-teaching Error; got: {findings:?}"
    );
}

#[test]
fn app_controller_deep_packages_import_is_facade_teaching_error() {
    let path = PathBuf::from("apps/demo/src/widget/widget.controller.ts");
    let source = r#"
import { createTable } from '../../../../packages/components/sail/primitives/src/index.ts'
export default controller(() => ({}))
"#;
    let findings = lint_controller_source(&path, source);
    assert!(
        facade_error(&findings),
        "deep packages/** import in app controller must facade-teaching Error; got: {findings:?}"
    );
}

#[test]
fn type_only_deep_packages_import_is_facade_teaching_error() {
    let path = PathBuf::from("apps/demo/src/stores/tally.ts");
    let source = r#"
import type { Signal } from '../../../../packages/sail/runtime/src/index.ts'
export type T = Signal<number>
"#;
    let findings = lint_app_source_source(&path, source);
    assert!(
        facade_error(&findings),
        "type-only deep packages still dual path; got: {findings:?}"
    );
}

#[test]
fn nested_non_seam_vendor_deep_packages_is_facade_teaching_error() {
    // Only apps/*/src/vendor/** is exempt — nested …/vendor/… is not the seam.
    let path = PathBuf::from("apps/demo/src/compositions/vendor/helper.ts");
    let source = r#"
export * from '../../../../../packages/sail/runtime/src/index.ts'
"#;
    let findings = lint_app_source_source(&path, source);
    assert!(
        facade_error(&findings),
        "nested non-seam vendor path must still facade-Error; got: {findings:?}"
    );
}

#[test]
fn export_named_from_deep_packages_is_facade_teaching_error() {
    let path = PathBuf::from("apps/demo/src/compositions/boot.ts");
    let source = r#"
export { signal } from '../../../../packages/sail/runtime/src/index.ts'
"#;
    let findings = lint_app_source_source(&path, source);
    assert!(
        facade_error(&findings),
        "export {{ x }} from deep packages must facade-Error; got: {findings:?}"
    );
}

#[test]
fn vendor_facade_deep_packages_reexport_is_allowed() {
    let path = PathBuf::from("apps/demo/src/vendor/runtime.ts");
    let source = r#"
export * from '../../../../packages/sail/runtime/src/index.ts'
"#;
    let findings = lint_app_source_source(&path, source);
    let bad: Vec<_> = findings
        .iter()
        .filter(|f| f.severity == Severity::Error && is_facade_teaching(&f.message))
        .collect();
    assert!(
        bad.is_empty(),
        "apps/*/src/vendor/** is the legal packaging seam; got: {bad:?}"
    );
}

#[test]
fn app_local_vendor_import_is_not_facade_teaching() {
    let path = PathBuf::from("apps/demo/src/compositions/boot.ts");
    let source = r#"
import { signal } from '../vendor/runtime'
export const n = signal(0)
"#;
    let findings = lint_app_source_source(&path, source);
    let bad: Vec<_> = findings
        .iter()
        .filter(|f| f.severity == Severity::Error && is_facade_teaching(&f.message))
        .collect();
    assert!(
        bad.is_empty(),
        "import via app-local vendor facade must be free; got: {bad:?}"
    );
}
