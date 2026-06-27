//! Transition directive parse + emit fixture.

use std::fs;
use std::path::PathBuf;

use sui_compiler::{compile, CompileMode, CompileOpts};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn compile_sui(name: &str) -> String {
    let path = fixtures_dir().join(format!("{name}.sui"));
    let source = fs::read_to_string(&path).expect("read fixture");
    let opts = CompileOpts {
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
        styles: None,
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
    };
    let out = compile(&source, opts).expect("compile transition fixture");
    out.js
}

#[test]
fn transition_directive_emits_register_transition() {
    let js = compile_sui("transition");
    // The compiler should produce one registerTransition call per
    // directive — three transition:, one in:, one out:.
    let calls = js.matches("registerTransition(").count();
    assert_eq!(
        calls, 4,
        "expected 4 registerTransition calls (transition:fade, in:slide, out:scale, transition:slide), got {calls}\n{js}"
    );
}

#[test]
fn transition_directive_passes_dir_string() {
    let js = compile_sui("transition");
    assert!(
        js.contains(", \"both\")"),
        "expected dir=\"both\" for transition: directive:\n{js}"
    );
    assert!(
        js.contains(", \"in\")"),
        "expected dir=\"in\" for in: directive:\n{js}"
    );
    assert!(
        js.contains(", \"out\")"),
        "expected dir=\"out\" for out: directive:\n{js}"
    );
}

#[test]
fn transition_directive_auto_imports_builtins() {
    let js = compile_sui("transition");
    // fade + slide + scale all referenced in the fixture → all should
    // be folded into the runtime import.
    let first_line = js.lines().find(|l| l.contains("@ship/sui-runtime\""))
        .unwrap_or("");
    for name in ["fade", "slide", "scale", "registerTransition"] {
        assert!(
            first_line.contains(name),
            "expected {name} in runtime import line; got:\n{first_line}"
        );
    }
}
#[test]
fn transition_directive_rejects_unknown_value_shape() {
    let source = r#"<div transition:fade="oops"></div>"#;
    let opts = CompileOpts {
        mode: CompileMode::Component,
        ..CompileOpts::default()
    };
    let err = compile(source, opts).expect_err("unexpected success");
    let msg = err.to_string();
    assert!(
        msg.contains("transition options must be an"),
        "expected error about expression form; got: {msg}"
    );
}

#[test]
fn transition_opts_object_literal_passes_scope_bound() {
    // Regression: scope-bound check used to flag the keys of a
    // transition opts object literal (`duration`, `axis`, etc.) as
    // free identifiers. The keys are property names — they don't
    // reference any outer binding and must be skipped even when the
    // surrounding scope doesn't declare them.
    //
    // `source_path` must be set so `scope_bound::check` actually
    // runs; without it the check is skipped (synthetic fragments in
    // unit tests). The path doesn't need to exist on disk — only the
    // paired-controller lookup reads from the filesystem and a
    // missing controller is fine.
    let source = r#"<div transition:slide={{ duration: 180 }}></div>"#;
    let opts = CompileOpts {
        mode: CompileMode::Component,
        source_path: Some(PathBuf::from("/nonexistent/_regression.sui")),
        ..CompileOpts::default()
    };
    compile(source, opts).expect("transition opts object literal must compile");
}

#[test]
fn transition_directive_custom_name_resolves_via_scope() {
    let source = r#"
<div transition:myfx={{ duration: 100 }}></div>
"#;
    let opts = CompileOpts {
        mode: CompileMode::Component,
        emit_component_imports: false,
        ..CompileOpts::default()
    };
    // Custom transition names compile (resolve via $scope at run time).
    // HTML lowercases attribute names, so authors must write the
    // identifier in lowercase to round-trip.
    let out = compile(source, opts).expect("custom transition compiles");
    assert!(
        out.js.contains("$scope.myfx"),
        "custom transition name must be rewritten via $scope:\n{}",
        out.js
    );
}
