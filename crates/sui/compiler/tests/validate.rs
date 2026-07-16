//! End-to-end validation tests for the three new compile-time rules:
//! closed `controller(def)` sections, scope-bound expressions, and
//! pure `derived` getters. Each test pairs a `.sui` template with a
//! `.ts` controller on disk, then asserts `compile()` rejects the
//! pair with the spec-mandated error message.

use std::fs;
use std::path::PathBuf;

use sui_compiler::{compile, CompileMode, CompileOpts};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/validate/fixtures")
}

fn opts(source_path: PathBuf) -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: Some("./view.controller.ts".into()),
        mode: CompileMode::Component,
        source_path: Some(source_path),
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

fn read(name: &str) -> (String, PathBuf) {
    let dir = fixtures_dir();
    let sui = dir.join(format!("{name}.sui"));
    let src = fs::read_to_string(&sui).unwrap_or_else(|e| panic!("read {}: {e}", sui.display()));
    (src, sui)
}

#[test]
fn unknown_controller_section_rejected() {
    let (src, path) = read("unknown-section");
    let err = compile(&src, opts(path)).expect_err("unknown section must fail");
    assert!(
        err.message.contains("Unknown controller section 'effects'"),
        "msg: {}",
        err.message
    );
    // The living section set: `use` died, `scope` became `setup`. (The
    // controller-collapse job shrinks this list again — its spec owns that
    // teaching update.)
    assert!(
        err.message.contains("setup, derived, on, mount, unmount"),
        "msg should list valid sections: {}",
        err.message
    );
}

#[test]
fn undeclared_identifier_rejected() {
    let (src, path) = read("undeclared-identifier");
    let err = compile(&src, opts(path)).expect_err("undeclared identifier must fail");
    assert!(
        err.message.contains("Identifier 'undeclared' is not declared in scope"),
        "msg: {}",
        err.message
    );
    assert!(
        err.message.contains("Available:"),
        "msg should list available names: {}",
        err.message
    );
}

#[test]
fn impure_derived_rejected() {
    let (src, path) = read("impure-derived");
    let err = compile(&src, opts(path)).expect_err("impure derived must fail");
    assert!(
        err.message.contains("Derived 'x'"),
        "msg: {}",
        err.message
    );
    assert!(
        err.message.contains("Date.now"),
        "msg should name the side effect: {}",
        err.message
    );
    assert!(
        err.message.contains("mount() or use()"),
        "msg should suggest the alternative: {}",
        err.message
    );
}

#[test]
fn pure_derived_with_paired_controller_compiles() {
    let (src, path) = read("happy-derived");
    compile(&src, opts(path)).expect("a clean controller pair should compile");
}
