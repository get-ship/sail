//! Style `animate: { <trigger>: play(…) }` → `__sail_motion_attach(...)`.
//!
//! Author form is the keyed map. Play names resolve through the theme
//! registry (`fade`, `slide-up`, `colors`). MotionAttach is derived.

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
    let js = compile_motion(r#"<button style({ animate: { hover: play(fade) } })> x</button>"#)
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
        js.contains("property: \"opacity\""),
        "expected property field:\n{js}"
    );
    assert!(js.contains("from: 0"), "expected fade from:\n{js}");
    assert!(js.contains("to: 1"), "expected fade to:\n{js}");
    // No AOT keyframe bake: the emit must not carry a baked `el.animate(...)`.
    assert!(
        !js.contains("el.animate("),
        "no AOT el.animate() — runtime owns animation creation:\n{js}"
    );
}

#[test]
fn unknown_preset_is_a_compile_error() {
    let err = compile_motion(
        r#"<button style({ animate: { hover: play(does-not-exist) } })> x</button>"#,
    )
    .expect_err("expected compile error");
    assert!(
        err.contains("does-not-exist"),
        "expected error to mention the unknown preset: {err}"
    );
    assert!(
        err.contains("unknown vocabulary") || err.contains("unknown preset"),
        "expected unknown-play diagnostic: {err}"
    );
}

/// The whole config cannot be handed in as one opaque expression — the
/// SHAPE is written out so the compiler can see the channels. Individual
/// FIELDS may now be signals; that is the point of this surface, and the
/// error says so rather than the old blanket "dynamic configs".
#[test]
fn whole_config_as_one_expression_is_rejected_and_names_the_surface() {
    // Authored text-level motion channels are unknown vocabulary (Style is product).
    let err = compile_motion(r#"<button fx:hover={someExpr}>x</button>"#)
        .expect_err("expected compile error");
    assert!(
        err.contains("unknown directive") || err.contains("style."),
        "expected unknown-directive / Style pointer: {err}"
    );
    assert!(
        !err.to_ascii_lowercase().contains("retired"),
        "must not embalm fx:: {err}"
    );
}

#[test]
fn multiple_props_share_one_attach_call() {
    let js =
        compile_motion(r#"<button style({ animate: { hover: play(fade-scale-in) } })> x</button>"#)
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

/// Named Style decls (`style name = { …, animate: … }` + `style(name)`) must
/// still emit `__sail_motion_attach`. Pre-fix, `collect_motion_attaches`
/// skipped `Style::Ref`, so every animation-page ambient/loop via a named
/// style painted and never moved.
#[test]
fn named_style_ref_emits_runtime_motion_attach() {
    let js = compile_motion(
        r#"
<scope>
  style ambientDot = { size({ w: 4, h: 4 }).radius(full).bg(primary), animate: { loop: play(slide-up) } }
</scope>
<div style(ambientDot)></div>
"#,
    )
    .expect("compile");
    assert!(
        js.contains("__sail_motion_attach("),
        "named style(ref) must still emit runtime motion attach:\n{js}"
    );
    assert!(js.contains("\"loop\""), "expected loop trigger:\n{js}");
    assert!(
        js.contains("property: \"translate-y\"") || js.contains("property:\"translate-y\""),
        "expected translate-y property:\n{js}"
    );
}

#[test]
fn loop_rotate_spin_emits_infinite_one_direction() {
    let js = compile_motion(r#"<img style({ animate: { loop: play(fade) } })> x</img>"#)
        .expect("compile");
    assert!(
        js.contains("__sail_motion_attach("),
        "expected runtime attach call:\n{js}"
    );
    assert!(js.contains("\"loop\""), "expected trigger string:\n{js}");
    assert!(
        js.contains("property: \"opacity\""),
        "expected property field:\n{js}"
    );
    assert!(js.contains("from: 0"), "expected from field:\n{js}");
    assert!(js.contains("to: 1"), "expected to field:\n{js}");
    assert!(
        js.contains("iterations: Infinity"),
        "expected infinite iterations:\n{js}"
    );
}

#[test]
fn trigger_call_form_is_rejected() {
    let err = compile_motion(
        r#"<button style({ animate: hover({ scale: 'spring:snappy' }) })> x</button>"#,
    )
    .expect_err("expected compile error");
    assert!(
        err.contains("animate:") && err.contains("play"),
        "must teach the play map: {err}"
    );
}

#[test]
fn enter_exit_on_style_chain_with_paint() {
    let js = compile_motion(
        r#"<card style({ pad(4), animate: { enter: play(fade), exit: play(fade) } })> x</card>"#,
    )
    .expect("compile");
    assert!(js.contains(r#"trigger: "enter""#), "{js}");
    assert!(js.contains(r#"trigger: "exit""#), "{js}");
    assert!(js.contains("__sail_motion_attach"), "{js}");
}

#[test]
fn enter_exit_via_animate_channel() {
    let js = compile_motion(
        r#"<card style({ pad(4), animate: { enter: play(fade), exit: play(fade) } })> x</card>"#,
    )
    .expect("compile animate channel");
    assert!(js.contains(r#"trigger: "enter""#), "{js}");
    assert!(js.contains(r#"trigger: "exit""#), "{js}");
    assert!(js.contains("__sail_motion_attach"), "{js}");
}

#[test]
fn fx_directive_is_unknown_vocabulary() {
    let err = compile_motion(r#"<card fx:enter={{ opacity: 'fade' }}>x</card>"#)
        .expect_err("fx: must error");
    assert!(
        err.contains("unknown directive") || err.contains("style."),
        "{err}"
    );
    assert!(!err.to_ascii_lowercase().contains("retired"), "{err}");
}

#[test]
fn authored_motion_attr_is_unknown_vocabulary() {
    let err = compile_motion(r#"<card motion:enter={{ opacity: 'fade' }}>x</card>"#)
        .expect_err("authored motion: must error");
    assert!(
        err.contains("unknown directive") || err.contains("style."),
        "{err}"
    );
    assert!(!err.to_ascii_lowercase().contains("retired"), "{err}");
}
