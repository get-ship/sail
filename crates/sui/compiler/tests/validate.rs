//! End-to-end validation tests for the compile-time rules: scope-bound
//! expressions and pure `derived` getters. Each test pairs a `.sui`
//! template with a `.ts` controller on disk, then asserts `compile()`
//! rejects the pair with the spec-mandated error message.
//!
//! The closed-`controller(def)`-sections rule (`unknown_controller_section_rejected`)
//! is retired along with the object-literal sections it policed: the
//! controller's setup function returns ONE flat scope object now, so
//! there's no privileged section-name allowlist to violate. An
//! unrecognized member on the returned object is either a plain typo
//! (which `undeclared_identifier_rejected` below already catches when a
//! template reads it) or just an extra scope member — ordinary
//! TypeScript, not a compiler-policed section name.

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
