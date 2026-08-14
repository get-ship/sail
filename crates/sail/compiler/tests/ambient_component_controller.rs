//! Ambient `signal` on app-local component controllers.
//!
//! Route page controllers and app-local `src/sail-*/` component controllers
//! share the same `<controller>` file shape. Bare `signal()` / `controller()`
//! / `derived()` must compile to a self-contained module with the ambient
//! import injected from `@ship/sail-runtime/internal` — no authored import.
//!
//! Omitting the authored import (lint: redundant ambient) must not leave
//! `ReferenceError: signal is not defined` on app-local custom elements; the
//! serve/materialize path re-wraps after transforms.

use sail_compiler::check::query_keys;
use sail_compiler::parse::preprocess::{extract_controller_block, wrap_controller_body};
use sail_compiler::{CompileMode, CompileOpts, ComponentMeta, compile};
use std::path::PathBuf;

/// App-local CE shape: `src/sail-g2-probe/sail-g2-probe.sail`.
const APP_LOCAL_COMPONENT: &str = r#"
<!-- src/sail-g2-probe/sail-g2-probe.sail — app-local custom element -->
<scope>
  props: {
    label = 'Probe'
  }
</scope>
<controller>
export default controller(() => {
  const count = signal(0)
  const doubled = derived(() => count() * 2)
  return {
    count,
    doubled,
    inc: () => count.set(count() + 1),
  }
})
</controller>
<button on:click={inc}>{label()}: {count()} / {doubled()}</button>
"#;

fn ambient_import_line(body: &str) -> Option<&str> {
    body.lines().find(|l| {
        let t = l.trim_start();
        t.starts_with("import") && t.contains("@ship/sail-runtime/internal")
    })
}

fn asserts_ambient(body: &str, idents: &[&str]) {
    let line = ambient_import_line(body)
        .unwrap_or_else(|| panic!("expected ambient import from /internal, got module:\n{body}"));
    for ident in idents {
        assert!(
            line.contains(ident),
            "ambient `{ident}` missing from inject line `{line}`\nmodule:\n{body}"
        );
    }
}

#[test]
fn app_local_component_controller_gets_ambient_signal() {
    assert!(
        !APP_LOCAL_COMPONENT.contains("from '@ship/sail-runtime'")
            && !APP_LOCAL_COMPONENT.contains("from \"@ship/sail-runtime\""),
        "fixture must not author a runtime import — bare ambient only"
    );
    let (_, body) = extract_controller_block(APP_LOCAL_COMPONENT);
    let body = body.expect("inline <controller>");
    asserts_ambient(&body, &["controller", "signal", "derived"]);
}

#[test]
fn app_local_component_controller_survives_query_keys_then_rewrap() {
    // Mirrors deck sail_plugin load + CLI synthesize: rewrite then ambient wrap.
    let (_, body) = extract_controller_block(APP_LOCAL_COMPONENT);
    let body = body.expect("controller");
    let rewritten = query_keys::rewrite(&body).expect("query_keys");
    let wrapped = wrap_controller_body(&rewritten);
    asserts_ambient(&wrapped, &["controller", "signal", "derived"]);
}

#[test]
fn app_local_component_compile_wires_virtual_controller_setup() {
    let out = compile(
        APP_LOCAL_COMPONENT,
        CompileOpts {
            setup_url: Some("./sail-g2-probe.sail.controller.ts".into()),
            mode: CompileMode::Component,
            component_meta: Some(ComponentMeta {
                name: "sail-g2-probe".into(),
            }),
            source_path: Some(PathBuf::from(
                "apps/eval-ui/src/sail-g2-probe/sail-g2-probe.sail",
            )),
            ..CompileOpts::default()
        },
    )
    .expect("compile app-local component");
    assert!(
        out.js.contains("sail-g2-probe.sail.controller.ts") || out.js.contains("import setup from"),
        "component emit must import the virtual controller module:\n{}",
        out.js
    );
    assert!(
        out.js.contains("component(\"sail-g2-probe\"")
            || out.js.contains("component('sail-g2-probe'"),
        "must register the custom element:\n{}",
        out.js
    );
}

#[test]
fn route_controller_same_shape_also_gets_ambient() {
    // Parity: route/page shape is not special — same wrap path.
    let route = r#"
<controller>
export default controller(() => {
  const n = signal(0)
  return { n }
})
</controller>
<p>{n()}</p>
"#;
    let (_, body) = extract_controller_block(route);
    asserts_ambient(&body.expect("controller"), &["controller", "signal"]);
}
