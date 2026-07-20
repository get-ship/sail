//! Plain `name={expr}` on custom elements emits `bindDynamic`; on native
//! IDL props emits `bindProp`. Author `prop:` / `attr:` prefixes are refused.

use sail_compiler::{compile, CompileMode, CompileOpts};

fn route_opts() -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: None,
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

#[test]
fn plain_ce_field_emits_bind_dynamic() {
    let src = r#"<my-grid items={items()}></my-grid>"#;
    let r = compile(src, route_opts()).expect("compile");
    assert!(
        r.js.contains(r#"bindDynamic(__e0, "items""#),
        "expected bindDynamic call, got:\n{}",
        r.js
    );
}

#[test]
fn plain_ce_field_imports_bind_dynamic() {
    let src = r#"<my-grid items={items()}></my-grid>"#;
    let r = compile(src, route_opts()).expect("compile");
    assert!(r.js.contains("bindDynamic"), "missing bindDynamic import");
}

#[test]
fn prop_prefix_is_compile_error() {
    let src = r#"<my-grid prop:items={items()}></my-grid>"#;
    let err = compile(src, route_opts()).expect_err("prop: must fail");
    assert!(
        err.message.contains("`prop:`") && err.message.contains("plain"),
        "msg: {}",
        err.message
    );
}

#[test]
fn attr_prefix_is_compile_error() {
    let src = r#"<div attr:hidden={h()}></div>"#;
    let err = compile(src, route_opts()).expect_err("attr: must fail");
    assert!(
        err.message.contains("`attr:`") && err.message.contains("plain"),
        "msg: {}",
        err.message
    );
}

#[test]
fn native_idl_value_emits_bind_prop() {
    let src = r#"<progress value={n()} max="100"></progress>"#;
    let r = compile(src, route_opts()).expect("compile");
    assert!(
        r.js.contains(r#"bindProp(__e0, "value""#),
        "expected bindProp for IDL value, got:\n{}",
        r.js
    );
}

#[test]
fn native_non_idl_emits_bind_attr() {
    let src = r#"<div title={t()}></div>"#;
    let r = compile(src, route_opts()).expect("compile");
    assert!(
        r.js.contains(r#"bindAttr(__e0, "title""#),
        "expected bindAttr, got:\n{}",
        r.js
    );
}

#[test]
fn multiple_ce_fields_each_emit_bind_dynamic() {
    let src = r#"<my-grid rows={rows()} label={title()}></my-grid>"#;
    let r = compile(src, route_opts()).expect("compile");
    let count = r.js.matches("bindDynamic(").count();
    assert_eq!(count, 2, "expected 2 bindDynamic calls, got {count}\n{}", r.js);
}
