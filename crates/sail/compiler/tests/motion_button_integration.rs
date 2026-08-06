#[test]
fn sail_button_with_motion_compiles() {
    use sail_compiler::{CompileMode, CompileOpts, compile};
    // sail-button dissolved to the bare L1 <button> — motion is a LANGUAGE
    // feature on any element, so this integration check uses an inline
    // component with a motion directive (the old file-read target is gone).
    let source = "<button>\n  style({ animate: hover({ scale: 'spring:snappy' }) })\n  Go\n</button>";
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
    let out = compile(source, opts).expect("compile button with motion");
    // The motion directive should have produced a __sail_motion_attach
    // call in the emit.
    assert!(
        out.js.contains("__sail_motion_attach"),
        "expected __sail_motion_attach in compiled JS:\n{}",
        out.js
    );
    assert!(
        out.js.contains("\"hover\""),
        "expected \"hover\" trigger in compiled JS"
    );
    // Runtime dispatch: motion.dev creates the animation at runtime;
    // no AOT `el.animate(...)` is emitted. The runtime owns reduced
    // motion + WAAPI orchestration in one place.
    assert!(
        !out.js.contains("el.animate("),
        "runtime owns animation creation — no AOT el.animate() should leak:\n{}",
        out.js
    );
}
