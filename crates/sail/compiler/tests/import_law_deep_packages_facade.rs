//! SPIKE S5.3 / JOB-E3 / G2b — deep `packages/**` imports in app code Error.
//!
//! Law: apps consume monorepo surface as `@ship/*` packages (node_modules /
//! package exports). Direct relative hops into `packages/**` are dual-path
//! cargo-cult and fail the import law hard-wall. Vendor facades are dead.
//!
//! Product surface: `sail_lint::lint_app_source_source` /
//! `sail_lint::lint_controller_source` (same rule as framework-import teaching).

use std::path::PathBuf;

use sail_lint::{Finding, Severity, lint_app_source_source, lint_controller_source};

const RULE_ID: &str = "sail-framework-import-teaching";

fn is_deep_package_teaching(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    lower.contains("packages")
        && lower.contains("import")
        && (lower.contains("@ship") || lower.contains("package export"))
}

fn deep_package_error(findings: &[Finding]) -> bool {
    findings.iter().any(|f| {
        f.severity == Severity::Error && f.rule == RULE_ID && is_deep_package_teaching(&f.message)
    })
}

#[test]
fn app_src_deep_packages_import_is_teaching_error() {
    let path = PathBuf::from("apps/demo/src/compositions/boot.ts");
    let source = r#"
import { signal } from '../../../../packages/sail/runtime/src/index.ts'
export const n = signal(0)
"#;
    let findings = lint_app_source_source(&path, source);
    assert!(
        deep_package_error(&findings),
        "deep packages/** import in app src must teaching Error; got: {findings:?}"
    );
}

#[test]
fn app_controller_deep_packages_import_is_teaching_error() {
    let path = PathBuf::from("apps/demo/src/widget/widget.controller.ts");
    let source = r#"
import { createTable } from '../../../../packages/components/sail/primitives/src/index.ts'
export default controller(() => ({}))
"#;
    let findings = lint_controller_source(&path, source);
    assert!(
        deep_package_error(&findings),
        "deep packages/** import in app controller must teaching Error; got: {findings:?}"
    );
}

#[test]
fn type_only_deep_packages_import_is_teaching_error() {
    let path = PathBuf::from("apps/demo/src/stores/tally.ts");
    let source = r#"
import type { Signal } from '../../../../packages/sail/runtime/src/index.ts'
export type T = Signal<number>
"#;
    let findings = lint_app_source_source(&path, source);
    assert!(
        deep_package_error(&findings),
        "type-only deep packages still dual path; got: {findings:?}"
    );
}

#[test]
fn nested_vendor_deep_packages_is_teaching_error() {
    let path = PathBuf::from("apps/demo/src/compositions/vendor/helper.ts");
    let source = r#"
export * from '../../../../../packages/sail/runtime/src/index.ts'
"#;
    let findings = lint_app_source_source(&path, source);
    assert!(
        deep_package_error(&findings),
        "nested vendor path must still Error; got: {findings:?}"
    );
}

#[test]
fn export_named_from_deep_packages_is_teaching_error() {
    let path = PathBuf::from("apps/demo/src/compositions/boot.ts");
    let source = r#"
export { signal } from '../../../../packages/sail/runtime/src/index.ts'
"#;
    let findings = lint_app_source_source(&path, source);
    assert!(
        deep_package_error(&findings),
        "export {{ x }} from deep packages must Error; got: {findings:?}"
    );
}

#[test]
fn vendor_facade_deep_packages_reexport_is_error() {
    // G2b: vendor dies — no packaging-seam exemption.
    let path = PathBuf::from("apps/demo/src/vendor/runtime.ts");
    let source = r#"
export * from '../../../../packages/sail/runtime/src/index.ts'
"#;
    let findings = lint_app_source_source(&path, source);
    assert!(
        deep_package_error(&findings),
        "apps/*/src/vendor/** is not exempt (G2b); got: {findings:?}"
    );
}

#[test]
fn app_local_ship_package_import_is_not_deep_package_teaching() {
    let path = PathBuf::from("apps/demo/src/compositions/boot.ts");
    let source = r#"
import { signal } from '@ship/sail-runtime'
export const n = signal(0)
"#;
    let findings = lint_app_source_source(&path, source);
    let bad: Vec<_> = findings
        .iter()
        .filter(|f| f.severity == Severity::Error && is_deep_package_teaching(&f.message))
        .collect();
    assert!(
        bad.is_empty(),
        "import via @ship/* package must be free; got: {bad:?}"
    );
}
