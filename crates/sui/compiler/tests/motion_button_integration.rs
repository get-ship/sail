#[test]
fn sui_button_with_motion_compiles() {
    use sui_compiler::{compile, CompileMode, CompileOpts};
    // sui-button dissolved to the bare L1 <button> — motion is a LANGUAGE
    // feature on any element, so this integration check uses an inline
    // component with a motion directive (the old file-read target is gone).
    let source = "<button motion:hover={{ scale: 'spring:snappy' }}>Go</button>";
    let opts = CompileOpts {
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    let out = compile(source, opts).expect("compile button with motion");
    // The motion directive should have produced a __sui_motion_attach
    // call in the emit.
    assert!(
        out.js.contains("__sui_motion_attach"),
        "expected __sui_motion_attach in compiled JS:\n{}",
        out.js
    );
    assert!(
        out.js.contains("\"hover\""),
        "expected \"hover\" trigger in compiled JS"
    );
    // Sanity: no raw numerical spring config — the AOT bake
    // replaces it with keyframes.
    assert!(
        !out.js.contains("stiffness:"),
        "AOT bake should not leak raw spring config to JS"
    );
}
