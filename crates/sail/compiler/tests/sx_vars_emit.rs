//! `bind(name)` binding integration tests — the resolution law's emit side.
//!
//! Pins: a matched prop/signal gets a CSS `@property` registration + host
//! reflection (seed at mount + on every write); an unmatched name refuses
//! with a did-you-mean; a signal not returned from `setup()` is not
//! reachable; a global token is unaffected (no component-local wiring);
//! action writes resolve without reflection; number→length realizes as px.

use sail_compiler::{CompileMode, CompileOpts, ComponentMeta, compile};

fn compile_component(source: &str, controller: Option<&str>) -> Result<String, String> {
    let mut src = String::new();
    if let Some(c) = controller {
        src.push_str("<controller>\n");
        src.push_str(c);
        src.push_str("\n</controller>\n");
    }
    src.push_str(source);
    let opts = CompileOpts {
        route_params: None,
        setup_url: Some("./view.ts".to_string()),
        mode: CompileMode::Component,
        source_path: None,
        component_meta: Some(ComponentMeta {
            name: "x-widget".to_string(),
        }),
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    compile(&src, opts)
        .map(|out| out.js)
        .map_err(|e| format!("{e:?}"))
}

#[test]
fn a_prop_bound_var_ref_emits_length_property_and_host_reflection() {
    let js = compile_component(
        r#"<scope>
  props: { glowRadius: number = 0 }
</scope>
<div style({ size({ w: bind(glowRadius) }) })> </div>"#,
        None,
    )
    .expect("compile");
    assert!(js.contains("registerCustomProperties("), "{js}");
    assert!(js.contains("@property --glow-radius"), "{js}");
    assert!(
        js.contains("syntax: '<length>'") || js.contains("syntax: '<number>'"),
        "{js}"
    );
    assert!(
        js.contains("initial-value: 0px") || js.contains("initial-value: 0"),
        "{js}"
    );
    assert!(js.contains("$scope.glowRadius()"), "{js}");
    assert!(
        js.contains("__root.style.setProperty(\"--glow-radius\""),
        "{js}"
    );
    assert!(
        js.contains("__v + \"px\"") || js.contains("+ \"px\"") || js.contains("String(__v)"),
        "number bind reflects onto the host custom property:\n{js}"
    );
    assert!(js.contains("effect("), "{js}");
}

#[test]
fn an_action_owned_var_ref_is_unaffected_like_a_token() {
    // `--row-y` is written by `use:virtualRow` — resolved via action `writes`,
    // no component-local `@property`/host-reflection wiring.
    let js = compile_component(
        r#"<div use:virtualRow={0} style({ translateY(bind(rowY)) })> </div>"#,
        None,
    )
    .expect("compile");
    assert!(!js.contains("registerCustomProperties("), "{js}");
    assert!(!js.contains("@property"), "{js}");
}

#[test]
fn action_write_without_use_refuses() {
    let err =
        compile_component(r#"<div style({ translateY(bind(rowY)) })> </div>"#, None).unwrap_err();
    assert!(
        err.contains("bind(rowY)") || err.contains("does not match"),
        "{err}"
    );
}

#[test]
fn a_setup_returned_signal_var_ref_emits_property_and_host_reflection() {
    let js = compile_component(
        r#"<div style({ opacity(bind(fade)) })> </div>"#,
        Some("controller((ctx) => { const fade = signal(0.5); return { fade }; })"),
    )
    .expect("compile");
    assert!(js.contains("registerCustomProperties("), "{js}");
    assert!(js.contains("@property --fade"), "{js}");
    assert!(js.contains("syntax: '<number>'"), "{js}");
    assert!(js.contains("$scope.fade()"), "{js}");
    assert!(js.contains("__root.style.setProperty(\"--fade\""), "{js}");
}

#[test]
fn derived_annotated_number_binds() {
    let js = compile_component(
        r#"<div style({ position({ t: bind(doubled) }) })> </div>"#,
        Some(
            "controller((ctx) => { const n = signal(8); const doubled = derived((): number => n() * 2); return { doubled }; })",
        ),
    )
    .expect("compile");
    assert!(js.contains("@property --doubled"), "{js}");
    assert!(
        js.contains("syntax: '<length>'") || js.contains("syntax: '<number>'"),
        "{js}"
    );
    assert!(js.contains("$scope.doubled()"), "{js}");
}

#[test]
fn number_to_color_refuses() {
    let err = compile_component(
        r#"<scope>
  props: { n: number = 0 }
</scope>
<div style({ bg(bind(n)) })> </div>"#,
        None,
    )
    .unwrap_err();
    assert!(
        err.contains("number") && (err.contains("color") || err.contains("bg")),
        "{err}"
    );
}

#[test]
fn a_global_token_var_ref_emits_no_component_local_wiring() {
    let js = compile_component(r#"<div style({ bg(surface-2) })> </div>"#, None).expect("compile");
    assert!(!js.contains("registerCustomProperties("), "{js}");
    assert!(!js.contains("@property"), "{js}");
}

#[test]
fn an_unmatched_var_ref_refuses_with_did_you_mean() {
    let err = compile_component(
        r#"<scope>
  props: { glowRadius: number = 0 }
</scope>
<div style({ position({ t: bind(glowRadiuz) }) })> </div>"#,
        None,
    )
    .unwrap_err();
    assert!(
        err.contains("does not match") || err.contains("bind("),
        "{err}"
    );
    assert!(
        err.contains("glowRadius") || err.contains("glow-radius"),
        "{err}"
    );
}

#[test]
fn a_signal_not_returned_from_setup_is_not_reachable() {
    let err = compile_component(
        r#"<div style({ opacity(bind(hue)) })> </div>"#,
        Some("controller((ctx) => { const hue = signal(220); return {}; })"),
    )
    .unwrap_err();
    assert!(
        err.contains("does not match") || err.contains("bind(hue)"),
        "{err}"
    );
}

#[test]
fn a_prop_never_read_via_var_ref_gets_no_wiring() {
    let js = compile_component(
        r#"<scope>
  props: { glowRadius: number = 0 }
</scope>
<div>{glowRadius()}</div>"#,
        None,
    )
    .expect("compile");
    assert!(!js.contains("registerCustomProperties("), "{js}");
    assert!(!js.contains("@property"), "{js}");
}
