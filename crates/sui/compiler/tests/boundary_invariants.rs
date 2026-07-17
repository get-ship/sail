//! Boundary invariant I1 — SUI emits STRUCTURE, not BEHAVIOR, and never a
//! serve location.
//!
//! Compiler output is DOM/structure wiring + bare-specifier imports only. It
//! must never:
//!   - hand-roll a scope Proxy inline (call a runtime helper instead — the
//!     `$signals`-undefined regression #149 came from a bespoke inline proxy
//!     that drifted from the runtime's),
//!   - emit a `/_deck/...` URL or an `importmap` (serving is Deck's concern;
//!     SUI knows no URLs).
//!
//! These gates run on representative compiled output (view + component) so a
//! regression fails the build rather than slipping to web-QA.

use sail_compiler::{compile, CompileMode, ComponentMeta, CompileOpts};

fn view_opts(setup: Option<&str>) -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: setup.map(|s| s.to_string()),
        mode: CompileMode::Component,
        source_path: None,
        component_meta: None,
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

fn component_opts(setup: Option<&str>) -> CompileOpts {
    CompileOpts {
        route_params: None,
        setup_url: setup.map(|s| s.to_string()),
        mode: CompileMode::Component,
        source_path: None,
        component_meta: Some(ComponentMeta { name: "sui-x".into() }),
        emit_dts: false,
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}
const FORBIDDEN: &[(&str, &str)] = &[
    ("new Proxy($scope", "hand-rolled scope proxy — call the runtime builder (componentMountState)"),
    ("/_deck/", "a served URL — serving is Deck's concern, SUI emits no URLs"),
    ("importmap", "an import map — that's the host page's job, not compiler output"),
];

fn assert_clean(js: &str, ctx: &str) {
    for (needle, why) in FORBIDDEN {
        assert!(
            !js.contains(needle),
            "I1 violation in {ctx}: emitted JS contains {needle:?} ({why}).\n--- output ---\n{js}",
        );
    }
}

#[test]
fn view_output_emits_no_behavior_or_urls() {
    let out = compile(r#"<p>{x()}</p>"#, view_opts(Some("./controller.ts")))
        .expect("compile view");
    assert_clean(&out.js, "view");
}

#[test]
fn component_output_emits_no_behavior_or_urls() {
    let out = compile(r#"<div>{x()}</div>"#, component_opts(Some("./sui-x.controller.ts")))
        .expect("compile component");
    assert_clean(&out.js, "component");
}

// The render-eval SSR I1 tests (compiler SSR/init contains no `/_deck/`) were
// removed with render-eval: the compiler no longer emits SSR, so there is no
// SSR output to leak a URL into. The I1 invariant is still enforced above on
// the compiled JS (view/component), and the host-injection + has_live_data
// behavior is covered by deck-core's payload tests.
