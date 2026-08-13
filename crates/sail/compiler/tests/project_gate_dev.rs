//! Project Pure gate on HMR, build, and `tsc -p tsconfig.pure.json`.
//!
//! Confession replay: DOM in a compositions sidecar must fail all three
//! paths with the same teaching.

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
    let diags = check_project_type_gate_incremental(
        project,
        &[project.join("src/widget/widget.sail")],
        false,
    );
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
    let diags = check_project_type_gate_incremental(project, &[bad], false);
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

/// HMR skip is cheap; a `.ts` tick still pays a tsgo spawn (warm ≈ cold).
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
    let skipped = check_project_type_gate_incremental(
        project,
        &[project.join("src/widget/widget.sail")],
        false,
    );
    let skip_ms = t2.elapsed().as_secs_f64() * 1000.0;
    assert!(skipped.is_empty());
    // Honest: this is skip incrementality, not in-process program reuse.
    eprintln!(
        "HMR project-gate overhead: cold={cold_ms:.1}ms warm={warm_ms:.1}ms skip={skip_ms:.2}ms \
         (warm≈cold is a tsgo spawn; skip is the cheap path)"
    );
}

#[test]
fn overflow_force_runs_gate_on_sail_only_tick() {
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path();
    write_app(project);
    let bad = project.join("src/compositions/current-nav.ts");
    std::fs::write(&bad, SIDECAR).unwrap();
    write_project_type_configs(project).expect("kind configs");
    let diags = check_project_type_gate_incremental(
        project,
        &[project.join("src/widget/widget.sail")],
        true,
    );
    assert!(
        diags
            .iter()
            .any(|d| d.rule.as_deref() == Some("sail-pure-region")),
        "force (lossy path list) must still run the Pure gate: {diags:?}"
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
    let output = Command::new(&tsc)
        .args(["--noEmit", "-p", "node_modules/.sail/tsconfig.pure.json"])
        .current_dir(project)
        .output()
        .expect("run tsc");
    assert!(
        !output.status.success(),
        "tsc -p tsconfig.pure.json must fail a compositions location sidecar"
    );
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        text.contains("location") && (text.contains("TS2304") || text.contains("Cannot find name")),
        "CI-path fixture must name `location` / TS2304, not a broken tsconfig: {text}"
    );
}
