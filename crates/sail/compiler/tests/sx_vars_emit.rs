//! `(--x)` binding integration tests — the resolution law's emit side.
//!
//! Pins: a matched prop/signal gets a CSS `@property` registration + host
//! reflection (seed at mount + on every write); an unmatched name refuses
//! with a did-you-mean; a signal not returned from `setup()` is not
//! reachable; a global token is unaffected (no component-local wiring).

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
fn a_prop_bound_var_ref_emits_property_and_host_reflection() {
    let js = compile_component(
        r#"<scope>
  props: { glowRadius: number = 0 }
</scope>
<div style({ box: w(bind(glowRadius)) })> </div>"#,
        None,
    )
    .expect("compile");
    assert!(js.contains("registerCustomProperties("), "{js}");
    assert!(js.contains("@property --glow-radius"), "{js}");
    assert!(js.contains("syntax: '<number>'"), "{js}");
    assert!(js.contains("$scope.glowRadius()"), "{js}");
    assert!(
        js.contains("__root.style.setProperty(\"--glow-radius\""),
        "{js}"
    );
    assert!(js.contains("effect("), "{js}");
}

#[test]
fn an_action_owned_var_ref_is_unaffected_like_a_token() {
    // `--row-y` is written by `use:virtualRow`, not declared by this
    // component as a prop or signal — the resolution law's token bucket
    // (via `sail_lint::tokens::CONVENTION_TOKENS`) covers action-owned
    // custom properties for exactly this reason: the ACTION owns the
    // name, so it gets no component-local `@property`/host-reflection
    // wiring and is never refused as unmatched.
    let js = compile_component(
        r#"<div use:virtualRow={0} style({ paint: translateY(bind(rowY)) })> </div>"#,
        None,
    )
    .expect("compile");
    assert!(!js.contains("registerCustomProperties("), "{js}");
    assert!(!js.contains("@property"), "{js}");
}

#[test]
fn a_setup_returned_signal_var_ref_emits_property_and_host_reflection() {
    let js = compile_component(
        r#"<div style({ paint: bg(bind(hue)) })> </div>"#,
        Some("controller((ctx) => { const hue = signal(220); return { hue }; })"),
    )
    .expect("compile");
    assert!(js.contains("registerCustomProperties("), "{js}");
    assert!(js.contains("@property --hue"), "{js}");
    assert!(js.contains("$scope.hue()"), "{js}");
    assert!(js.contains("__root.style.setProperty(\"--hue\""), "{js}");
}

#[test]
fn a_global_token_var_ref_emits_no_component_local_wiring() {
    let js = compile_component(r#"<div style({ paint: bg(surface-2) })> </div>"#, None).expect("compile");
    assert!(!js.contains("registerCustomProperties("), "{js}");
    assert!(!js.contains("@property"), "{js}");
}

#[test]
fn an_unmatched_var_ref_refuses_with_did_you_mean() {
    let err = compile_component(
        r#"<scope>
  props: { glowRadius: number = 0 }
</scope>
<div style({ box: top(bind(glowRadiuz)) })> </div>"#,
        None,
    )
    .unwrap_err();
    assert!(err.contains("does not match"), "{err}");
    assert!(err.contains("glow-radius"), "{err}");
}

#[test]
fn a_signal_not_returned_from_setup_is_not_reachable() {
    let err = compile_component(
        r#"<div style({ paint: bg(bind(hue)) })> </div>"#,
        Some("controller((ctx) => { const hue = signal(220); return {}; })"),
    )
    .unwrap_err();
    assert!(err.contains("does not match"), "{err}");
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
