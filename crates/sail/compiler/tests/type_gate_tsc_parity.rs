//! Gate program ≡ `tsc -p` on the same tree (studio / site / components).
//!
//! Permanent: every future type-check engine swap has to keep this green.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use sail_compiler::ambients::{
    PROJECT_TSCONFIG_ACTION_REL, PROJECT_TSCONFIG_PURE_REL, write_project_type_configs,
};
use sail_compiler::check::tscheck::{
    TsDiag, TsDiagKey, collect_project_gate_files, collect_project_ts_diags, parse_tsc_diagnostics,
    rel_diag_path, reset_type_gate_servers, resolve_tsgo, ts_diag_key,
};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("workspace root")
}

fn tsc_bin(root: &Path) -> PathBuf {
    let tsc = root.join("node_modules/.bin/tsc");
    assert!(tsc.is_file(), "tsc required at {}", tsc.display());
    tsc
}

fn run_tsc_p(project: &Path, tsc: &Path, kind_rel: &str) -> Vec<TsDiag> {
    if !project.join(kind_rel).is_file() {
        return Vec::new();
    }
    let output = Command::new(tsc)
        .args(["--noEmit", "--pretty", "false", "-p", kind_rel])
        .current_dir(project)
        .output()
        .expect("run tsc -p");
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    parse_tsc_diagnostics(&text)
}

fn product_rels(project: &Path, files: &[(PathBuf, String)]) -> BTreeSet<String> {
    files
        .iter()
        .map(|(p, _)| {
            p.strip_prefix(project)
                .unwrap_or(p)
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect()
}

fn keys_for_product(
    project: &Path,
    product: &BTreeSet<String>,
    diags: &[TsDiag],
) -> BTreeSet<TsDiagKey> {
    diags
        .iter()
        .filter(|d| product.contains(&rel_diag_path(project, &d.file)))
        .map(|d| ts_diag_key(project, d))
        .collect()
}

fn assert_parity(label: &str, project: &Path, tsc: &Path) {
    write_project_type_configs(project).expect("kind configs");
    let files = collect_project_gate_files(project);
    assert!(
        !files.is_empty(),
        "{label}: expected product .ts under {}",
        project.display()
    );
    let product = product_rels(project, &files);
    reset_type_gate_servers();
    let t0 = std::time::Instant::now();
    let gate = collect_project_ts_diags(project, &files);
    let gate_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let t1 = std::time::Instant::now();
    let mut tsc_diags = run_tsc_p(project, tsc, PROJECT_TSCONFIG_PURE_REL);
    tsc_diags.extend(run_tsc_p(project, tsc, PROJECT_TSCONFIG_ACTION_REL));
    let gate_keys = keys_for_product(project, &product, &gate);
    let tsc_keys = keys_for_product(project, &product, &tsc_diags);
    let only_gate: Vec<_> = gate_keys.difference(&tsc_keys).cloned().collect();
    let only_tsc: Vec<_> = tsc_keys.difference(&gate_keys).cloned().collect();
    eprintln!(
        "{label}: product diags gate={} tsc={} gate_ms={gate_ms:.0} tsc_ms={:.0} (must be identical)",
        gate_keys.len(),
        tsc_keys.len(),
        t1.elapsed().as_secs_f64() * 1000.0
    );
    assert!(
        only_gate.is_empty() && only_tsc.is_empty(),
        "{label}: gate ≡ tsc -p failed\n  gate {}  tsc {}\n  only gate: {only_gate:#?}\n  only tsc: {only_tsc:#?}",
        gate_keys.len(),
        tsc_keys.len()
    );
}

#[test]
fn studio_gate_matches_tsc_p() {
    let root = workspace_root();
    if resolve_tsgo(Some(&root)).is_none() {
        panic!("tsgo required");
    }
    assert_parity("studio", &root.join("apps/studio"), &tsc_bin(&root));
}

#[test]
fn site_gate_matches_tsc_p() {
    let root = workspace_root();
    if resolve_tsgo(Some(&root)).is_none() {
        panic!("tsgo required");
    }
    assert_parity("site", &root.join("apps/sail-site"), &tsc_bin(&root));
}

#[test]
fn components_gate_matches_tsc_p() {
    let root = workspace_root();
    if resolve_tsgo(Some(&root)).is_none() {
        panic!("tsgo required");
    }
    assert_parity(
        "components",
        &root.join("packages/components/sail"),
        &tsc_bin(&root),
    );
}

#[test]
fn parity_fixture_red_when_css_ambient_dropped() {
    let root = workspace_root();
    if resolve_tsgo(Some(&root)).is_none() {
        panic!("tsgo required");
    }
    let tsc = tsc_bin(&root);
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path();
    std::fs::create_dir_all(project.join("src")).unwrap();
    std::fs::write(
        project.join("package.json"),
        r#"{"name":"@ship/parity-drift"}"#,
    )
    .unwrap();
    std::fs::create_dir_all(project.join("src/stores")).unwrap();
    std::fs::write(
        project.join("src/stores/counter.ts"),
        "export const n = 1\n",
    )
    .unwrap();
    std::fs::write(
        project.join("src/a.ts"),
        "import { n } from '@/stores/counter'\nimport './local.css'\nexport const m = n\n",
    )
    .unwrap();
    std::fs::write(project.join("src/local.css"), "/* empty */\n").unwrap();
    write_project_type_configs(project).expect("kind configs");

    let files = vec![
        (
            project.join("src/a.ts"),
            std::fs::read_to_string(project.join("src/a.ts")).unwrap(),
        ),
        (
            project.join("src/stores/counter.ts"),
            std::fs::read_to_string(project.join("src/stores/counter.ts")).unwrap(),
        ),
    ];
    let product = product_rels(project, &files);
    let tsc_diags = run_tsc_p(project, &tsc, PROJECT_TSCONFIG_PURE_REL);
    let tsc_keys = keys_for_product(project, &product, &tsc_diags);

    reset_type_gate_servers();
    let gate = collect_project_ts_diags(project, &files);
    let gate_keys = keys_for_product(project, &product, &gate);
    assert_eq!(
        gate_keys, tsc_keys,
        "in-place program must match tsc before the drift seed"
    );

    // Drop the ambient `*.css` script after tsc has already answered. The
    // gate reloads the kind program and must now disagree (TS2882).
    let css_ambient = project.join("node_modules/.sail/css-modules.d.ts");
    std::fs::write(&css_ambient, "// no declare module '*.css'\n").expect("clear css ambient");
    reset_type_gate_servers();
    let drifted = collect_project_ts_diags(project, &files);
    let drifted_keys = keys_for_product(project, &product, &drifted);
    assert_ne!(
        drifted_keys, tsc_keys,
        "dropping css-modules.d.ts must diverge from tsc -p (fixture is not a tautology)\n  tsc={tsc_keys:?}\n  drifted={drifted_keys:?}"
    );
    assert!(
        drifted
            .iter()
            .any(|d| d.code == "TS2882" || d.code == "TS2307"),
        "dropped *.css ambient must surface TS2882/TS2307: {drifted:?}"
    );
}
