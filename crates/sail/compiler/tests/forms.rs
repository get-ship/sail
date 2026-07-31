//! Integration tests for `<form mutation="client.method">`.

use sail_compiler::bindings;
use sail_compiler::parse::parse;
use sail_compiler::{CompileMode, CompileOpts, compile};

fn compile_route(source: &str) -> sail_compiler::CompileError {
    compile(
        source,
        CompileOpts {
            route_params: None,
            setup_url: None,
            mode: CompileMode::Component,
            source_path: None,
            component_meta: None,
            emit_dts: false,
            scope_capabilities: Vec::new(),
            emit_component_imports: false,
            lint_overrides: Vec::new(),
        },
    )
    .expect_err("expected error")
}

#[test]
fn form_mutation_appears_in_binding_manifest() {
    let src = r#"<form mutation="demo.addTodo"></form>"#;
    let program = parse(src).expect("parse");
    let reg = bindings::build(&program);
    let binding = reg.get(0).expect("expected at least one binding");
    assert_eq!(binding.key, "b0");
    assert_eq!(binding.kind.as_marker_str(), "mutation");
    assert_eq!(binding.expr, "demo.addTodo");
}

#[test]
fn invalid_mutation_expr_is_rejected() {
    let src = r#"<form mutation="create"></form>"#;
    let err = compile_route(src);
    assert!(
        err.message.contains("mutation="),
        "error should mention mutation=; got: {}",
        err.message
    );
}
