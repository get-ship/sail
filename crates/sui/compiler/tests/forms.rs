//! Integration tests for `<form mutation="ship.X.Y">` — parse, render,
//! binding manifest, and validation error paths.

use sui_compiler::bindings;
use sui_compiler::parse::parse;
use sui_compiler::render::{RenderOpts, render_default as render};
use sui_compiler::{CompileMode, CompileOpts, compile};

fn render_source(src: &str) -> String {
    let program = parse(src).expect("parse");
    let opts = RenderOpts { view_name: "myview".into() };
    render(&program, &opts).expect("render").html
}

fn compile_view(source: &str) -> sui_compiler::CompileError {
    compile(
        source,
        CompileOpts {
            setup_url: None,
            mode: CompileMode::View,
            source_path: None,
            styles: None,
            component_meta: None,
            emit_dts: false,
            emit_ssr: false,
            emit_component_imports: false,
        },
    )
    .expect_err("expected error")
}

/// Parser produces `AttrInstr::Mutation` and render emits only the structural
/// marker. After 2c the compiler never bakes action= or method= — those are
/// injected by the host (deck-core) using its own route builders.
#[test]
fn form_mutation_renders_marker_only_no_url_in_ssr() {
    let src = r#"<form mutation="ship.agents.create"><button type="submit">Go</button></form>"#;
    let html = render_source(src);
    assert!(
        html.contains(r#"mutation="b0""#),
        "expected mutation marker; got: {html:?}"
    );
    // The compiler must NOT bake /_deck/ URLs into SSR output.
    assert!(
        !html.contains("/_deck/"),
        "compiler SSR must not contain /_deck/ URLs; got: {html:?}"
    );
    assert!(
        !html.contains(r#"method="post""#),
        "compiler must not emit method=post; host does that; got: {html:?}"
    );
}

/// A static `action=` on the author's form is preserved by the host injector —
/// the compiler just passes it through as a Static attribute without adding
/// any mutation-derived action.
#[test]
fn form_mutation_compiler_preserves_author_action_passthrough() {
    let src = r#"<form action="/custom" mutation="ship.agents.create"></form>"#;
    let html = render_source(src);
    assert!(
        html.contains(r#"action="/custom""#),
        "author action must pass through; got: {html:?}"
    );
    assert!(
        !html.contains("/_deck/mutation"),
        "compiler must not inject /_deck/mutation; got: {html:?}"
    );
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
