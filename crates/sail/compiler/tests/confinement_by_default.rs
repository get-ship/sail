//! THE RED/GREEN GATE for confine-by-default (#98): an inline `<controller>`
//! is governed by `sail-controller-no-ambient-authority` unconditionally,
//! path-independent — no `views/` substring, no `/components/` exemption.
//! Fixtures below live OUTSIDE any `views/` tree (a `packages/components/`-
//! shaped path) specifically to prove the old path heuristic is gone: the
//! lint must still fire there. `unconfined` / `network` grant the lint
//! only — `fetch` stays unnameable under the sail typecheck (no DOM).

use std::fs;
use std::path::PathBuf;

use sail_compiler::check::policy::Policy;
use sail_compiler::{CompileMode, CompileOpts, ComponentMeta, compile};

fn opts_for(path: &PathBuf) -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path.clone()),
        component_meta: Some(ComponentMeta {
            name: "sail-widget".to_string(),
        }),
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Policy::overrides_for(path),
    }
}

/// A fixture directory shaped like `packages/components/<pkg>/src/sail-widget/`
/// — the sanctioned inline-controller shape, deliberately NOT under `views/`.
/// `config` is written as `sail.config.json` at the package root when `Some`.
fn write_fixture(config: Option<&str>) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let pkg_root = tmp.path().join("packages").join("components").join("acme");
    let src_dir = pkg_root.join("src").join("sail-widget");
    fs::create_dir_all(&src_dir).unwrap();
    if let Some(body) = config {
        fs::write(pkg_root.join("sail.config.json"), body).unwrap();
    }
    let sail = src_dir.join("sail-widget.sail");
    fs::write(
        &sail,
        "<controller>\nexport default controller(() => {\n  fetch('/x')\n  return {}\n})\n</controller>\n<p>hi</p>",
    )
    .unwrap();
    (tmp, sail)
}

#[test]
fn bare_fetch_refuses_regardless_of_path_with_no_config() {
    // No sail.config.json at all: confined by default, no path heuristic to
    // exempt a `packages/components/`-shaped path — must refuse.
    let (_tmp, sail) = write_fixture(None);
    let src = fs::read_to_string(&sail).unwrap();
    let err = compile(&src, opts_for(&sail)).expect_err("bare fetch with no grant must refuse");
    assert_eq!(
        err.category.as_deref(),
        Some("sail-controller-no-ambient-authority")
    );
}

#[test]
fn unconfined_true_does_not_name_fetch_in_the_sail_typecheck() {
    // `unconfined` softens the lint. Controllers still typecheck as Sail
    // (ESNext + sail ambients, no DOM) — `fetch` is unnameable there.
    // Network/unconfined drive CSP / lint, not a second TypeEnv.
    let (_tmp, sail) = write_fixture(Some(r#"{"unconfined": true}"#));
    let src = fs::read_to_string(&sail).unwrap();
    let err = compile(&src, opts_for(&sail))
        .expect_err("unconfined must not make fetch nameable in a controller");
    assert_eq!(err.category.as_deref(), Some("sail-pure-region"));
    assert!(
        err.message.contains("fetch"),
        "refusal must name fetch: {}",
        err.message
    );
}

#[test]
fn network_origin_grant_does_not_name_fetch_in_the_sail_typecheck() {
    // `network` is CSP / origin allowlist. It is not a controller fetch API.
    let (_tmp, sail) = write_fixture(Some(r#"{"network": ["self"]}"#));
    let src = fs::read_to_string(&sail).unwrap();
    let err = compile(&src, opts_for(&sail))
        .expect_err("network: [\"self\"] must not make fetch nameable in a controller");
    assert_eq!(err.category.as_deref(), Some("sail-pure-region"));
    assert!(
        err.message.contains("fetch"),
        "refusal must name fetch: {}",
        err.message
    );
}

#[test]
fn network_grant_does_not_imply_unconfined() {
    // A `network` grant is surgical, not whole-hog: a DIFFERENT ambient-authority
    // primitive (eval, never part of the network capability) must still refuse.
    let tmp = tempfile::tempdir().unwrap();
    let pkg_root = tmp.path().join("packages").join("components").join("acme");
    let src_dir = pkg_root.join("src").join("sail-widget");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(
        pkg_root.join("sail.config.json"),
        r#"{"network": ["self"]}"#,
    )
    .unwrap();
    let sail = src_dir.join("sail-widget.sail");
    fs::write(
        &sail,
        "<controller>\nexport default controller(() => {\n  eval('2+2')\n  return {}\n})\n</controller>\n<p>hi</p>",
    )
    .unwrap();
    let src = fs::read_to_string(&sail).unwrap();
    let err = compile(&src, opts_for(&sail))
        .expect_err("eval must stay refused under a network-only grant");
    assert_eq!(
        err.category.as_deref(),
        Some("sail-controller-no-ambient-authority")
    );
}

#[test]
fn malformed_unconfined_value_fails_closed_and_stays_confined() {
    let (_tmp, sail) = write_fixture(Some(r#"{"unconfined": "yes"}"#));
    let src = fs::read_to_string(&sail).unwrap();
    let err = compile(&src, opts_for(&sail))
        .expect_err("a non-bool \"unconfined\" must fail closed to confined");
    assert_eq!(
        err.category.as_deref(),
        Some("sail-controller-no-ambient-authority")
    );
}
