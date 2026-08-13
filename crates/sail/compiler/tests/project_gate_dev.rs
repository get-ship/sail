//! JOB-DEV-GATE-1 — project Pure gate on every code-accepting path.
//!
//! Confession replay: DOM in a compositions sidecar must fail HMR, build,
//! and CI (`tsc -p tsconfig.pure.json`) — same teaching.

use std::path::{Path, PathBuf};
use std::process::Command;

use sail_compiler::ambients::write_project_type_configs;
use sail_compiler::check::tscheck::{
    check_project_type_gate, check_project_type_gate_incremental, collect_project_type_gate,
    format_project_gate_overlay, tsgo_invocation_count,
};
use sail_compiler::diagnostics::DiagSeverity;

const SIDECAR: &str = r#"
export function currentNav() {
  return typeof location !== 'undefined' ? location.pathname : '/'
}
"#;

const ACTION_SIDECAR: &str = r#"
import type { Action } from '@ship/sail-runtime'
export const currentNav: Action<undefined> = (node) => {
  return { destroy() {} }
}
"#;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("workspace root")
}

fn write_app(root: &Path) {
    std::fs::create_dir_all(root.join("src/compositions")).unwrap();
    std::fs::create_dir_all(root.join("src/actions")).unwrap();
    std::fs::create_dir_all(root.join("routes")).unwrap();
    std::fs::write(
        root.join("package.json"),
        r#"{"name":"@ship/dev-gate-fixture"}"#,
    )
    .unwrap();
    std::fs::write(root.join("src/compositions/ok.ts"), "export const n = 1\n").unwrap();
}

#[test]
fn confession_sidecar_fails_project_gate() {
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path();
    write_app(project);
    let bad = project.join("src/compositions/current-nav.ts");
    std::fs::write(&bad, SIDECAR).unwrap();
    write_project_type_configs(project).expect("kind configs");

    let files = vec![(bad.clone(), SIDECAR.to_string())];
    let diags = collect_project_type_gate(project, &files);
    let errs: Vec<_> = diags
        .iter()
        .filter(|d| d.severity == DiagSeverity::Error)
        .collect();
    assert!(
        errs.iter().any(|d| {
            d.rule.as_deref() == Some("sail-pure-region") && d.message.contains("`location`")
        }),
        "DOM in compositions sidecar must fail Pure: {errs:?}"
    );
    assert!(
        errs.iter().any(|d| d
            .message
            .contains("compositions wire state; the DOM floor lives in src/actions/")),
        "teaching LAW must surface: {errs:?}"
    );
}

#[test]
fn confession_action_typed_sidecar_fails_jurisdiction() {
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path();
    write_app(project);
    let bad = project.join("src/compositions/current-nav.ts");
    std::fs::write(&bad, ACTION_SIDECAR).unwrap();
    write_project_type_configs(project).expect("kind configs");

    let diags = collect_project_type_gate(project, &[(bad, ACTION_SIDECAR.to_string())]);
    assert!(
        diags
            .iter()
            .any(|d| d.rule.as_deref() == Some("sail-directory-jurisdiction")),
        "Action-typed sidecar must fail jurisdiction: {diags:?}"
    );
}

#[test]
fn incremental_skips_when_only_sail_changed() {
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path();
    write_app(project);
    write_project_type_configs(project).expect("kind configs");
    let before = tsgo_invocation_count();
    let diags =
        check_project_type_gate_incremental(project, &[project.join("src/widget/widget.sail")]);
    assert!(diags.is_empty(), "non-ts change must not emit: {diags:?}");
    assert_eq!(
        tsgo_invocation_count(),
        before,
        "HMR must not re-invoke tsgo when no product .ts changed"
    );
}

#[test]
fn incremental_reruns_on_composition_change() {
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path();
    write_app(project);
    let bad = project.join("src/compositions/current-nav.ts");
    std::fs::write(&bad, SIDECAR).unwrap();
    write_project_type_configs(project).expect("kind configs");
    let diags = check_project_type_gate_incremental(project, &[bad]);
    assert!(
        diags
            .iter()
            .any(|d| d.rule.as_deref() == Some("sail-pure-region")),
        "composition .ts change must run the gate: {diags:?}"
    );
}

#[test]
fn overlay_names_the_law() {
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path();
    write_app(project);
    let bad = project.join("src/compositions/current-nav.ts");
    std::fs::write(&bad, SIDECAR).unwrap();
    write_project_type_configs(project).expect("kind configs");
    let diags = check_project_type_gate(project);
    let body = format_project_gate_overlay(&diags);
    assert!(
        body.contains("compositions wire state; the DOM floor lives in src/actions/"),
        "{body}"
    );
}

/// HMR overhead: skip is ~0; warm incremental reuses tsBuildInfo.
#[test]
fn incremental_hmr_overhead_measured() {
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path();
    write_app(project);
    for i in 0..20 {
        std::fs::write(
            project.join(format!("src/compositions/m{i}.ts")),
            format!("export const n{i} = {i}\n"),
        )
        .unwrap();
    }
    write_project_type_configs(project).expect("kind configs");
    let t0 = std::time::Instant::now();
    let first = check_project_type_gate(project);
    let cold_ms = t0.elapsed().as_secs_f64() * 1000.0;
    assert!(
        first.iter().all(|d| d.severity != DiagSeverity::Error),
        "clean fixture must be green: {first:?}"
    );
    let t1 = std::time::Instant::now();
    let second = check_project_type_gate(project);
    let warm_ms = t1.elapsed().as_secs_f64() * 1000.0;
    assert!(second.iter().all(|d| d.severity != DiagSeverity::Error));
    let t2 = std::time::Instant::now();
    let skipped =
        check_project_type_gate_incremental(project, &[project.join("src/widget/widget.sail")]);
    let skip_ms = t2.elapsed().as_secs_f64() * 1000.0;
    assert!(skipped.is_empty());
    eprintln!(
        "HMR project-gate overhead: cold={cold_ms:.1}ms warm={warm_ms:.1}ms skip={skip_ms:.2}ms"
    );
}

/// CI path: `tsc -p node_modules/.sail/tsconfig.pure.json` fails the sidecar.
#[test]
fn confession_sidecar_fails_tsc_pure_config() {
    let root = workspace_root();
    let tsc = root.join("node_modules/.bin/tsc");
    if !tsc.is_file() {
        panic!("tsc required for CI-path fixture");
    }
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path();
    write_app(project);
    std::fs::write(project.join("src/compositions/current-nav.ts"), SIDECAR).unwrap();
    write_project_type_configs(project).expect("kind configs");
    // Point tsc at the workspace typescript so the generated config resolves.
    let status = Command::new(&tsc)
        .args(["--noEmit", "-p", "node_modules/.sail/tsconfig.pure.json"])
        .current_dir(project)
        .status()
        .expect("run tsc");
    assert!(
        !status.success(),
        "tsc -p tsconfig.pure.json must fail a compositions location sidecar"
    );
}
