//! Diagnostic-coverage tests. The compiler must reject the same
//! constructs the interpreter rejects (statements, assignments,
//! arrows where not allowed) with a message and span pointing at
//! the user's source.

use sui_compiler::{compile, CompileMode, CompileOpts};

fn compile_view(source: &str) -> sui_compiler::CompileError {
    compile(
        source,
        CompileOpts {
        route_params: None,
            setup_url: None,
            mode: CompileMode::Component,
            source_path: None,
            component_meta: None,
            emit_dts: false,
            emit_component_imports: false,
            lint_overrides: Vec::new(),
        },
    )
    .expect_err("expected error")
}

#[test]
fn state_directive_is_retired() {
    // The legacy `<state>` directive was retired in favour of declaring
    // reactive state in the controller's `setup:` factory. Authors who
    // still ship the old form get a parse error pointing at the
    // migration target.
    let src = r#"<state name="x" value="0" />"#;
    let err = compile_view(src);
    assert!(
        err.message.contains("no longer supported"),
        "msg: {}",
        err.message
    );
    assert!(
        err.message.contains("setup:"),
        "msg should name the migration target: {}",
        err.message
    );
    assert_eq!(err.category.as_deref(), Some("<state name=\"x\">"));
}

#[test]
fn arrow_in_attr_expression_is_rejected() {
    let src = r#"<button disabled={() => true}></button>"#;
    let err = compile_view(src);
    assert!(err.message.contains("arrow"), "msg: {}", err.message);
}

#[test]
fn statement_in_text_expression_is_rejected() {
    let src = "<p>{a; b}</p>";
    let err = compile_view(src);
    assert!(err.message.contains("statements"), "msg: {}", err.message);
}

#[test]
fn arrow_in_on_handler_is_allowed() {
    // sanity: on:click={() => fn()} must not trigger the arrow check.
    let src = r#"<button on:click={() => x()}></button>"#;
    compile(
        src,
        CompileOpts {
        route_params: None,
            setup_url: None,
            mode: CompileMode::Component,
            source_path: None,
            component_meta: None,
            emit_dts: false,
            emit_component_imports: false,
            lint_overrides: Vec::new(),
        },
    )
    .expect("on: handlers allow arrows");
}

#[test]
fn assignment_diagnostic_has_a_span() {
    let src = r#"<p>{a = 1}</p>"#;
    let err = compile_view(src);
    assert!(err.span.is_some(), "should carry a span");
    let rendered = err.render(src);
    assert!(rendered.contains("line 1"), "rendered: {rendered}");
}

#[test]
fn bind_this_with_non_identifier_is_rejected() {
    let src = r#"<button bind:this={a.b}>x</button>"#;
    let err = compile_view(src);
    assert!(
        err.message.contains("identifier") || err.message.contains("bare"),
        "msg: {}",
        err.message
    );
    assert!(err.span.is_some());
}

#[test]
fn bind_this_with_call_expression_is_rejected() {
    let src = r#"<button bind:this={getRef()}>x</button>"#;
    let err = compile_view(src);
    assert!(
        err.message.contains("identifier") || err.message.contains("bare"),
        "msg: {}",
        err.message
    );
}

#[test]
fn reflect_empty_on_non_slot_is_rejected() {
    let src = r#"<div reflect-empty="data-x">x</div>"#;
    let err = compile_view(src);
    assert!(
        err.message.contains("slot"),
        "msg: {}",
        err.message
    );
}

#[test]
fn reflect_empty_with_invalid_attr_name_is_rejected() {
    let src = r#"<slot reflect-empty="DataIconOnly"></slot>"#;
    let err = compile_view(src);
    assert!(
        err.message.contains("[a-z]") || err.message.contains("must match"),
        "msg: {}",
        err.message
    );
}

#[test]
fn reflect_empty_with_expression_is_rejected() {
    let src = r#"<slot reflect-empty={attr}></slot>"#;
    let err = compile_view(src);
    assert!(
        err.message.contains("static") || err.message.contains("expression"),
        "msg: {}",
        err.message
    );
}

#[test]
fn reflect_empty_with_empty_value_is_rejected() {
    let src = r#"<slot reflect-empty=""></slot>"#;
    let err = compile_view(src);
    assert!(
        err.message.contains("attribute name") || err.message.contains("requires"),
        "msg: {}",
        err.message
    );
}

#[test]
fn unknown_modifier_is_rejected() {
    let src = r#"<button on:click|wat={() => x()}></button>"#;
    let err = compile_view(src);
    assert!(err.message.contains("modifier"), "msg: {}", err.message);
}

#[test]
fn missing_close_tag_is_rejected() {
    let src = r#"<div>"#;
    let err = compile_view(src);
    assert!(err.message.contains("</div>") || err.message.contains("unterminated"));
}

#[test]
fn the_retired_prop_tag_is_rejected() {
    // This used to assert `<prop type="custom">` -> "unknown type". Both halves are
    // gone. `<prop>` itself is retired (props are declared as typed TS in the
    // `<scope>` block — ONE place, not N tags), and a custom TS type is now
    // perfectly legal: `PropKind::Identity` carries it through verbatim rather than
    // forcing everything into a closed primitive enum. So the invariant worth
    // holding is the retirement, and that its error TEACHES rather than just refuses.
    // NOTE: this literal must stay a `<prop>` tag. It is the specimen, not an
    // authoring example — a migration sweep that "fixes" it silently guts the guard.
    let src = r#"<prop name="x" type="custom" default="" />"#;
    let err = compile_view(src);
    assert!(
        err.message.contains("<prop>") || err.message.contains("unknown element"),
        "the retired `<prop>` tag must be rejected by name: {}",
        err.message
    );
}
