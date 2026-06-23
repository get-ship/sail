//! Regression: component-mount state must come from the SHARED runtime builder
//! (`componentMountState`), not a proxy hand-rolled into compiler output.
//!
//! History: the compiler used to emit a bespoke `__sp` Proxy inline. It lacked
//! the `$signals` accessor the runtime proxy had, so every controller migrated
//! to `state.$signals.x` (#121) threw at mount — invisible to tsc, caught only
//! by web-QA. The fix unified both mount paths onto one runtime builder; the
//! compiler now emits a CALL to `componentMountState($scope)` and imports it,
//! so the proxy contract can't drift between compiled output and the runtime.

use sui_compiler::{compile, CompileMode, ComponentMeta, CompileOpts};

fn component_opts(setup: Option<&str>) -> CompileOpts {
    CompileOpts {
        setup_url: setup.map(|s| s.to_string()),
        mode: CompileMode::Component,
        source_path: None,
        styles: None,
        component_meta: Some(ComponentMeta {
            name: "sui-test-dock".into(),
            shadow: true,
        }),
        emit_dts: false,
        emit_ssr: false,
        emit_component_imports: false,
    }
}

#[test]
fn component_mount_uses_shared_runtime_state_builder() {
    let src = r#"<div>{x()}</div>"#;
    let out = compile(src, component_opts(Some("./sui-test-dock.controller.ts")))
        .expect("compile component + paired controller");

    // Mount state is built by the shared runtime helper — a CALL, not a
    // hand-rolled proxy in compiler output.
    assert!(
        out.js.contains("const __sp = componentMountState($scope);"),
        "component-mount must call the shared componentMountState builder, got:\n{}",
        out.js
    );
    // The helper must be imported from the runtime.
    assert!(
        out.js.contains("componentMountState") && out.js.contains("@ship/sui-runtime"),
        "componentMountState must be imported from @ship/sui-runtime, got:\n{}",
        out.js
    );
    // No bespoke proxy hand-rolled into the module (the drift vector).
    assert!(
        !out.js.contains("new Proxy($scope"),
        "compiler must not hand-roll a $scope proxy; got:\n{}",
        out.js
    );
    // Mount is wired with the proxy.
    assert!(
        out.js.contains("__lc.mount(__sp, host)"),
        "expected mount(__sp, host) wiring, got:\n{}",
        out.js
    );
}
