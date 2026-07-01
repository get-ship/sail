//! View-side controller pairing: when a `.sui` view is compiled with
//! `setup_url: Some(...)`, the emitted JS must import the controller
//! module, call `setup.setup(ctx)`, and merge its return into `$scope`
//! via `Object.assign`. Components have always done this; this test
//! locks in the parity for views.

use sui_compiler::{compile, CompileMode, CompileOpts};

fn view_opts(setup: Option<&str>) -> CompileOpts {
    CompileOpts {
        setup_url: setup.map(|s| s.to_string()),
        mode: CompileMode::Component,
        source_path: None,
        styles: None,
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

#[test]
fn view_with_paired_controller_emits_import() {
    let src = r#"<p>{x()}</p>"#;
    let out = compile(src, view_opts(Some("./index.controller.ts")))
        .expect("compile view + paired controller");
    assert!(
        out.js.contains("import setup from \"./index.controller.ts\""),
        "expected sibling controller import, got:\n{}",
        out.js
    );
}

#[test]
fn view_with_paired_controller_calls_setup_and_merges() {
    let src = r#"<p>{x()}</p>"#;
    let out = compile(src, view_opts(Some("./index.controller.ts")))
        .expect("compile view + paired controller");
    // The view-side setup-call branch (emit_setup_call) must run. It
    // guards on `setup.setup` being a function and merges the return
    // into $scope so derived/scope additions land in template scope.
    assert!(
        out.js.contains("typeof setup.setup === \"function\""),
        "expected guarded setup.setup() invocation, got:\n{}",
        out.js
    );
    assert!(
        out.js.contains("const __r = setup.setup(ctx);"),
        "expected setup.setup(ctx) call, got:\n{}",
        out.js
    );
    assert!(
        out.js.contains("Object.assign($scope, __r)"),
        "expected Object.assign($scope, __r) merge, got:\n{}",
        out.js
    );
}

#[test]
fn view_without_paired_controller_emits_no_import_or_setup_call() {
    let src = r#"<p>{x()}</p>"#;
    let out = compile(src, view_opts(None)).expect("compile bare view");
    assert!(
        !out.js.contains("import setup from"),
        "no controller — must not emit import, got:\n{}",
        out.js
    );
    assert!(
        !out.js.contains("setup.setup(ctx)"),
        "no controller — must not emit setup.setup() call, got:\n{}",
        out.js
    );
}
