//! Integration tests for `<form mutation="ship.X.Y">` — parse, binding
//! manifest, and validation error paths. (SSR-output tests moved out with
//! render-eval; the host owns first-paint HTML now.)

use sui_compiler::bindings;
use sui_compiler::parse::parse;
use sui_compiler::{CompileMode, CompileOpts, compile};

fn compile_view(source: &str) -> sui_compiler::CompileError {
    compile(
        source,
        CompileOpts {
            setup_url: None,
            mode: CompileMode::Component,
            source_path: None,
            styles: None,
            component_meta: None,
            emit_dts: false,
            emit_component_imports: false,
        lint_overrides: Vec::new(),
        },
    )
    .expect_err("expected error")
}

/// The binding manifest contains a `mutation` entry for the form.
#[test]
fn form_mutation_appears_in_binding_manifest() {
    let src = r#"<form mutation="ship.agents.create"></form>"#;
    let program = parse(src).expect("parse");
    let reg = bindings::build(&program);
    let binding = reg.get(0).expect("expected at least one binding");
    assert_eq!(binding.key, "b0");
    assert_eq!(binding.kind.as_marker_str(), "mutation");
    assert_eq!(binding.expr, "ship.agents.create");
}

/// Compiler rejects a `mutation=` value that is not a `ship.X.Y` chain.
#[test]
fn invalid_mutation_expr_is_rejected() {
    let src = r#"<form mutation="agents.create"></form>"#;
    let err = compile_view(src);
    assert!(
        err.message.contains("mutation="),
        "error should mention mutation=; got: {}",
        err.message
    );
}
