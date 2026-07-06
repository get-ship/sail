//! End-to-end coverage for the five type-strengthening gaps closed
//! in `crates/sui/compiler/src/check/{scope_member, key_identity,
//! bind_this_target, on_callable, for_iterable}.rs`. Each test drives
//! the full `compile()` pipeline so the wiring is exercised in
//! addition to the unit tests inside each check module.

use std::fs;
use std::path::PathBuf;

use sui_compiler::{compile, CompileMode, CompileOpts};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/type_strengthening_fixtures")
}

fn paired_opts(path: PathBuf) -> CompileOpts {
    CompileOpts {
        setup_url: Some("./view.controller.ts".into()),
        mode: CompileMode::Component,
        source_path: Some(path),
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

fn standalone_opts(path: PathBuf) -> CompileOpts {
    CompileOpts {
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

fn read(name: &str) -> (String, PathBuf) {
    let dir = fixtures_dir();
    let sui = dir.join(format!("{name}.sui"));
    let src = fs::read_to_string(&sui)
        .unwrap_or_else(|e| panic!("read {}: {e}", sui.display()));
    (src, sui)
}

#[test]
fn scope_dot_typo_rejected() {
    let (src, path) = read("scope-typo");
    let err = compile(&src, paired_opts(path)).expect_err("scope.coun typo must fail");
    assert!(err.message.contains("scope.coun"), "msg: {}", err.message);
    assert!(
        err.message.contains("Available"),
        "should list available scope keys: {}",
        err.message
    );
}

#[test]
fn for_key_array_literal_rejected() {
    let (src, path) = read("for-key-literal");
    let err = compile(&src, standalone_opts(path)).expect_err("array-literal key must fail");
    assert!(
        err.message.contains("array literals"),
        "msg: {}",
        err.message
    );
}

#[test]
fn for_iterable_number_rejected() {
    let (src, path) = read("for-iterable-number");
    let err = compile(&src, standalone_opts(path)).expect_err("number iterable must fail");
    assert!(err.message.contains("number"), "msg: {}", err.message);
}

#[test]
fn bind_this_to_signal_rejected() {
    let (src, path) = read("bind-this-signal");
    let err = compile(&src, paired_opts(path)).expect_err("bind:this on signal must fail");
    assert!(
        err.message.contains("`ref(...)`"),
        "msg: {}",
        err.message
    );
}

#[test]
fn on_handler_must_be_callable() {
    let (src, path) = read("on-handler-signal");
    let err = compile(&src, paired_opts(path)).expect_err("signal handler must fail");
    assert!(
        err.message.contains("callable"),
        "msg: {}",
        err.message
    );
}

#[test]
fn happy_view_with_all_checks_compiles() {
    // The classic happy-path fixture must still compile through every
    // new check pass.
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/validate/fixtures");
    let sui = dir.join("happy-derived.sui");
    let src = fs::read_to_string(&sui).unwrap();
    compile(&src, paired_opts(sui)).expect("happy-derived must still compile");
}
