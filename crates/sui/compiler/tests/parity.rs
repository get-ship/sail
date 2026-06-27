//! Parity tests. For each fixture, compile via the AOT compiler and
//! confirm the emitted JS is parseable by Node and structurally
//! consistent with re-running the same compile.
//!
//! The full DOM-level parity check (compile + interpret side-by-side
//! in jsdom and compare serialized HTML) lives in
//! `tests/parity_runner.mjs`. It is gated on `RUN_PARITY=1` because
//! it requires the workspace to be installed (`bun install`) so the
//! TypeScript sources of `@ship/sui-dom` and `@ship/sui-runtime`
//! resolve at runtime.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use sui_compiler::{compile, CompileMode, ComponentMeta, CompileOpts};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn ship_button_meta() -> ComponentMeta {
    ComponentMeta { name: "ship-button-test".into(), shadow: false }
}

fn compile_fixture(name: &str, mode: CompileMode, setup: Option<&str>) -> String {
    let component_meta = if mode == CompileMode::Component {
        Some(ship_button_meta())
    } else {
        None
    };
    let path = fixtures_dir().join(format!("{name}.sui"));
    let source = fs::read_to_string(&path).unwrap();
    let out = compile(
        &source,
        CompileOpts {
            setup_url: setup.map(String::from),
            mode,
            source_path: Some(path),
            styles: None,
            component_meta,
            emit_dts: false,
            emit_component_imports: false,
        },
    )
    .expect("compile");
    out.js
}

fn write_mjs(name: &str, js: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "sui-compiler-parity-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let p = dir.join(format!("{name}.mjs"));
    fs::write(&p, js).unwrap();
    p
}

fn node_check(path: &PathBuf) {
    let output = Command::new("node")
        .arg("--check")
        .arg(path)
        .output();
    let output = match output {
        Ok(o) => o,
        Err(e) => {
            eprintln!("`node` not found ({e}); skipping parity syntactic check");
            return;
        }
    };
    if output.status.success() {
        return;
    }
    // This is a SYNTAX check, not a resolution check (the full hydrate-and-run
    // path is `full_parity_with_interpreter`, gated on RUN_PARITY + a workspace
    // install). The `node` on PATH here is a Bun shim, and Bun's `node --check`
    // *resolves* bare imports — unlike real node's `--check`, which only parses.
    // Module resolution happens AFTER parsing, so a "Cannot find module" error
    // means the file parsed cleanly: that's a pass for this gate. Only a genuine
    // parse/syntax failure (which never reaches resolution) should fail.
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("Cannot find module") {
        return;
    }
    panic!(
        "compiled JS at {} did not parse with `node --check`:\n{stderr}",
        path.display()
    );
}

#[test]
fn hello_compiles_to_valid_js() {
    let js = compile_fixture("hello", CompileMode::Component, None);
    let p = write_mjs("hello", &js);
    node_check(&p);
}

#[test]
fn counter_compiles_to_valid_js() {
    let js = compile_fixture("counter", CompileMode::Component, Some("./counter.controller.ts"));
    let p = write_mjs("counter", &js);
    node_check(&p);
}

#[test]
fn agents_compiles_to_valid_js() {
    let js = compile_fixture("agents", CompileMode::Component, Some("./agents.controller.ts"));
    let p = write_mjs("agents", &js);
    node_check(&p);
}

#[test]
fn ship_button_compiles_to_valid_js() {
    let js = compile_fixture("ship-button", CompileMode::Component, None);
    let p = write_mjs("ship-button", &js);
    node_check(&p);
}

/// Compiling the same source twice must produce byte-identical JS.
/// This is the structural parity gate: the parse + emit pipeline is
/// a pure function.
#[test]
fn compile_is_deterministic() {
    for (name, mode, setup) in [
        ("hello", CompileMode::Component, None),
        ("counter", CompileMode::Component, Some("./counter.controller.ts")),
        ("agents", CompileMode::Component, Some("./agents.controller.ts")),
        ("ship-button", CompileMode::Component, None),
    ] {
        let a = compile_fixture(name, mode, setup);
        let b = compile_fixture(name, mode, setup);
        assert_eq!(a, b, "compiler not deterministic for {name}");
    }
}

/// Regression: the `{#for}` emit must not declare `const $scope = …`
/// in the same block as a `Object.create($scope)` reference. JS const
/// declarations create a Temporal Dead Zone from the start of the
/// block, so the `Object.create($scope)` would fault on the
/// to-be-declared inner `$scope` rather than reach the outer one.
/// The fix wraps the iteration body in an IIFE that takes `$scope`
/// as a parameter — outer `$scope` stays reachable; inner closures
/// see the IIFE's parameter binding.
#[test]
fn for_iteration_emit_has_no_scope_tdz() {
    let js = compile_fixture("agents", CompileMode::Component, Some("./agents.controller.ts"));
    // The view's setup body legitimately declares the outer scope
    // exactly once: `const $scope = Object.create(null);`. Any
    // additional `const $scope = …` is the TDZ-shadow pattern.
    let scope_decls = js.matches("const $scope =").count();
    assert_eq!(
        scope_decls, 1,
        "expected exactly one `const $scope = …` declaration (the outer \
         seed); extra ones are the TDZ-shadow pattern. Found {scope_decls} in:\n{js}"
    );
    // The IIFE shape must be present in both the for-render and the
    // while-error render hoists the agents fixture exercises.
    // The agents fixture's key (`agent.id`) is scope-free, so the key
    // fn emits flat — only the for-render and while-error render hoists
    // carry the IIFE wrapper.
    let iife_count = js.matches("(($scope) =>").count();
    assert!(
        iife_count >= 2,
        "expected at least 2 `(($scope) => …)(__child…)` IIFE wrappers \
         (render + while-error render); found {iife_count} in:\n{js}"
    );
    // node --check confirms the wrapped emission is still valid JS.
    let p = write_mjs("agents-tdz-regression", &js);
    node_check(&p);
}

/// Full hydrate-and-compare parity. Opt-in: requires the workspace's
/// JS dependencies to be installed (`bun install`) so that
/// `@ship/sui-dom` and `@ship/sui-runtime` resolve.
#[test]
fn full_parity_with_interpreter() {
    if std::env::var("RUN_PARITY").is_err() {
        eprintln!(
            "skipping full parity (set RUN_PARITY=1 with workspace deps installed to enable)"
        );
        return;
    }
    let runner = fixtures_dir()
        .parent()
        .unwrap()
        .join("parity_runner.mjs");
    assert!(
        runner.exists(),
        "parity_runner.mjs missing at {}",
        runner.display()
    );
    let status = Command::new("node")
        .arg(&runner)
        .status()
        .expect("invoke node");
    assert!(status.success(), "parity_runner failed");
}
