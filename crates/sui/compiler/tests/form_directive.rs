//! `<prop form-value />` + `<prop form-disabled />` integration tests.
//!
//! Covers parse, validation, and JS / CEM / schema emit. Runtime
//! wiring is exercised in
//! `packages/sui/runtime/test/form-associated-prop.test.ts`.

use sui_compiler::parse::parse;
use sui_compiler::{compile, CompileMode, CompileOpts, ComponentMeta};

fn component_opts(name: &str) -> CompileOpts {
    CompileOpts {
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: Some(ComponentMeta { name: name.into(), shadow: false }),
        emit_dts: true,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

fn expect_err(src: &str) -> sui_compiler::CompileError {
    compile(src, component_opts("x-el")).expect_err("expected compile error")
}

#[test]
fn parses_form_value_flag() {
    let src = r#"<prop name="value" type="string" default="" form-value /><span>x</span>"#;
    let program = parse(src).expect("parse");
    let p = program.props.iter().find(|p| p.name == "value").expect("prop");
    assert!(p.form_value);
    assert!(!p.form_disabled);
}

#[test]
fn parses_form_disabled_flag() {
    let src =
        r#"<prop name="disabled" type="bool" default="false" form-disabled /><span>x</span>"#;
    let program = parse(src).expect("parse");
    let p = program.props.iter().find(|p| p.name == "disabled").expect("prop");
    assert!(p.form_disabled);
    assert!(!p.form_value);
}

#[test]
fn absent_flags_parse_unflagged() {
    let src = r#"<prop name="v" type="string" /><span>x</span>"#;
    let program = parse(src).expect("parse");
    let p = &program.props[0];
    assert!(!p.form_value);
    assert!(!p.form_disabled);
}

#[test]
fn two_form_value_props_is_error() {
    let src = r#"
        <prop name="a" type="string" form-value />
        <prop name="b" type="string" form-value />
        <span>x</span>
    "#;
    let err = expect_err(src);
    assert!(
        err.message.contains("at most one form-value"),
        "msg: {}",
        err.message,
    );
}

#[test]
fn form_value_on_array_is_error() {
    let src = r#"<prop name="v" type="array" form-value /><span>x</span>"#;
    let err = expect_err(src);
    assert!(
        err.message.contains("form-value"),
        "expected form-value diagnostic, got: {}",
        err.message,
    );
}

#[test]
fn form_value_on_object_is_error() {
    let src = r#"<prop name="v" type="object" form-value /><span>x</span>"#;
    let err = expect_err(src);
    assert!(
        err.message.contains("form-value"),
        "expected form-value diagnostic, got: {}",
        err.message,
    );
}

#[test]
fn form_disabled_on_string_is_error() {
    let src = r#"<prop name="d" type="string" form-disabled /><span>x</span>"#;
    let err = expect_err(src);
    assert!(
        err.message.contains("form-disabled"),
        "expected form-disabled diagnostic, got: {}",
        err.message,
    );
    assert!(err.message.contains("bool"), "msg: {}", err.message);
}

#[test]
fn js_emits_form_associated_for_form_value() {
    let src = r#"<prop name="value" type="string" default="" form-value /><span>x</span>"#;
    let out = compile(src, component_opts("x-el")).expect("compile");
    assert!(
        out.js.contains("formAssociated: true"),
        "js missing formAssociated: true\n{}",
        out.js,
    );
    assert!(
        out.js.contains("formValue: \"value\""),
        "js missing formValue: \"value\"\n{}",
        out.js,
    );
}

#[test]
fn js_emits_form_disabled_propname() {
    let src =
        r#"<prop name="disabled" type="bool" default="false" form-disabled /><span>x</span>"#;
    let out = compile(src, component_opts("x-el")).expect("compile");
    assert!(out.js.contains("formAssociated: true"), "{}", out.js);
    assert!(out.js.contains("formDisabled: \"disabled\""), "{}", out.js);
}

#[test]
fn js_omits_form_fields_when_no_flags() {
    let src = r#"<prop name="x" type="string" /><span>x</span>"#;
    let out = compile(src, component_opts("x-el")).expect("compile");
    assert!(!out.js.contains("formAssociated"), "{}", out.js);
    assert!(!out.js.contains("formValue"), "{}", out.js);
    assert!(!out.js.contains("formDisabled"), "{}", out.js);
}

#[test]
fn cem_marks_form_associated() {
    let src = r#"<prop name="value" type="string" form-value /><span>x</span>"#;
    let out = compile(src, component_opts("x-el")).expect("compile");
    let cem = out.cem.expect("cem");
    let decl = &cem["modules"][0]["declarations"][0];
    assert_eq!(decl["formAssociated"], serde_json::Value::Bool(true));
    let attrs = decl["attributes"].as_array().expect("attrs");
    let value_attr = attrs.iter().find(|a| a["name"] == "value").expect("value attr");
    assert_eq!(value_attr["formValue"], serde_json::Value::Bool(true));
}

#[test]
fn schema_marks_form_associated() {
    let src = r#"<prop name="value" type="string" form-value /><span>x</span>"#;
    let out = compile(src, component_opts("x-el")).expect("compile");
    let schema = out.schema.expect("schema");
    let component = &schema["component"];
    assert_eq!(component["formAssociated"], serde_json::Value::Bool(true));
    let props = component["props"].as_array().expect("props");
    let value_prop = props
        .iter()
        .find(|p| p["name"] == "value")
        .expect("value prop");
    assert_eq!(value_prop["formValue"], serde_json::Value::Bool(true));
}

#[test]
fn dts_exports_form_associated_when_flagged() {
    let src =
        r#"<prop name="value" type="string" default="" form-value /><span>x</span>"#;
    let out = compile(src, component_opts("x-el")).expect("compile");
    let dts = out.dts.expect("dts");
    assert!(
        dts.contains("export const formAssociated: true"),
        "dts missing formAssociated export\n{dts}",
    );
}

#[test]
fn dts_omits_form_associated_when_unflagged() {
    let src = r#"<prop name="value" type="string" default="" /><span>x</span>"#;
    let out = compile(src, component_opts("x-el")).expect("compile");
    let dts = out.dts.expect("dts");
    assert!(
        !dts.contains("formAssociated"),
        "dts should not declare formAssociated\n{dts}",
    );
}
