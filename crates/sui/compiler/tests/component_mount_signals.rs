//! Regression: a tagged source emits ONE thing — the deferred mount-form module
//! (`const __default = component((ctx) => …)` returning `{ __mount(container) }`)
//! registered via `component(tag, def, __default)`. Setup, render, and the
//! controller lifecycle are unified through the runtime's `mountComponent` —
//! the compiler MUST NOT hand-roll lifecycle wiring (`componentMountState`,
//! `__lc.mount`, a `$scope` proxy) inline anymore.
//!
//! History: the compiler used to render eagerly and wire `componentMountState`
//! + `__lc.mount(__sp, host)` inline (a second engine parallel to the page
//! mount-form). The engine collapse moved all of that to `mountComponent`, so
//! the proxy/lifecycle contract lives in exactly one place (the runtime).

use sui_compiler::{compile, CompileMode, ComponentMeta, CompileOpts};

fn component_opts(setup: Option<&str>) -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: setup.map(|s| s.to_string()),
        mode: CompileMode::Component,
        source_path: None,
        component_meta: Some(ComponentMeta {
            name: "sui-test-dock".into(),

        }),
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

#[test]
fn component_emits_deferred_module_registered_via_component() {
    let src = r#"<div>{x()}</div>"#;
    let out = compile(src, component_opts(Some("./sui-test-dock.controller.ts")))
        .expect("compile component + paired controller");

    // ONE shape: the mount-form module bound to a const, then registered.
    assert!(
        out.js.contains("const __default = Object.assign(component((ctx) =>"),
        "component must emit the mount-form module as __default, got:\n{}",
        out.js
    );
    assert!(
        out.js.contains("component(\"sui-test-dock\", {") && out.js.contains("}, __default);"),
        "component must register via component(tag, def, __default), got:\n{}",
        out.js
    );
    // Render is deferred to `__mount(container)` — the custom-element wrapper
    // calls it on connect; pages/internal mounts call it too.
    assert!(
        out.js.contains("__mount(container)"),
        "component setup must return a deferred __mount(container), got:\n{}",
        out.js
    );
    // Lifecycle wiring moved to the runtime's mountComponent (the one engine).
    // The compiler must NOT hand-roll it inline anymore.
    assert!(
        !out.js.contains("componentMountState")
            && !out.js.contains("__lc.mount")
            && !out.js.contains("new Proxy($scope"),
        "compiler must not inline lifecycle wiring (mountComponent owns it), got:\n{}",
        out.js
    );
}
