//! SPEC-L2-PACKAGE-1 gate 5 — component package import legal; runtime value import errors.

use std::path::PathBuf;

use sail_lint::{Severity, lint_controller_source};

fn is_framework_import_teaching(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    lower.contains("ambient")
        && (lower.contains("framework") || lower.contains("delete"))
        && lower.contains("import")
}

#[test]
fn app_controller_component_package_import_is_legal() {
    let path = PathBuf::from("apps/demo/routes/home/page.controller.ts");
    let source = r#"
import '@ship/sail-components/charts'
import type { Signal } from '@ship/sail-runtime'
export default controller(() => {
  const n = signal(0)
  return { n }
})
"#;
    let findings = lint_controller_source(&path, source);
    let teaching = findings
        .iter()
        .filter(|f| f.severity == Severity::Error && is_framework_import_teaching(&f.message))
        .collect::<Vec<_>>();
    assert!(
        teaching.is_empty(),
        "component package side-effect import must be legal; got: {findings:?}"
    );
}

#[test]
fn app_controller_runtime_value_import_still_errors() {
    let path = PathBuf::from("apps/demo/routes/home/page.controller.ts");
    let source = r#"
import { signal } from '@ship/sail-runtime'
export default controller(() => {
  const n = signal(0)
  return { n }
})
"#;
    let findings = lint_controller_source(&path, source);
    let hit = findings
        .iter()
        .any(|f| f.severity == Severity::Error && is_framework_import_teaching(&f.message));
    assert!(
        hit,
        "runtime value import in app controller must still teaching-Error; got: {findings:?}"
    );
}
