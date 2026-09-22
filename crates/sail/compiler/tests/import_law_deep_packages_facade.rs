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

/// G2b: deep `packages/**` is monorepo hygiene (`tests/hygiene`), not
/// this product rule. These tests pin that the rule stays out.
fn no_framework_teaching(findings: &[Finding]) -> bool {
    !findings.iter().any(|f| f.rule == RULE_ID)
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
        no_framework_teaching(&findings),
        "G2b: deep packages/** is hygiene, not this rule; got: {findings:?}"
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
        !findings.iter().any(|f| f.rule == RULE_ID),
        "G2b: deep packages/** is hygiene, not framework-import teaching; got: {findings:?}"
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
        no_framework_teaching(&findings),
        "G2b: type-only deep packages is hygiene, not this rule; got: {findings:?}"
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
        no_framework_teaching(&findings),
        "G2b: nested vendor deep packages is hygiene, not this rule; got: {findings:?}"
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
        no_framework_teaching(&findings),
        "G2b: export from deep packages is hygiene, not this rule; got: {findings:?}"
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
        no_framework_teaching(&findings),
        "G2b: vendor facade deep packages is hygiene, not this rule; got: {findings:?}"
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
