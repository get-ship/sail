//! Gate tests for SPEC-PLATFORM-NEUTRALITY-1 §1 — per-kind type environments.
//!
//! Spike S2.1 red gate. Calls product APIs only.
//! Implementer must not modify, skip, or weaken these tests.
//!
//! Expected product surface (`sail_compiler::ambients`):
//! - `TypeEnv` — `Pure` | `Action` | `Framework`
//! - `type_env_for_kind(FileKind) -> TypeEnv`
//! - `type_env_for_path(path, source?) -> TypeEnv` (framework packages first, else classify)
//! - `lib_for_type_env(TypeEnv) -> &[&str]` — Pure/Action: ESNext only; Framework: +DOM
//! - `emit_action_observation_dts() -> String` — rAF / observers / getComputedStyle ambient
//!
//! Behavioral law:
//! 1. Composition / store / controller (Pure) snippets cannot name `document`
//! 2. Action snippets cannot name `document`
//! 3. Action snippets can name observation set (rAF, getComputedStyle, …)
//! 4. Framework package paths keep DOM
//! 5. `project_map::classify` feeds kind into env selection

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sail_compiler::ambients::{
    emit_action_observation_dts, lib_for_type_env, type_env_for_kind, type_env_for_path, TypeEnv,
};
use sail_compiler::project_map::{classify, FileKind};

// ── helpers ────────────────────────────────────────────────────────────────

fn has_dom_lib(lib: &[&str]) -> bool {
    lib.iter()
        .any(|e| *e == "DOM" || *e == "DOM.Iterable" || e.starts_with("DOM."))
}

fn has_esnext_lib(lib: &[&str]) -> bool {
    lib.iter()
        .any(|e| *e == "ESNext" || *e == "ES2022" || e.starts_with("ES2"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

fn tsc_path(repo: &Path) -> Option<PathBuf> {
    let p = repo.join("node_modules/.bin/tsc");
    if p.exists() {
        return Some(p);
    }
    // Allow local typescript package layout without .bin link.
    let alt = repo.join("node_modules/typescript/bin/tsc");
    alt.exists().then_some(alt)
}

/// Write a minimal tsc project for `env` and typecheck `source` as `snippet.ts`.
/// Returns (success, combined stdout+stderr).
fn tsc_snippet(env: TypeEnv, source: &str) -> Option<(bool, String)> {
    let repo = repo_root();
    let tsc = tsc_path(&repo)?;

    let work = tempfile::tempdir().expect("tempdir for type-env tsc");
    let root = work.path();

    let lib = lib_for_type_env(env);
    let lib_json: Vec<String> = lib.iter().map(|e| format!("\"{e}\"")).collect();
    let lib_arr = lib_json.join(", ");

    let mut files_include = vec!["\"./snippet.ts\"".to_string()];
    if env == TypeEnv::Action {
        let obs = emit_action_observation_dts();
        fs::write(root.join("action-observation.d.ts"), obs).unwrap();
        files_include.push("\"./action-observation.d.ts\"".to_string());
    }

    let cfg = format!(
        r#"{{
  "compilerOptions": {{
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "lib": [{lib_arr}],
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true
  }},
  "files": [{files}]
}}"#,
        files = files_include.join(", "),
    );
    fs::write(root.join("tsconfig.json"), cfg).unwrap();
    fs::write(root.join("snippet.ts"), source).unwrap();

    let out = Command::new(&tsc)
        .args(["-p", "tsconfig.json", "--noEmit"])
        .current_dir(root)
        .output()
        .expect("run tsc");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    Some((out.status.success(), combined))
}

// ── 1. classify → type env selection ───────────────────────────────────────

#[test]
fn pure_kinds_map_to_pure_env() {
    for kind in [
        FileKind::Composition,
        FileKind::Store,
        FileKind::Route,
        FileKind::Component,
    ] {
        assert_eq!(
            type_env_for_kind(kind),
            TypeEnv::Pure,
            "{kind:?} must be Pure (ESNext only — document unnameable)"
        );
    }
}

#[test]
fn action_kind_maps_to_action_env() {
    assert_eq!(
        type_env_for_kind(FileKind::Action),
        TypeEnv::Action,
        "Action kind must get the observation-only type env"
    );
}

#[test]
fn theme_and_other_do_not_grant_dom() {
    // Apps never gain free DOM merely by being Theme/Other.
    assert_eq!(type_env_for_kind(FileKind::Theme), TypeEnv::Pure);
    assert_eq!(type_env_for_kind(FileKind::Other), TypeEnv::Pure);
}

#[test]
fn classify_feeds_composition_path_to_pure() {
    let path = Path::new("apps/x/src/compositions/use-foo.ts");
    let src = "export function useFoo() { return 1 }\n";
    let c = classify(path, Some(src));
    assert_eq!(c.kind, FileKind::Composition);
    assert_eq!(
        type_env_for_path(path, Some(src)),
        TypeEnv::Pure,
        "composition path must select Pure via classify"
    );
    assert_eq!(type_env_for_kind(c.kind), TypeEnv::Pure);
}

#[test]
fn classify_feeds_store_path_to_pure() {
    let path = Path::new("apps/x/src/stores/tally.ts");
    let src = "export const tally = defineStore('tally', () => ({}))\n";
    let c = classify(path, Some(src));
    assert_eq!(c.kind, FileKind::Store);
    assert_eq!(type_env_for_path(path, Some(src)), TypeEnv::Pure);
    assert_eq!(type_env_for_kind(c.kind), TypeEnv::Pure);
}

#[test]
fn classify_feeds_action_path_to_action_env() {
    let path = Path::new("apps/x/src/actions/focus.ts");
    let src = "export function focus(node) { return {} }\n";
    let c = classify(path, Some(src));
    assert_eq!(c.kind, FileKind::Action);
    assert_eq!(
        type_env_for_path(path, Some(src)),
        TypeEnv::Action,
        "app action path must select Action env"
    );
    assert_eq!(type_env_for_kind(c.kind), TypeEnv::Action);
}

#[test]
fn controller_region_paths_are_pure() {
    // Controllers live in route/component regions — Pure, not Action/Framework.
    let route_ctrl = Path::new("apps/x/routes/home/controller.ts");
    assert_eq!(
        type_env_for_path(route_ctrl, Some("export default controller(() => ({}))\n")),
        TypeEnv::Pure,
        "route controller module must be Pure"
    );
    let comp_sail = Path::new("apps/x/src/widget/widget.sail");
    assert_eq!(
        type_env_for_path(comp_sail, None),
        TypeEnv::Pure,
        "component .sail (controller host) must be Pure"
    );
    assert_eq!(type_env_for_kind(FileKind::Component), TypeEnv::Pure);
    assert_eq!(type_env_for_kind(FileKind::Route), TypeEnv::Pure);
}

#[test]
fn framework_package_path_keeps_dom_env() {
    let cases = [
        "packages/sail/runtime/src/actions/focus.ts",
        "packages/sail/runtime/src/index.ts",
        "packages/sail/dom/src/hydrate/index.ts",
    ];
    for p in cases {
        assert_eq!(
            type_env_for_path(Path::new(p), None),
            TypeEnv::Framework,
            "framework path {p} must be Framework (DOM allowed)"
        );
    }
}

#[test]
fn framework_wins_over_actions_path_shape() {
    // Runtime catalog under …/actions/** is still Framework, not author Action.
    let path = Path::new("packages/sail/runtime/src/actions/in-view.ts");
    // classify would say Action from path shape; product must prefer framework.
    let c = classify(path, Some("export const inView = (node) => ({})\n"));
    assert_eq!(
        c.kind,
        FileKind::Action,
        "sanity: path shape classifies as Action"
    );
    assert_eq!(
        type_env_for_path(path, Some("export const inView = (node) => ({})\n")),
        TypeEnv::Framework,
        "framework package must not be stripped to Action env"
    );
}

// ── 2. lib entries ─────────────────────────────────────────────────────────

#[test]
fn pure_lib_is_esnext_only_without_dom() {
    let lib = lib_for_type_env(TypeEnv::Pure);
    assert!(
        has_esnext_lib(lib),
        "Pure env must include an ES* lib, got {lib:?}"
    );
    assert!(
        !has_dom_lib(lib),
        "Pure env must not include DOM lib entries (document unnameable), got {lib:?}"
    );
}

#[test]
fn action_lib_is_esnext_only_without_dom() {
    let lib = lib_for_type_env(TypeEnv::Action);
    assert!(
        has_esnext_lib(lib),
        "Action env must include an ES* lib, got {lib:?}"
    );
    assert!(
        !has_dom_lib(lib),
        "Action env must not include DOM lib (observation ambient only), got {lib:?}"
    );
}

#[test]
fn framework_lib_includes_dom() {
    let lib = lib_for_type_env(TypeEnv::Framework);
    assert!(
        has_esnext_lib(lib),
        "Framework env must include an ES* lib, got {lib:?}"
    );
    assert!(
        has_dom_lib(lib),
        "Framework env must include DOM, got {lib:?}"
    );
}

// ── 3. action observation ambient ──────────────────────────────────────────

#[test]
fn action_observation_ambient_declares_observation_set() {
    let dts = emit_action_observation_dts();
    assert!(
        !dts.trim().is_empty(),
        "emit_action_observation_dts must emit a non-empty ambient pack"
    );
    for name in [
        "requestAnimationFrame",
        "cancelAnimationFrame",
        "IntersectionObserver",
        "ResizeObserver",
        "MutationObserver",
        "getComputedStyle",
    ] {
        assert!(
            dts.contains(name),
            "action observation ambient must declare {name}:\n{dts}"
        );
    }
}

#[test]
fn action_observation_ambient_does_not_declare_god_globals() {
    let dts = emit_action_observation_dts();
    // Strip line comments so teaching prose may mention the banned names.
    let code: String = dts
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    for needle in [
        "declare var document",
        "declare const document",
        "declare let document",
        "declare var window",
        "declare const window",
        "declare let window",
        "declare var localStorage",
        "declare const localStorage",
        "interface Document ",
        "interface Window ",
    ] {
        assert!(
            !code.contains(needle),
            "action observation ambient must not declare god global via `{needle}`:\n{dts}"
        );
    }
}

// ── 4. tsc behavioral proofs (opt-in when tsc present) ──────────────────────

#[test]
fn pure_composition_snippet_document_fails_typecheck() {
    let Some((ok, out)) = tsc_snippet(
        TypeEnv::Pure,
        r#"
// composition / store / controller shape — must not see document
export function useTitle(n: number): void {
  document.title = `count: ${n}`
}
"#,
    ) else {
        eprintln!("SKIP pure_composition_snippet_document_fails_typecheck: tsc not found");
        // Still fail the *product* contract when tsc is absent: Pure lib must
        // exclude DOM so document is unnameable by construction.
        let lib = lib_for_type_env(TypeEnv::Pure);
        assert!(
            !has_dom_lib(lib),
            "without tsc, Pure lib must still exclude DOM (got {lib:?})"
        );
        return;
    };
    assert!(
        !ok,
        "Pure env must fail typecheck when naming document; was green:\n{out}"
    );
    assert!(
        out.contains("document")
            || out.contains("Cannot find name")
            || out.contains("2304")
            || out.contains("2552"),
        "tsc output should name document / cannot-find-name:\n{out}"
    );
}

#[test]
fn pure_store_snippet_window_fails_typecheck() {
    let Some((ok, out)) = tsc_snippet(
        TypeEnv::Pure,
        r#"
export function openBlank(url: string): void {
  window.open(url, '_blank')
}
"#,
    ) else {
        eprintln!("SKIP pure_store_snippet_window_fails_typecheck: tsc not found");
        let lib = lib_for_type_env(TypeEnv::Pure);
        assert!(!has_dom_lib(lib), "Pure lib must exclude DOM, got {lib:?}");
        return;
    };
    assert!(
        !ok,
        "Pure env must fail typecheck when naming window; was green:\n{out}"
    );
}

#[test]
fn action_snippet_document_fails_typecheck() {
    let Some((ok, out)) = tsc_snippet(
        TypeEnv::Action,
        r#"
export function paintRoot(node: Element): void {
  // banned — actions observe the handed node, never ambient document
  document.body.appendChild(node)
}
"#,
    ) else {
        eprintln!("SKIP action_snippet_document_fails_typecheck: tsc not found");
        let lib = lib_for_type_env(TypeEnv::Action);
        assert!(
            !has_dom_lib(lib),
            "Action lib must exclude DOM so document is unnameable, got {lib:?}"
        );
        return;
    };
    assert!(
        !ok,
        "Action env must fail typecheck when naming document; was green:\n{out}"
    );
}

#[test]
fn action_snippet_observation_set_typechecks() {
    let Some((ok, out)) = tsc_snippet(
        TypeEnv::Action,
        r#"
export function observe(node: Element): { destroy(): void } {
  const ro = new ResizeObserver(() => {
    void getComputedStyle(node)
  })
  ro.observe(node)
  const id = requestAnimationFrame(() => {
    void node
  })
  return {
    destroy() {
      cancelAnimationFrame(id)
      ro.disconnect()
    },
  }
}
"#,
    ) else {
        eprintln!("SKIP action_snippet_observation_set_typechecks: tsc not found");
        // Without tsc: observation ambient must still declare the set.
        let dts = emit_action_observation_dts();
        for name in ["requestAnimationFrame", "ResizeObserver", "getComputedStyle"] {
            assert!(
                dts.contains(name),
                "observation ambient must declare {name} (tsc absent):\n{dts}"
            );
        }
        let lib = lib_for_type_env(TypeEnv::Action);
        assert!(
            !has_dom_lib(lib),
            "Action lib must be ESNext-only (observation ambient supplies names), got {lib:?}"
        );
        return;
    };
    assert!(
        ok,
        "Action env must typecheck observation set (rAF / ResizeObserver / getComputedStyle):\n{out}"
    );
}

#[test]
fn framework_snippet_document_typechecks() {
    let Some((ok, out)) = tsc_snippet(
        TypeEnv::Framework,
        r#"
export function inject(styleText: string): void {
  const el = document.createElement('style')
  el.textContent = styleText
  document.head.appendChild(el)
}
"#,
    ) else {
        eprintln!("SKIP framework_snippet_document_typechecks: tsc not found");
        let lib = lib_for_type_env(TypeEnv::Framework);
        assert!(
            has_dom_lib(lib),
            "Framework lib must include DOM when tsc absent, got {lib:?}"
        );
        return;
    };
    assert!(
        ok,
        "Framework env must allow document:\n{out}"
    );
}
