//! Gate tests for SPEC-PROJECT-STRUCTURE-1 §4 — block order is grammar.
//!
//! Spike S1.3 red gate. Calls product `compile()` only.
//! Implementer must not modify, skip, or weaken these tests.
//!
//! Expected product surface:
//! - `compile(source, opts)` is a **parse error** when both `<scope>` and
//!   `<controller>` appear and `<controller>` precedes `<scope>`
//! - Teaching error names the legal order (`<scope>` before `<controller>`)
//! - Legal order (scope → controller → template) compiles
//! - Not a lint tier — must fail `compile()` (no dual-path / warn-only)

use std::path::PathBuf;

use sail_compiler::{CompileMode, CompileOpts, ComponentMeta, compile};

fn component_opts(name: &str) -> CompileOpts {
    CompileOpts {
        element_prefix: "sail".into(),
        host_data_attr: "data-sail-host".into(),
        route_params: None,
        setup_url: Some(format!("./{name}.sail.controller.ts")),
        mode: CompileMode::Component,
        source_path: Some(PathBuf::from(format!("/x/{name}.sail"))),
        component_meta: Some(ComponentMeta {
            name: name.to_string(),
        }),
        emit_dts: false,
        scope_capabilities: Vec::new(),
        emit_component_imports: false,
        lint_overrides: Vec::new(),
    }
}

/// Minimal legal SFC: scope, then controller, then template.
/// Controller returns real state so empty-controller lint does not fire.
const SCOPE_THEN_CONTROLLER: &str = r#"<scope>
  props: {
    label = 'ok'
  }
</scope>
<controller>
export default controller(() => {
  const ticks = signal(0)
  return { ticks }
})
</controller>
<div>{label}: {ticks()}</div>
"#;

/// Illegal SFC: controller appears before scope (corpus violator shape).
const CONTROLLER_THEN_SCOPE: &str = r#"<controller>
export default controller(() => {
  const ticks = signal(0)
  return { ticks }
})
</controller>
<scope>
  props: {
    label = 'ok'
  }
</scope>
<div>{label}: {ticks()}</div>
"#;

/// Teaching diagnostic must name both blocks and the legal precedence.
fn is_block_order_teaching(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    let names_scope = msg.contains("<scope>") || lower.contains("scope");
    let names_controller = msg.contains("<controller>") || lower.contains("controller");
    let teaches_order = lower.contains("precede")
        || lower.contains("before")
        || lower.contains("order")
        || lower.contains("must come");
    names_scope && names_controller && teaches_order
}

#[test]
fn parser_rejects_controller_before_scope_with_teaching_error() {
    let err = compile(CONTROLLER_THEN_SCOPE, component_opts("order-bad")).expect_err(
        "controller-before-scope must be a compile/parse error (SPEC §4 grammar, not lint)",
    );
    assert!(
        is_block_order_teaching(&err.message),
        "error must teach `<scope>` before `<controller>`; got: {}",
        err.message
    );
}

#[test]
fn parser_accepts_scope_before_controller() {
    compile(SCOPE_THEN_CONTROLLER, component_opts("order-ok"))
        .expect("scope-before-controller is the one legal order and must compile");
}
