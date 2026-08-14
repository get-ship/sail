//! `animate: { <trigger>: play(…) }` — the one author grammar.
//!
//! Play names resolve through the theme registry. MotionAttach is derived
//! from the play (keyframe → `{ opacity: { from, to }, … }`). Retired
//! trigger-call / raw-object author forms are refused.

use sail_compiler::{CompileMode, CompileOpts, compile};

fn compile_motion(source: &str) -> Result<String, String> {
    let opts = CompileOpts {
        scope_capabilities: Vec::new(),
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    compile(source, opts)
        .map(|out| out.js)
        .map_err(|e| format!("{e:?}"))
}

fn ok(source: &str) -> String {
    compile_motion(source).unwrap_or_else(|e| panic!("expected `{source}` to compile, got: {e}"))
}

fn err(source: &str) -> String {
    compile_motion(source).expect_err(&format!("expected `{source}` to be rejected"))
}

#[test]
fn play_fade_enter_emits_opacity_from_to() {
    let js = ok(r#"<card style({ animate: { enter: play(fade) } })> x</card>"#);
    assert!(js.contains("__sail_motion_attach"), "{js}");
    assert!(js.contains(r#"trigger: "enter""#), "{js}");
    assert!(js.contains("property: \"opacity\""), "{js}");
    assert!(js.contains("from: 0"), "{js}");
    assert!(js.contains("to: 1"), "{js}");
}

#[test]
fn play_fade_loop_is_infinite() {
    let js = ok(r#"<card style({ animate: { loop: play(fade) } })> x</card>"#);
    assert!(js.contains("iterations: Infinity"), "{js}");
    assert!(js.contains(r#"trigger: "loop""#), "{js}");
}

#[test]
fn play_slide_up_expands_opacity_and_translate() {
    let js = ok(r#"<card style({ animate: { enter: play(slide-up) } })> x</card>"#);
    assert!(js.contains("property: \"opacity\""), "{js}");
    assert!(
        js.contains("property: \"translate-y\"") || js.contains("property:\"translate-y\""),
        "{js}"
    );
}

#[test]
fn play_fade_scale_in_expands_opacity_and_scale() {
    let js = ok(r#"<card style({ animate: { enter: play(fade-scale-in) } })> x</card>"#);
    assert!(js.contains("property: \"opacity\""), "{js}");
    assert!(js.contains("property: \"scale\""), "{js}");
}

#[test]
fn unknown_play_is_rejected() {
    let e = err(r#"<card style({ animate: { loop: play(notARealPlayZzz) } })> x</card>"#);
    assert!(
        e.contains("notARealPlayZzz") && e.contains("unknown vocabulary"),
        "{e}"
    );
}

#[test]
fn trigger_call_form_is_rejected() {
    let e = err(r#"<card style({ animate: loop({ rotate: 'spin' }) })> x</card>"#);
    assert!(
        e.contains("animate:") && e.contains("play"),
        "must teach the play map: {e}"
    );
}

#[test]
fn a_fully_literal_config_stays_a_plain_object() {
    let js = ok(r#"<card style({ animate: { loop: play(fade) } })> x</card>"#);
    assert!(
        js.contains("__sail_motion_attach") && !js.contains("() => ({ trigger"),
        "a literal config should stay a plain object:\n{js}"
    );
}

#[test]
fn the_motion_disposer_is_registered_for_cleanup() {
    let js = ok(r#"<card style({ animate: { loop: play(fade) } })> x</card>"#);
    assert!(
        js.contains(".push(__sail_motion_attach("),
        "the disposer must be registered on the cleanup list:\n{js}"
    );
}

#[test]
fn transition_play_is_css_not_runtime_attach() {
    let js = ok(r#"<card style({ animate: { transition: play(colors, fast) } })> x</card>"#);
    assert!(
        !js.contains("__sail_motion_attach"),
        "transition play is CSS emit, not a runtime trigger:\n{js}"
    );
}
