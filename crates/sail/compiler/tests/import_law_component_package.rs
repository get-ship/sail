//! SPEC-L2-PACKAGE-1 gate 5 — component package import legal; runtime value import errors.
//!
//! Component vs runtime distinction is driven by package.json export metadata
//! (`customElements` / `.compiled.js` exports), not a hardcoded package name list.

use std::fs;
use std::path::PathBuf;

use sail_lint::{Severity, lint_controller_source};
use tempfile::tempdir;

fn is_framework_import_teaching(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    lower.contains("ambient")
        && (lower.contains("framework") || lower.contains("delete"))
        && lower.contains("import")
}

/// Layout: app controller + node_modules/@ship/sail-components with component metadata.
fn write_app_with_components_dep() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempdir().expect("tmpdir");
    let app = tmp.path().join("apps/demo/routes/home");
    fs::create_dir_all(&app).unwrap();
    let ctrl = app.join("page.controller.ts");

    let pkg = tmp
        .path()
        .join("apps/demo/node_modules/@ship/sail-components");
    fs::create_dir_all(&pkg).unwrap();
    fs::write(
        pkg.join("package.json"),
        r#"{
          "name": "@ship/sail-components",
          "sailComponents": true,
          "customElements": "dist/custom-elements.json",
          "exports": {
            ".": "./src/index.js",
            "./charts": "./charts/src/index.js",
            "./sail-bar-chart": "./charts/src/sail-bar-chart/sail-bar-chart.compiled.js"
          }
        }"#,
    )
    .unwrap();

    (tmp, ctrl)
}

#[test]
fn app_controller_component_package_import_is_legal() {
    let (_tmp, path) = write_app_with_components_dep();
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
        "component package side-effect import must be legal via package metadata; got: {findings:?}"
    );
}

#[test]
fn app_controller_runtime_value_import_still_errors() {
    let (_tmp, path) = write_app_with_components_dep();
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

#[test]
fn third_party_components_package_import_legal_via_metadata() {
    let tmp = tempdir().expect("tmpdir");
    let app = tmp.path().join("apps/acme/routes/home");
    fs::create_dir_all(&app).unwrap();
    let ctrl = app.join("page.controller.ts");

    // Third-party components package — not @ship/* hardcoded list.
    let pkg = tmp.path().join("apps/acme/node_modules/@acme/sail-widgets");
    fs::create_dir_all(&pkg).unwrap();
    fs::write(
        pkg.join("package.json"),
        r#"{
          "name": "@acme/sail-widgets",
          "customElements": "dist/custom-elements.json",
          "exports": {
            ".": "./src/index.js",
            "./sail-widget": "./src/sail-widget/sail-widget.compiled.js"
          }
        }"#,
    )
    .unwrap();

    // Non-component @ship package still errors (no metadata → runtime teaching).
    // Component import of @acme/sail-widgets is free — but framework_import_teaching
    // only fires on `@ship/*`. So verify @acme is free AND a metadata-backed
    // @ship components package is free while runtime is not (covered above).
    // For @ship third-party-shaped name with metadata:
    let ship_third = tmp
        .path()
        .join("apps/acme/node_modules/@ship/community-widgets");
    fs::create_dir_all(&ship_third).unwrap();
    fs::write(
        ship_third.join("package.json"),
        r#"{
          "name": "@ship/community-widgets",
          "customElements": "dist/custom-elements.json",
          "exports": {
            "./sail-thing": "./src/sail-thing/sail-thing.compiled.js"
          }
        }"#,
    )
    .unwrap();

    let source = r#"
import '@ship/community-widgets/sail-thing'
export default controller(() => ({}))
"#;
    let findings = lint_controller_source(&ctrl, source);
    let teaching = findings
        .iter()
        .filter(|f| f.severity == Severity::Error && is_framework_import_teaching(&f.message))
        .collect::<Vec<_>>();
    assert!(
        teaching.is_empty(),
        "third-party @ship components package with customElements metadata must be legal; got: {findings:?}"
    );

    // Without metadata, same-shaped name is teaching-Error.
    let bare = tmp.path().join("apps/bare/routes/home");
    fs::create_dir_all(&bare).unwrap();
    let bare_ctrl = bare.join("page.controller.ts");
    let bare_pkg = tmp
        .path()
        .join("apps/bare/node_modules/@ship/not-components");
    fs::create_dir_all(&bare_pkg).unwrap();
    fs::write(
        bare_pkg.join("package.json"),
        r#"{"name":"@ship/not-components","main":"index.js"}"#,
    )
    .unwrap();
    let findings2 = lint_controller_source(
        &bare_ctrl,
        "import { x } from '@ship/not-components'\nexport default controller(() => ({}))\n",
    );
    let hit = findings2
        .iter()
        .any(|f| f.severity == Severity::Error && is_framework_import_teaching(&f.message));
    assert!(
        hit,
        "non-component @ship package without metadata must teaching-Error; got: {findings2:?}"
    );
}
