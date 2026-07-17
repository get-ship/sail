//! `prop:foo={expr}` parses and emits `bindProp` + `hydrateProp`.

use sail_compiler::{compile, CompileMode, CompileOpts};

fn view_opts() -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

#[test]
fn prop_attribute_emits_bind_prop_call() {
    let src = r#"<my-grid prop:items={items()}></my-grid>"#;
    let r = compile(src, view_opts()).expect("compile");
    assert!(
        r.js.contains(r#"bindProp(__e0, "items""#),
        "expected bindProp call, got:\n{}",
        r.js
    );
}

#[test]
fn prop_imports_runtime_and_dom_helpers() {
    let src = r#"<my-grid prop:items={items()}></my-grid>"#;
    let r = compile(src, view_opts()).expect("compile");
    assert!(r.js.contains("bindProp"), "missing bindProp import");
    assert!(r.js.contains("hydrateProp"), "missing hydrateProp import");
}

#[test]
fn prop_without_expr_is_compile_error() {
    let src = r#"<my-grid prop:items="literal"></my-grid>"#;
    let err = compile(src, view_opts())
        .expect_err("prop:items=literal must fail — needs {expr}");
    assert!(
        err.message.contains("prop:items expects {expr}"),
        "msg: {}",
        err.message
    );
}

#[test]
fn empty_prop_name_is_compile_error() {
    let src = r#"<my-grid prop:={items()}></my-grid>"#;
    let err = compile(src, view_opts()).expect_err("empty prop name must fail");
    assert!(
        err.message.contains("prop: requires a property name"),
        "msg: {}",
        err.message
    );
}

#[test]
fn multiple_props_each_emit_their_own_call() {
    let src = r#"<my-grid prop:rows={rows()} prop:label={title()}></my-grid>"#;
    let r = compile(src, view_opts()).expect("compile");
    let count = r.js.matches("bindProp(").count();
    assert_eq!(count, 2, "expected 2 bindProp calls, got {count}\n{}", r.js);
}
