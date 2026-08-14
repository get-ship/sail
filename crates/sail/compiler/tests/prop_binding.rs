//! Plain `name={expr}` on custom elements emits `bindDynamic`; on native
//! IDL props emits `bindProp`. Author `prop:` / `attr:` prefixes are refused.

use sail_compiler::{CompileMode, CompileOpts, compile};

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
        err.message.contains("unknown directive") && err.message.contains("prop:items"),
        "msg: {}",
        err.message
    );
    assert!(!err.message.to_ascii_lowercase().contains("retired"));
}

#[test]
fn attr_prefix_is_compile_error() {
    let src = r#"<div attr:hidden={h()}></div>"#;
    let err = compile(src, route_opts()).expect_err("attr: must fail");
    assert!(
        err.message.contains("unknown directive") && err.message.contains("attr:hidden"),
        "msg: {}",
        err.message
    );
    assert!(!err.message.to_ascii_lowercase().contains("retired"));
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
fn dialog_open_emits_bind_open_not_prop() {
    // Modal open must use showModal via bindOpen — IDL open prop is non-modal.
    let src = r#"<dialog open={open()}>hi</dialog>"#;
    let r = compile(src, route_opts()).expect("compile");
    assert!(
        r.js.contains("bindOpen(") && !r.js.contains(r#"bindProp(__e0, "open""#),
        "expected bindOpen for dialog open, got:\n{}",
        r.js
    );
    // Bare `open()` call → signal cell for write-back on native close (not a getter thunk).
    assert!(
        r.js.contains("bindOpen(__e0, ($scope.open))") || r.js.contains("bindOpen(__e0, (open))"),
        "expected signal cell stripped of (), got:\n{}",
        r.js
    );
    assert!(
        !r.js.contains("bindOpen(__e0, () =>"),
        "must not wrap open in a getter thunk:\n{}",
        r.js
    );
    // Default / omitted dismissible → two-arg bindOpen (runtime default true).
    assert!(
        !r.js.contains("dismissible: false"),
        "omitted dismissible must not pass opts:\n{}",
        r.js
    );
}

#[test]
fn dialog_dismissible_false_emits_bind_open_opts() {
    // Destructive confirm: author writes dismissible={false}; emit wires
    // bindOpen third-arg opts so ESC/backdrop cannot close.
    let src = r#"<dialog open={open()} dismissible={false}>confirm</dialog>"#;
    let r = compile(src, route_opts()).expect("compile");
    assert!(
        r.js.contains("bindOpen(") && r.js.contains("{ dismissible: false }"),
        "expected bindOpen(..., {{ dismissible: false }}), got:\n{}",
        r.js
    );
    // Signal cell still stripped — opts are the third arg, not a getter wrap.
    assert!(
        r.js.contains("bindOpen(__e0, ($scope.open), { dismissible: false })")
            || r.js
                .contains("bindOpen(__e0, (open), { dismissible: false })"),
        "expected three-arg bindOpen with signal cell + opts, got:\n{}",
        r.js
    );
}

#[test]
fn dialog_dismissible_true_stays_two_arg_bind_open() {
    let src = r#"<dialog open={open()} dismissible={true}>ok</dialog>"#;
    let r = compile(src, route_opts()).expect("compile");
    assert!(
        r.js.contains("bindOpen(") && !r.js.contains("dismissible: false"),
        "true must not emit false opts:\n{}",
        r.js
    );
    assert!(
        !r.js.contains("{ dismissible:"),
        "true stays two-arg (runtime default), got:\n{}",
        r.js
    );
}

#[test]
fn dialog_dismissible_static_false_emits_opts() {
    let src = r#"<dialog open={open()} dismissible="false">confirm</dialog>"#;
    let r = compile(src, route_opts()).expect("compile");
    assert!(
        r.js.contains("{ dismissible: false }"),
        "static dismissible=\"false\" must wire opts:\n{}",
        r.js
    );
}

#[test]
fn details_open_still_bind_prop() {
    let src = r#"<details open={o()}><summary>x</summary></details>"#;
    let r = compile(src, route_opts()).expect("compile");
    assert!(
        r.js.contains(r#"bindProp(__e0, "open""#),
        "details open stays IDL prop, got:\n{}",
        r.js
    );
}

#[test]
fn selected_on_div_is_attr_not_idl_prop() {
    let src = r#"<div selected={on()}></div>"#;
    let r = compile(src, route_opts()).expect("compile");
    assert!(
        r.js.contains(r#"bindAttr(__e0, "selected""#),
        "div.selected is not IDL — need attribute for CSS [selected], got:\n{}",
        r.js
    );
    assert!(!r.js.contains(r#"bindProp(__e0, "selected""#));
}

#[test]
fn selected_on_option_is_idl_prop() {
    let src = r#"<option selected={on()}></option>"#;
    let r = compile(src, route_opts()).expect("compile");
    assert!(
        r.js.contains(r#"bindProp(__e0, "selected""#),
        "option.selected is IDL, got:\n{}",
        r.js
    );
}

#[test]
fn multiple_ce_fields_each_emit_bind_dynamic() {
    let src = r#"<my-grid rows={rows()} label={title()}></my-grid>"#;
    let r = compile(src, route_opts()).expect("compile");
    let count = r.js.matches("bindDynamic(").count();
    assert_eq!(
        count, 2,
        "expected 2 bindDynamic calls, got {count}\n{}",
        r.js
    );
}

#[test]
fn freeform_chip_stamps_prop_defaults() {
    let src = r#"
<div>
  <chip size="sm">x</chip>
</div>
"#;
    let r = compile(src, route_opts()).expect("compile");
    // Chip tone is opt-in (no default) — omitted when the author does not set it.
    assert!(
        !r.js.contains(r#"tone="default""#) && !r.js.contains(r#"tone=\"default\""#),
        "chip must not invent a default tone:\n{}",
        r.js
    );
    assert!(
        r.js.contains(r#"variant="solid""#) || r.js.contains(r#"variant=\"solid\""#),
        "expected default variant stamped:\n{}",
        r.js
    );
    assert!(
        r.js.contains(r#"size="sm""#) || r.js.contains(r#"size=\"sm\""#),
        "size=sm preserved:\n{}",
        r.js
    );
}

#[test]
fn on_callback_prop_emits_bind_prop_on_native_host() {
    // Freeform L1 hosts are native tags; on-* must still bind as properties.
    let src = r#"<div on-remove={drop}></div>"#;
    let r = compile(src, route_opts()).expect("compile");
    assert!(
        r.js.contains(r#"bindProp(__e0, "onRemove""#)
            || r.js.contains(r#"bindProp(__e0, "onRemove","#),
        "expected bindProp for on-remove callback, got:\n{}",
        r.js
    );
    assert!(!r.js.contains(r#"bindAttr(__e0, "on-remove""#));
}

#[test]
fn custom_event_on_handler_is_refused() {
    let src = r#"<chip removable on:sail-remove={drop}>x</chip>"#;
    let err = compile(src, route_opts()).expect_err("on:sail-remove must fail");
    assert!(
        err.message.contains("sail-remove")
            && (err.message.contains("on-remove") || err.message.contains("callback")),
        "msg: {}",
        err.message
    );
}

#[test]
fn form_success_event_is_refused() {
    let src = r#"<form mutation="demo.x" on:success={() => {}}></form>"#;
    let err = compile(src, route_opts()).expect_err("on:success must fail");
    assert!(
        err.message.contains("success")
            && (err.message.contains("on-success") || err.message.contains("callback")),
        "msg: {}",
        err.message
    );
}
