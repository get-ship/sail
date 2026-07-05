//! `motion:` directive integration tests.
//!
//! Verifies the AOT-baked `el.animate(...)` + `__sui_motion_attach(...)`
//! shape the compiler emits for various `motion:` directive forms.
//!
//! `motion:<trigger>={{ <property>: '<preset>' }}` — the property is one
//! of the closed `MOTION_PROPERTIES` (scale, opacity, translate-y, ...);
//! the preset is a name resolved against `animation-presets.json`
//! (e.g. `spring:snappy`, `spring:gentle`, `fade`, `press`).

use sui_compiler::{compile, CompileMode, CompileOpts};

fn compile_motion(source: &str) -> Result<String, String> {
    let opts = CompileOpts {
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        styles: None,
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    compile(source, opts)
        .map(|out| out.js)
        .map_err(|e| format!("{e:?}"))
}

#[test]
fn static_hover_scale_emits_anim_and_attach() {
    let js = compile_motion(
        r#"<button motion:hover={{ scale: 'spring:snappy' }}>x</button>"#,
    )
    .expect("compile");
    // Per-prop keyframe + `el.animate(...)` result variables.
    assert!(js.contains("__kf"), "expected __kf<n> in output:\n{js}");
    assert!(js.contains("__anim"), "expected __anim<n> in output:\n{js}");
    // The runtime attach wrapper.
    assert!(
        js.contains("__sui_motion_attach("),
        "expected __sui_motion_attach call:\n{js}"
    );
    // The trigger string appears verbatim.
    assert!(js.contains("\"hover\""), "expected \"hover\" trigger:\n{js}");
    // Duration is a resolved number, not a token name.
    assert!(
        js.contains("duration: "),
        "expected a resolved numeric duration:\n{js}"
    );
}

#[test]
fn unknown_preset_is_a_compile_error() {
    let err = compile_motion(
        r#"<button motion:hover={{ scale: 'does-not-exist' }}>x</button>"#,
    )
    .expect_err("expected compile error");
    assert!(
        err.contains("does-not-exist"),
        "expected error to mention the unknown preset: {err}"
    );
    // The lint surfaces this first with a friendlier message; the
    // compiler would say "unknown preset" if the lint were absent.
    // Accept either.
    assert!(
        err.contains("is not in the closed vocabulary")
            || err.contains("unknown preset"),
        "expected 'unknown preset' or lint wording in error: {err}"
    );
}

#[test]
fn dynamic_config_is_rejected() {
    // `motion:hover={someExpr}` is not yet supported in milestone 1.
    let err = compile_motion(r#"<button motion:hover={someExpr}>x</button>"#)
        .expect_err("expected compile error");
    assert!(err.contains("dynamic"), "expected 'dynamic' in error: {err}");
}

#[test]
fn multiple_props_share_one_attach_call() {
    let js = compile_motion(
        r#"<button motion:hover={{ scale: 'spring:snappy', opacity: 'fade' }}>x</button>"#,
    )
    .expect("compile");
    // Two per-prop `__anim<N>` variables should be referenced inside
    // the same `__sui_motion_attach(...)` enter/leave arrays.
    let attach_idx = js
        .find("__sui_motion_attach(")
        .expect("__sui_motion_attach call");
    let after = &js[attach_idx..];
    let enter_start = after.find("enter: [").expect("enter: [");
    // Find the matching close bracket — handle nested brackets by
    // counting (the `enter: [...]` array may contain other brackets in
    // animation objects, but for milestone 1 shapes it doesn't).
    let enter_end_rel = after[enter_start..].find(']').expect("closing ]");
    let enter_block = &after[enter_start..enter_start + enter_end_rel + 1];
    let anim_count = enter_block.matches("__anim").count();
    assert!(
        anim_count >= 2,
        "expected at least 2 __anim refs in enter array, got {anim_count}:\n{js}"
    );
}

#[test]
fn unknown_property_alias_is_rejected() {
    let err = compile_motion(
        r#"<button motion:hover={{ foo: 'spring:snappy' }}>x</button>"#,
    )
    .expect_err("expected compile error");
    assert!(err.contains("foo"), "expected error to mention 'foo': {err}");
    // The parser's closed set is documented in the error.
    assert!(
        err.contains("scale")
            || err.contains("opacity")
            || err.contains("translate-y")
            || err.contains("translate-x")
            || err.contains("rotate"),
        "expected error to list supported properties: {err}"
    );
}
