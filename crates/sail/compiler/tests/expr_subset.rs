//! Integration tests for the renderable-expression subset check.
//!
//! These exercise the wiring through `compile()` so we know the
//! check fires after parse / lower and before emit, with diagnostics
//! that carry a span.

use sail_compiler::{CompileMode, CompileOpts, compile};

fn opts() -> CompileOpts {
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
    }
}

fn compile_route(src: &str) -> sail_compiler::CompileError {
    compile(src, opts()).expect_err("expected error")
}

#[test]
fn rejects_new_in_text_interpolation() {
    let err = compile_route("<p>{new Date()}</p>");
    assert!(err.message.contains("`new`"), "msg: {}", err.message);
    assert!(err.span.is_some(), "should carry a span");
}

#[test]
fn rejects_await_in_attribute_expr() {
    let err = compile_route(r#"<p title={await loadName()}></p>"#);
    assert!(err.message.contains("`await`"), "msg: {}", err.message);
}

#[test]
fn rejects_this_in_text_expression() {
    let err = compile_route(r#"<p>{this.value}</p>"#);
    assert!(err.message.contains("`this`"), "msg: {}", err.message);
}

#[test]
fn rejects_call_with_args_in_interpolation() {
    let err = compile_route("<p>{format(name, 'short')}</p>");
    assert!(
        err.message.contains("format") || err.message.contains("call"),
        "msg: {}",
        err.message
    );
}

#[test]
fn allows_zero_arg_call_in_interpolation() {
    compile("<p>{count()}</p>", opts()).expect("zero-arg call must compile");
}

#[test]
fn allows_allowlisted_method_call() {
    compile("<p>{xs.slice(0, 3)}</p>", opts()).expect("xs.slice must compile");
}

#[test]
fn rejects_non_allowlisted_method_call() {
    let err = compile_route("<p>{xs.push(1)}</p>");
    assert!(err.message.contains("push"), "msg: {}", err.message);
}

#[test]
fn rejects_regex_literal() {
    let err = compile_route("<p>{name.match(/foo/)}</p>");
    // The regex literal lives inside an arg list, so the call check
    // and the regex check are both relevant — either one is fine.
    assert!(
        err.message.contains("regex") || err.message.contains("match"),
        "msg: {}",
        err.message
    );
}

#[test]
fn rejects_tagged_template() {
    // Use an attribute so the brace contents stay a single expression;
    // text-node `{...}` is not always greedy enough to wrap a backtick
    // string with HTML-shaped contents.
    let err = compile_route(r#"<p title={html`hi ${name}`}></p>"#);
    assert!(
        err.message.contains("tagged template"),
        "msg: {}",
        err.message
    );
}

#[test]
fn allows_on_handler_with_arrow_and_args() {
    // Event handlers must be exempt from the renderable subset.
    compile(
        r#"<button on:click={() => navigate('/x')}>go</button>"#,
        opts(),
    )
    .expect("event handlers may use arrows + arg-bearing calls");
}

#[test]
fn diagnostic_carries_span_and_category() {
    let err = compile_route("<p>{new X()}</p>");
    assert!(err.span.is_some(), "expected a span");
    assert!(err.category.is_some(), "expected a category");
    let rendered = err.render("<p>{new X()}</p>");
    assert!(rendered.contains("line 1"), "rendered: {rendered}");
}
