#[test]
fn sui_button_with_motion_compiles() {
    use sui_compiler::{compile, CompileMode, CompileOpts};
    // Repo-relative so it survives worktree moves (was a hardcoded absolute
    // path to the old sui-motion worktree, which broke on reorg).
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../packages/components/sui/primitives/src/sui-button/sui-button.sui");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let opts = CompileOpts {
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(std::path::PathBuf::from("sui-button.sui")),
        styles: None,
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    };
    let out = compile(&source, opts).expect("compile sui-button");
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
