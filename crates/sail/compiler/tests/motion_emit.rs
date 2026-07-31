//! Style motion chain methods → `__sail_motion_attach(...)` integration tests.
//!
//! Author form: `sx={style.… .enter({…}) / .hover({…}) / .loop({…}) …}`.
//! Config body grammar is identical to the retired `fx:<trigger>={{…}}`.
//! Property is one of the closed `MOTION_PROPERTIES`; preset resolves against
//! `animation-presets.json` (e.g. `spring:snappy`, `fade`, `press`).

use sail_compiler::{CompileMode, CompileOpts, compile};

fn compile_motion(source: &str) -> Result<String, String> {
    let opts = CompileOpts {
        route_params: None,
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: None,
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    compile(source, opts)
        .map(|out| out.js)
        .map_err(|e| format!("{e:?}"))
}

#[test]
fn static_hover_scale_emits_runtime_dispatch() {
    let js = compile_motion(r#"<button sx={style.hover({ scale: 'spring:snappy' })}>x</button>"#)
        .expect("compile");
    // Runtime dispatch: the per-property `MotionConfig` is embedded as
    // a literal object, then handed to `__sail_motion_attach`. No AOT
    // `el.animate(...)` is emitted — motion.dev owns the animation
    // creation at runtime (one path, one reduced-motion authority).
    assert!(
        js.contains("__sail_motion_attach("),
        "expected runtime attach call:\n{js}"
    );
    assert!(js.contains("\"hover\""), "expected trigger string:\n{js}");
    assert!(
        js.contains("property: \"scale\""),
        "expected property field:\n{js}"
    );
    assert!(
        js.contains("preset: \"spring:snappy\""),
        "expected preset field:\n{js}"
    );
    assert!(
        js.contains("kind: \"spring\""),
        "expected kind field:\n{js}"
    );
    assert!(
        js.contains("stiffness: 380"),
        "expected stiffness field:\n{js}"
    );
    assert!(js.contains("damping: 30"), "expected damping field:\n{js}");
    // No AOT keyframe bake: the emit must not carry a baked `el.animate(...)`.
    assert!(
        !js.contains("el.animate("),
        "no AOT el.animate() — runtime owns animation creation:\n{js}"
    );
}

#[test]
fn unknown_preset_is_a_compile_error() {
    let err = compile_motion(r#"<button sx={style.hover({ scale: 'does-not-exist' })}>x</button>"#)
        .expect_err("expected compile error");
    assert!(
        err.contains("does-not-exist"),
        "expected error to mention the unknown preset: {err}"
    );
    assert!(
        err.contains("is not in the closed vocabulary") || err.contains("unknown preset"),
        "expected 'unknown preset' or lint wording in error: {err}"
    );
}

/// The whole config cannot be handed in as one opaque expression — the
/// SHAPE is written out so the compiler can see the channels. Individual
/// FIELDS may now be signals; that is the point of this surface, and the
/// error says so rather than the old blanket "dynamic configs".
#[test]
fn whole_config_as_one_expression_is_rejected_and_names_the_fix() {
    // Public fx: is retired with a migration error (not the old dynamic-config wording).
    let err = compile_motion(r#"<button fx:hover={someExpr}>x</button>"#)
        .expect_err("expected compile error");
    assert!(
        err.contains("retired") || err.contains("style."),
        "expected fx: retirement / Style migration wording: {err}"
    );
}

#[test]
fn multiple_props_share_one_attach_call() {
    let js = compile_motion(
        r#"<button sx={style.hover({ scale: 'spring:snappy', opacity: 'fade' })}>x</button>"#,
    )
    .expect("compile");
    let attach_idx = js
        .find("__sail_motion_attach(")
        .expect("__sail_motion_attach call");
    let after = &js[attach_idx..];
    let motions_start = after.find("motions: [").expect("motions: [");
    let motions_end_rel = after[motions_start..].find(']').expect("closing ]");
    let motions_block = &after[motions_start..motions_start + motions_end_rel + 1];
    // Two property fields = two motions in the array.
    let prop_count = motions_block.matches("property:").count();
    assert!(
        prop_count >= 2,
        "expected at least 2 property fields in motions array, got {prop_count}:\n{js}"
    );
}

#[test]
fn loop_rotate_spin_emits_infinite_one_direction() {
    let js =
        compile_motion(r#"<img sx={style.loop({ rotate: 'spin' })}>x</img>"#).expect("compile");
    assert!(
        js.contains("__sail_motion_attach("),
        "expected runtime attach call:\n{js}"
    );
    assert!(js.contains("\"loop\""), "expected trigger string:\n{js}");
    assert!(
        js.contains("property: \"rotate\""),
        "expected property field:\n{js}"
    );
    assert!(
        js.contains("preset: \"spin\""),
        "expected preset field:\n{js}"
    );
    assert!(js.contains("kind: \"tween\""), "expected kind field:\n{js}");
    assert!(js.contains("from: 0"), "expected from field:\n{js}");
    assert!(js.contains("to: 360"), "expected to field:\n{js}");
    assert!(
        js.contains("duration: 14"),
        "expected a 14s duration baked from the glacial token:\n{js}"
    );
    assert!(
        js.contains("easing: \"linear\""),
        "expected linear easing:\n{js}"
    );
    assert!(
        js.contains("iterations: Infinity"),
        "expected infinite iterations:\n{js}"
    );
    // NOT alternate — a spin goes one way, it does not ping-pong.
    assert!(
        js.contains("direction: \"normal\""),
        "expected one-direction spin:\n{js}"
    );
}

#[test]
fn unknown_property_alias_is_rejected() {
    let err = compile_motion(r#"<button sx={style.hover({ foo: 'spring:snappy' })}>x</button>"#)
        .expect_err("expected compile error");
    assert!(
        err.contains("foo"),
        "expected error to mention 'foo': {err}"
    );
    assert!(
        err.contains("scale")
            || err.contains("opacity")
            || err.contains("translate-y")
            || err.contains("translate-x")
            || err.contains("rotate"),
        "expected error to list supported properties: {err}"
    );
}

#[test]
fn enter_exit_on_style_chain_with_paint() {
    let js = compile_motion(
        r#"<card sx={style.pad(4).enter({ opacity: 'fade' }).exit({ opacity: 'fade-out' })}>x</card>"#,
    )
    .expect("compile");
    assert!(js.contains(r#"trigger: "enter""#), "{js}");
    assert!(js.contains(r#"trigger: "exit""#), "{js}");
    assert!(js.contains("__sail_motion_attach"), "{js}");
}

#[test]
fn fx_directive_is_a_compile_error_naming_style_methods() {
    let err = compile_motion(r#"<card fx:enter={{ opacity: 'fade' }}>x</card>"#)
        .expect_err("fx: must error");
    assert!(err.contains("retired"), "{err}");
    assert!(err.contains(".enter") || err.contains("style."), "{err}");
}
