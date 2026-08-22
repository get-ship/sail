//! Job 0 — same law everywhere: emit scan + purity call.
//!
//! `sail check` / build / CI scan action emit for CSSOM and invoke setup
//! once against a host that does **not** endow CSSOM. Fail → compile Error
//! with `sail-no-imperative-style`. Not an author `<test>` block. Not SES
//! in the browser. LSP [`DiagnosticTier::OnType`] stays types/IR only.
//!
//! Public path the implementer must grow: [`check_all`] / [`check_all_with_tier`]
//! / [`compile`] of a `.sail` that uses the action (OnSave / compile / project
//! check run the scan; OnType does not).

use std::fs;
use std::path::{Path, PathBuf};

use sail_compiler::{
    CompileMode, CompileOpts, DiagSeverity, Diagnostic, DiagnosticTier, check_all,
    check_all_with_tier, compile,
};

const STYLE_FLOOR: &str = "sail-no-imperative-style";

fn opts(source_path: PathBuf) -> CompileOpts {
    CompileOpts {
        element_prefix: "sail".into(),
        host_data_attr: "data-sail-host".into(),
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(source_path),
        component_meta: None,
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

struct ActionProject {
    _tmp: tempfile::TempDir,
    sail: PathBuf,
    src: String,
}

fn write_project(action_name: &str, action_ts: &str) -> ActionProject {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::write(root.join("sail.config.json"), "{}\n").unwrap();
    fs::write(
        root.join("package.json"),
        r#"{"name":"same-law-everywhere"}"#,
    )
    .unwrap();
    fs::create_dir_all(root.join("src/actions")).unwrap();
    fs::write(
        root.join(format!("src/actions/{action_name}.ts")),
        action_ts,
    )
    .unwrap();
    let sail = root.join("src/widget.sail");
    let src = format!(
        r#"<controller>
import {{ {action_name} }} from '@/actions/{action_name}'
export default controller(() => {{
  return {{ {action_name} }}
}})
</controller>
<p use:{action_name}></p>
"#
    );
    fs::write(&sail, &src).unwrap();
    ActionProject {
        _tmp: tmp,
        sail,
        src,
    }
}

fn style_floor_errors(diags: &[Diagnostic]) -> Vec<&Diagnostic> {
    diags
        .iter()
        .filter(|d| d.severity == DiagSeverity::Error && d.rule.as_deref() == Some(STYLE_FLOOR))
        .collect()
}

const DIRTY_STYLE: &str = r#"
export function paint(node: Element) {
  node.style.opacity = '0.5'
  return {}
}
"#;

const DIRTY_CREATE_STYLE: &str = r#"
export function paint(node: Element) {
  void node
  document.createElement('style')
  return {}
}
"#;

const DIRTY_COMPUTED: &str = r#"
export function paint(node: Element) {
  node['style'].opacity = '0.5'
  return {}
}
"#;

const CLEAN_LISTEN: &str = r#"
export function listen(node: Element) {
  node.addEventListener('click', () => {})
  return { destroy() {} }
}
"#;

const CLEAN_COMMENT_ONLY: &str = r#"
export function listen(node: Element) {
  // el.style — a comment is not CSSOM access
  node.addEventListener('click', () => {})
  return { destroy() {} }
}
"#;

#[test]
fn dirty_action_style_write_is_compile_error() {
    let proj = write_project("paint", DIRTY_STYLE);
    let diags = check_all(&proj.src, Some(&proj.sail), &[], None);
    assert!(
        !style_floor_errors(&diags).is_empty(),
        "check_all of a .sail that uses an action writing node.style must Error \
         with {STYLE_FLOOR} (emit scan). diags: {diags:?}"
    );
    let err = compile(&proj.src, opts(proj.sail.clone()))
        .expect_err("compile of a .sail that uses a CSSOM-writing action must fail");
    assert_eq!(
        err.category.as_deref(),
        Some(STYLE_FLOOR),
        "compile Error must carry {STYLE_FLOOR}, got: {err:?}"
    );
}

#[test]
fn dirty_action_create_element_style_is_compile_error() {
    let proj = write_project("paint", DIRTY_CREATE_STYLE);
    let diags = check_all(&proj.src, Some(&proj.sail), &[], None);
    assert!(
        !style_floor_errors(&diags).is_empty(),
        "createElement('style') in a used action is the access crime ({STYLE_FLOOR}). diags: {diags:?}"
    );
}

#[test]
fn clean_action_has_no_style_floor_error() {
    let proj = write_project("listen", CLEAN_LISTEN);
    let diags = check_all(&proj.src, Some(&proj.sail), &[], None);
    assert!(
        style_floor_errors(&diags).is_empty(),
        "listener-only action must not emit {STYLE_FLOOR}: {diags:?}"
    );
    match compile(&proj.src, opts(proj.sail.clone())) {
        Ok(_) => {}
        Err(e) => assert_ne!(
            e.category.as_deref(),
            Some(STYLE_FLOOR),
            "clean action must not fail as {STYLE_FLOOR}: {e:?}"
        ),
    }
}

#[test]
fn comment_el_style_is_not_cssom() {
    let proj = write_project("listen", CLEAN_COMMENT_ONLY);
    let diags = check_all(&proj.src, Some(&proj.sail), &[], None);
    assert!(
        style_floor_errors(&diags).is_empty(),
        "a comment `// el.style` is not CSSOM — scan must be AST/host, not a string grep: {diags:?}"
    );
}

#[test]
fn computed_style_access_in_emit_is_purity_failure() {
    // Static member lint may miss `node['style']`. Emit scan / purity call
    // (load emit, invoke setup once, host does not endow CSSOM) must fail.
    let proj = write_project("paint", DIRTY_COMPUTED);
    let diags = check_all(&proj.src, Some(&proj.sail), &[], None);
    assert!(
        !style_floor_errors(&diags).is_empty(),
        "computed CSSOM access in action emit must Error ({STYLE_FLOOR}); \
         host must not endow .style / CSSStyleSheet / insertRule. diags: {diags:?}"
    );
    let err = compile(&proj.src, opts(proj.sail.clone()))
        .expect_err("purity call: setup that paints must fail compile");
    assert_eq!(
        err.category.as_deref(),
        Some(STYLE_FLOOR),
        "purity-call Error must be {STYLE_FLOOR}, got: {err:?}"
    );
}

#[test]
fn on_type_does_not_run_emit_scan_or_purity_call() {
    let proj = write_project("paint", DIRTY_STYLE);
    let on_type = check_all_with_tier(
        &proj.src,
        Some(&proj.sail),
        &[],
        None,
        DiagnosticTier::OnType,
    );
    assert!(
        style_floor_errors(&on_type).is_empty(),
        "OnType is types/IR only — no emit-scan / purity-call (not every LSP keystroke): {on_type:?}"
    );
}

#[test]
fn on_save_runs_emit_scan() {
    let proj = write_project("paint", DIRTY_STYLE);
    let on_save = check_all_with_tier(
        &proj.src,
        Some(&proj.sail),
        &[],
        None,
        DiagnosticTier::OnSave,
    );
    assert!(
        !style_floor_errors(&on_save).is_empty(),
        "OnSave / sail check must run emit-scan: {on_save:?}"
    );
}

/// Purity call is not an author `<test>` block and not SES in a browser.
#[test]
fn purity_is_not_an_author_test_block() {
    let tmp = tempfile::tempdir().unwrap();
    let sail = tmp.path().join("src/widget.sail");
    fs::create_dir_all(sail.parent().unwrap()).unwrap();
    fs::write(tmp.path().join("sail.config.json"), "{}\n").unwrap();
    fs::create_dir_all(tmp.path().join("src/actions")).unwrap();
    fs::write(tmp.path().join("src/actions/paint.ts"), DIRTY_STYLE).unwrap();
    // No `<test>` block. The host check is compiler-owned.
    let src = r#"<controller>
import { paint } from '@/actions/paint'
export default controller(() => {
  return { paint }
})
</controller>
<p use:paint></p>
"#;
    fs::write(&sail, src).unwrap();
    assert!(
        !src.contains("<test>"),
        "fixture must not lean on an author <test> block"
    );
    let diags = check_all(src, Some(&sail), &[], None);
    assert!(
        !style_floor_errors(&diags).is_empty(),
        "compiler emit-scan / purity-call (not author <test>) must report {STYLE_FLOOR}: {diags:?}"
    );
}

#[test]
fn no_sibling_cssom_smell_rule_id() {
    let proj = write_project("paint", DIRTY_STYLE);
    let diags = check_all(&proj.src, Some(&proj.sail), &[], None);
    assert!(
        diags
            .iter()
            .all(|d| d.rule.as_deref() != Some("sail-l1-cssom-smell")),
        "do not invent sail-l1-cssom-smell: {diags:?}"
    );
}

#[test]
fn fixture_action_path_is_under_src_actions() {
    let proj = write_project("paint", DIRTY_STYLE);
    let action = proj.sail.parent().unwrap().join("actions/paint.ts");
    assert!(action.is_file(), "{}", action.display());
    let rel = action.to_string_lossy().replace('\\', "/");
    assert!(
        rel.contains("/src/actions/"),
        "dirty action must live under src/actions/**: {rel}"
    );
    let _ = Path::new(&rel);
}
