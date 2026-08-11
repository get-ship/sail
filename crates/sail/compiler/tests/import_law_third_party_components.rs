//! SPIKE S5.3 — third-party components (packages/components) get no import-law exemption.
//!
//! Operator doctrine: L2 packs are third-party code we happen to author. Not framework.
//! Value `import … from '@ship/sail-runtime'` in a pack controller must teaching-Error
//! the same way app code does (SPEC-AMBIENT-SURFACE-1 §2 ambient law).
//!
//! RED while framework_import_teaching still carves out packages/components.
//! Implementer must not modify this file.

use std::path::PathBuf;

use sail_lint::{lint_controller_source, Severity};

fn is_framework_import_teaching(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    lower.contains("ambient")
        && (lower.contains("framework") || lower.contains("delete"))
        && lower.contains("import")
}

#[test]
fn pack_controller_runtime_value_import_is_teaching_error() {
    let path = PathBuf::from(
        "packages/components/sail/primitives/src/sail-badge/sail-badge.controller.ts",
    );
    let source = r#"
import { controller, signal } from '@ship/sail-runtime'
export default controller(() => {
  const n = signal(0)
  return { n }
})
"#;
    let findings = lint_controller_source(&path, source);
    let hit = findings.iter().any(|f| {
        f.severity == Severity::Error && is_framework_import_teaching(&f.message)
    });
    assert!(
        hit,
        "third-party pack controller must get teaching Error on @ship/sail-runtime value import; \
         got: {findings:?}"
    );
}
