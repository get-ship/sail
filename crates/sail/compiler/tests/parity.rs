//! Parity tests. For each fixture, compile via the AOT compiler and
//! confirm the emitted JS is parseable by Node and structurally
//! consistent with re-running the same compile.
//!
//! The full DOM-level parity check (compile + interpret side-by-side
//! in jsdom and compare serialized HTML) lives in
//! `tests/parity_runner.ts`. It is gated on `RUN_PARITY=1` because
//! it requires the workspace to be installed (`bun install`) so the
//! TypeScript sources of `@ship/sail-dom` and `@ship/sail-runtime`
//! resolve at runtime.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use sail_compiler::{CompileMode, CompileOpts, ComponentMeta, compile};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn ship_button_meta() -> ComponentMeta {
    ComponentMeta {
        name: "ship-button-test".into(),
    }
}

fn compile_fixture(name: &str, mode: CompileMode, setup: Option<&str>) -> String {
    let component_meta = if mode == CompileMode::Component {
        Some(ship_button_meta())
    } else {
        None
    };
    let path = fixtures_dir().join(format!("{name}.sail"));
    let source = fs::read_to_string(&path).unwrap();
    let out = compile(
        &source,
        CompileOpts {
            element_prefix: "sail".into(),
            host_data_attr: "data-sail-host".into(),
            route_params: None,
            setup_url: setup.map(String::from),
            mode,
            source_path: Some(path),
            component_meta,
            emit_dts: false,
            scope_capabilities: Vec::new(),
            emit_component_imports: false,
            lint_overrides: Vec::new(),
        },
    )
    .expect("compile");
    out.js
}

fn write_mjs(name: &str, js: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sail-compiler-parity-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let p = dir.join(format!("{name}.mjs"));
    fs::write(&p, js).unwrap();
    p
}

fn node_check(path: &PathBuf) {
    let output = Command::new("node").arg("--check").arg(path).output();
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
    let js = compile_fixture(
        "counter",
        CompileMode::Component,
        Some("./counter.controller.ts"),
    );
    let p = write_mjs("counter", &js);
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
        (
            "counter",
            CompileMode::Component,
            Some("./counter.controller.ts"),
        ),
        ("ship-button", CompileMode::Component, None),
    ] {
        let a = compile_fixture(name, mode, setup);
        let b = compile_fixture(name, mode, setup);
        assert_eq!(a, b, "compiler not deterministic for {name}");
    }
}

/// Full hydrate-and-compare parity. Opt-in: requires the workspace's
/// JS dependencies to be installed (`bun install`) so that
/// `@ship/sail-dom` and `@ship/sail-runtime` resolve.
#[test]
fn full_parity_with_interpreter() {
    if std::env::var("RUN_PARITY").is_err() {
        eprintln!(
            "skipping full parity (set RUN_PARITY=1 with workspace deps installed to enable)"
        );
        return;
    }
    let runner = fixtures_dir().parent().unwrap().join("parity_runner.ts");
    assert!(
        runner.exists(),
        "parity_runner.ts missing at {}",
        runner.display()
    );
    let status = Command::new("bun")
        .arg(&runner)
        .status()
        .expect("invoke bun");
    assert!(status.success(), "parity_runner failed");
}
