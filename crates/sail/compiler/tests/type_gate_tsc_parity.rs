//! Gate program ≡ `tsc -p` on the same tree (studio / site / components).
//!
//! Permanent: every future type-check engine swap has to keep this green.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

/// One live tsgo program at a time — parallel parity tests otherwise
/// flake with `project session missing`.
static LIVE_PROGRAM: Mutex<()> = Mutex::new(());

use sail_compiler::DiagSeverity;
use sail_compiler::ambients::{
    PROJECT_TSCONFIG_ACTION_REL, PROJECT_TSCONFIG_SAIL_REL, write_project_type_configs,
};
use sail_compiler::check::tscheck::{
    TsDiag, TsDiagKey, check_project_type_gate, collect_project_gate_files,
    collect_project_ts_diags, collect_sail_type_gate, lsp_is_disabled, parse_tsc_diagnostics,
    rel_diag_path, reset_type_gate_servers, resolve_product_path, resolve_tsgo, ts_diag_key,
    tsgo_invocation_count,
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
        .filter(|d| {
            let rel = rel_diag_path(project, &d.file);
            if product.contains(&rel) {
                return true;
            }
            // Same suffix rule the CLI uses — CWD-prefixed labels
            // (`apps/studio/src/foo.ts`) must not vanish from the fixture.
            let f = d.file.replace('\\', "/");
            product
                .iter()
                .any(|p| f.ends_with(p.as_str()) || f.contains(&format!("/{p}")))
        })
        .map(|d| ts_diag_key(project, d))
        .collect()
}

/// Walk paths the CLI stores when CWD is the workspace (`apps/studio/src/…`).
fn cli_walk_files(workspace: &Path, files: &[(PathBuf, String)]) -> Vec<(PathBuf, String)> {
    files
        .iter()
        .map(|(p, src)| {
            let rel = p
                .strip_prefix(workspace)
                .unwrap_or(p.as_path())
                .to_path_buf();
            (rel, src.clone())
        })
        .collect()
}

fn assert_parity(label: &str, project: &Path, tsc: &Path) {
    let _live = LIVE_PROGRAM.lock().expect("live program lock");
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
    let gate = collect_project_ts_diags(project, &files).expect("live program");
    assert!(
        !lsp_is_disabled(),
        "{label}: parity must use the long-lived program, not CLI tsgo -p"
    );
    let gate_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let t1 = std::time::Instant::now();
    let mut tsc_diags = run_tsc_p(project, tsc, PROJECT_TSCONFIG_SAIL_REL);
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
fn relative_cli_walk_paths_match_tsc_p() {
    let root = workspace_root();
    if resolve_tsgo(Some(&root)).is_none() {
        panic!("tsgo required");
    }
    let tsc = tsc_bin(&root);
    for (label, rel) in [
        ("studio-cli-walk", "apps/studio"),
        ("site-cli-walk", "apps/sail-site"),
        ("components-cli-walk", "packages/components/sail"),
    ] {
        let project = root.join(rel);
        write_project_type_configs(&project).expect("kind configs");
        let files = collect_project_gate_files(&project);
        assert!(
            !files.is_empty(),
            "{label}: expected product .ts under {}",
            project.display()
        );
        let walked = cli_walk_files(&root, &files);
        assert!(
            walked.iter().any(|(p, _)| !p.is_absolute()),
            "{label}: CLI walk paths must stay CWD-relative"
        );
        for (p, _) in &walked {
            let resolved = resolve_product_path(&project, p);
            assert!(
                resolved.as_ref().is_some_and(|r| r.is_file()),
                "{label}: {} must resolve to a real file, got {resolved:?}",
                p.display()
            );
        }
        let product = product_rels(&project, &files);
        let _live = LIVE_PROGRAM.lock().expect("live program lock");
        reset_type_gate_servers();
        let gate = collect_project_ts_diags(&project, &walked).expect("live program");
        assert!(
            !lsp_is_disabled(),
            "{label}: parity must use the long-lived program, not CLI tsgo -p"
        );
        let mut tsc_diags = run_tsc_p(&project, &tsc, PROJECT_TSCONFIG_SAIL_REL);
        tsc_diags.extend(run_tsc_p(&project, &tsc, PROJECT_TSCONFIG_ACTION_REL));
        let gate_keys = keys_for_product(&project, &product, &gate);
        let tsc_keys = keys_for_product(&project, &product, &tsc_diags);
        let only_gate: Vec<_> = gate_keys.difference(&tsc_keys).cloned().collect();
        let only_tsc: Vec<_> = tsc_keys.difference(&gate_keys).cloned().collect();
        eprintln!(
            "{label}: product diags gate={} tsc={}",
            gate_keys.len(),
            tsc_keys.len()
        );
        assert!(
            only_gate.is_empty() && only_tsc.is_empty(),
            "{label}: relative walk must match tsc -p (ghost URI regression)\n  only gate: {only_gate:#?}\n  only tsc: {only_tsc:#?}"
        );
    }
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
    let tsc_diags = run_tsc_p(project, &tsc, PROJECT_TSCONFIG_SAIL_REL);
    let tsc_keys = keys_for_product(project, &product, &tsc_diags);

    reset_type_gate_servers();
    let gate = collect_project_ts_diags(project, &files).expect("live program");
    assert!(!lsp_is_disabled(), "live program required");
    let gate_keys = keys_for_product(project, &product, &gate);
    assert_eq!(
        gate_keys, tsc_keys,
        "in-place program must match tsc before the drift seed"
    );

    // Keep the session live. last_diags must not survive an emptied
    // css-modules.d.ts (file still present, body changed).
    let css_ambient = project.join("node_modules/.sail/css-modules.d.ts");
    std::fs::write(&css_ambient, "// no declare module '*.css'\n").expect("clear css ambient");
    let drifted = check_project_type_gate(project);
    assert!(
        !lsp_is_disabled(),
        "live program required after ambient edit"
    );
    assert!(
        drifted.iter().any(|d| {
            d.severity == DiagSeverity::Error
                && (d.message.contains("TS2882")
                    || d.message.contains("TS2307")
                    || d.message.contains(".css"))
        }),
        "warm session must go red after css-modules.d.ts body change: {drifted:?}"
    );
}

#[test]
fn pure_project_cannot_name_resize_observer() {
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
        r#"{"name":"@ship/parity-pure-ro"}"#,
    )
    .unwrap();
    std::fs::write(
        project.join("src/observe.ts"),
        "export const r = new ResizeObserver(() => {})\n",
    )
    .unwrap();
    write_project_type_configs(project).expect("kind configs");

    let tsc_diags = run_tsc_p(project, &tsc, PROJECT_TSCONFIG_SAIL_REL);
    assert!(
        tsc_diags
            .iter()
            .any(|d| d.code == "TS2304" && d.message.contains("ResizeObserver")),
        "tsc -p kind must refuse ResizeObserver under Pure: {tsc_diags:?}"
    );

    reset_type_gate_servers();
    let files = collect_project_gate_files(project);
    let gate = collect_project_ts_diags(project, &files).expect("live program");
    assert!(!lsp_is_disabled(), "live program required");
    assert!(
        gate.iter()
            .any(|d| d.code == "TS2304" && d.message.contains("ResizeObserver")),
        "Pure live program must match tsc (TS2304 ResizeObserver): {gate:?}"
    );
}

#[test]
fn kind_config_evicts_mirror_session() {
    let root = workspace_root();
    if resolve_tsgo(Some(&root)).is_none() {
        panic!("tsgo required");
    }
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path();
    std::fs::create_dir_all(project.join("src/stores")).unwrap();
    std::fs::write(
        project.join("package.json"),
        r#"{"name":"@ship/parity-evict"}"#,
    )
    .unwrap();
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

    reset_type_gate_servers();
    let before = tsgo_invocation_count();
    let _mirror = collect_project_ts_diags(project, &files).expect("mirror program");
    assert!(!lsp_is_disabled(), "live program required");
    let after_mirror = tsgo_invocation_count();
    assert!(
        after_mirror > before,
        "mirror collect must spawn tsgo ({before} → {after_mirror})"
    );

    write_project_type_configs(project).expect("kind configs");
    let kind = collect_project_ts_diags(project, &files).expect("kind program");
    assert!(!lsp_is_disabled(), "live program required after eviction");
    assert!(
        tsgo_invocation_count() > after_mirror,
        "kind collect must spawn a new program after evicting the mirror session"
    );
    let product = product_rels(project, &files);
    let keys = keys_for_product(project, &product, &kind);
    assert!(
        keys.iter()
            .all(|k| k.code != "TS2882" && k.code != "TS2307"),
        "in-place kind program must resolve @/ and *.css: {keys:?}"
    );
}

fn region_src(bindings: &str, body: &str) -> String {
    format!(
        "\
<controller>
import {{ {bindings} }} from './kinds'

export default controller(() => {{
{body}
}})
</controller>
<div />
"
    )
}

fn write_kinds_mod(project: &Path) {
    std::fs::create_dir_all(project.join("src")).unwrap();
    std::fs::write(
        project.join("package.json"),
        r#"{"name":"@ship/parity-export-kinds"}"#,
    )
    .unwrap();
    std::fs::write(
        project.join("src/kinds.ts"),
        "export function makeValue() {\n  return { n: 1 }\n}\n\
         export interface Face {\n  n: number\n}\n\
         export type Alias = ReturnType<typeof makeValue>\n",
    )
    .unwrap();
}

/// Gate ≡ `tsc -p` for value / interface / typeof-alias exports imported
/// from a `.sail` region via a relative path. Lives on the region program
/// (not the project-batch CLI).
#[test]
fn sail_region_relative_export_kinds_match_tsc_p() {
    let root = workspace_root();
    if resolve_tsgo(Some(&root)).is_none() {
        panic!("tsgo required");
    }
    let tsc = tsc_bin(&root);
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path();
    write_kinds_mod(project);
    std::fs::write(
        project.join("src/widget.ts"),
        "import { makeValue, type Face, type Alias } from './kinds'\n\
         export const v: Face = makeValue()\n\
         export const a: Alias = v\n",
    )
    .unwrap();
    write_project_type_configs(project).expect("kind configs");

    let tsc_diags = run_tsc_p(project, &tsc, PROJECT_TSCONFIG_SAIL_REL);
    let tsc_export_errs: Vec<_> = tsc_diags
        .iter()
        .filter(|d| d.code == "TS2305" || d.code == "TS2614" || d.code == "TS2724")
        .collect();
    assert!(
        tsc_export_errs.is_empty(),
        "tsc -p must see all three export kinds: {tsc_diags:?}"
    );

    reset_type_gate_servers();
    // Reuse order that used to phantom: value-only first, then all three.
    let first = collect_sail_type_gate(
        &region_src("makeValue", "  const v = makeValue()\n  return { v }"),
        Some(&project.join("src/first.sail")),
    );
    assert!(
        !first.iter().any(|d| d.severity == DiagSeverity::Error),
        "value-only region must be green: {first:?}"
    );
    let gate = collect_sail_type_gate(
        &region_src(
            "makeValue, type Face, type Alias",
            "  const v: Face = makeValue()\n  const a: Alias = v\n  return { v, a }",
        ),
        Some(&project.join("src/widget.sail")),
    );
    assert!(
        !lsp_is_disabled(),
        "export-kind parity must use the live program, not CLI tsgo -p"
    );
    let gate_export_errs: Vec<_> = gate
        .iter()
        .filter(|d| {
            d.severity == DiagSeverity::Error
                && (d.message.contains("TS2305")
                    || d.message.contains("TS2614")
                    || d.message.contains("TS2724")
                    || d.message.contains("no exported member"))
        })
        .collect();
    assert!(
        gate_export_errs.is_empty(),
        "live region program must match tsc on value + interface + typeof-alias: {gate:?}"
    );
}

#[test]
fn sail_region_missing_export_is_red_like_tsc_p() {
    let root = workspace_root();
    if resolve_tsgo(Some(&root)).is_none() {
        panic!("tsgo required");
    }
    let tsc = tsc_bin(&root);
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path();
    write_kinds_mod(project);
    std::fs::write(
        project.join("src/broken.ts"),
        "import { noSuch } from './kinds'\nexport const x = noSuch\n",
    )
    .unwrap();
    write_project_type_configs(project).expect("kind configs");

    let tsc_diags = run_tsc_p(project, &tsc, PROJECT_TSCONFIG_SAIL_REL);
    assert!(
        tsc_diags
            .iter()
            .any(|d| d.code == "TS2305" && d.message.contains("noSuch")),
        "tsc -p must refuse a missing named export: {tsc_diags:?}"
    );

    reset_type_gate_servers();
    let gate = collect_sail_type_gate(
        &region_src("noSuch", "  return { noSuch }"),
        Some(&project.join("src/broken.sail")),
    );
    assert!(!lsp_is_disabled(), "live program required");
    assert!(
        gate.iter().any(|d| {
            d.severity == DiagSeverity::Error
                && (d.message.contains("TS2305") || d.message.contains("noSuch"))
        }),
        "region gate must go red on a missing export (stub would silent-green): {gate:?}"
    );
}
