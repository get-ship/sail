//! Diagnostic-coverage tests. The compiler must reject the same
//! constructs the interpreter rejects (statements, assignments,
//! arrows where not allowed) with a message and span pointing at
//! the user's source.

use sail_compiler::{CompileMode, CompileOpts, Diagnostic, compile, compile_element};

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
fn arrow_in_attr_expression_is_rejected() {
    let src = r#"<button disabled={() => true}></button>"#;
    let err = compile_route(src);
    assert!(err.message.contains("arrow"), "msg: {}", err.message);
}

#[test]
fn statement_in_text_expression_is_rejected() {
    let src = "<p>{a; b}</p>";
    let err = compile_route(src);
    assert!(err.message.contains("statements"), "msg: {}", err.message);
}

#[test]
fn arrow_in_on_handler_is_allowed() {
    // sanity: on:click={() => fn()} must not trigger the arrow check.
    let src = r#"<button on:click={() => x()}>Go</button>"#;
    compile(
        src,
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
    .expect("on: handlers allow arrows");
}

#[test]
fn assignment_diagnostic_has_a_span() {
    let src = r#"<p>{a = 1}</p>"#;
    let err = compile_route(src);
    assert!(err.span.is_some(), "should carry a span");
    let rendered = err.render(src);
    assert!(rendered.contains("line 1"), "rendered: {rendered}");
}

#[test]
fn bind_this_is_unknown_bind_regardless_of_value_shape() {
    for src in [
        r#"<button bind:this={a.b}>x</button>"#,
        r#"<button bind:this={getRef()}>x</button>"#,
        r#"<button bind:this={el}>x</button>"#,
    ] {
        let err = compile_route(src);
        assert!(
            err.message.contains("unknown bind:this") && err.message.contains("value"),
            "bind:this must be generic unknown-bind, got: {}",
            err.message
        );
        assert!(
            !err.message.to_ascii_lowercase().contains("retired"),
            "must not embalm: {}",
            err.message
        );
        assert!(err.span.is_some());
    }
}

#[test]
fn reflect_empty_on_non_slot_is_rejected() {
    let src = r#"<div reflect-empty="data-x">x</div>"#;
    let err = compile_route(src);
    assert!(err.message.contains("slot"), "msg: {}", err.message);
}

#[test]
fn reflect_empty_with_invalid_attr_name_is_rejected() {
    let src = r#"<slot reflect-empty="DataIconOnly"></slot>"#;
    let err = compile_route(src);
    assert!(
        err.message.contains("[a-z]") || err.message.contains("must match"),
        "msg: {}",
        err.message
    );
}

#[test]
fn reflect_empty_with_expression_is_rejected() {
    let src = r#"<slot reflect-empty={attr}></slot>"#;
    let err = compile_route(src);
    assert!(
        err.message.contains("static") || err.message.contains("expression"),
        "msg: {}",
        err.message
    );
}

#[test]
fn reflect_empty_with_empty_value_is_rejected() {
    let src = r#"<slot reflect-empty=""></slot>"#;
    let err = compile_route(src);
    assert!(
        err.message.contains("attribute name") || err.message.contains("requires"),
        "msg: {}",
        err.message
    );
}

#[test]
fn unknown_modifier_is_rejected() {
    let src = r#"<button on:click|wat={() => x()}></button>"#;
    let err = compile_route(src);
    assert!(err.message.contains("modifier"), "msg: {}", err.message);
}

#[test]
fn missing_close_tag_is_rejected() {
    let src = r#"<div>"#;
    let err = compile_route(src);
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
    // authoring example — a rewrite sweep that "fixes" it silently guts the guard.
    let src = r#"<prop name="x" type="custom" default="" />"#;
    let err = compile_route(src);
    assert!(
        err.message.contains("<prop>") || err.message.contains("unknown element"),
        "the retired `<prop>` tag must be rejected by name: {}",
        err.message
    );
}

// ---------------------------------------------------------------------
// Element mode's "exactly one root element" diagnostics must point at a
// REAL, in-bounds location in the AUTHOR'S source, not past end-of-file and
// not a meaningless (1,1) fallback.
//
// `emit_element`'s root-count checks used `CompileError::new(...)` with no
// `.with_span(...)` at all, so `Diagnostic::from_compile_error`'s `None`
// branch reported every one of these at a hardcoded (1,1) — never the
// actual problem location. Worse: the IR's spans are byte offsets into
// the PREPROCESSED source (`parse::preprocess::preprocess_impl` can inject
// a leading `<style>` block and otherwise change the text's length ahead of
// any given position), so naively reusing an `Instr`'s span against the
// caller's ORIGINAL source can point past its end — reusing them was tried
// and confirmed to produce exactly that: a multi-root fixture whose second
// root's IR span (start byte 92) exceeded the 59-byte original source.
// These pinned spans are computed by an independent textual scan of the
// ORIGINAL source instead (see `skip_scope_and_style` /
// `top_level_element_spans` in `emit/element.rs`), so they can never drift
// past the text that's actually being rendered.

/// Zero-root element source: a `<scope>` block with nothing after it. The
/// diagnostic must land right where a root element was expected — the byte
/// immediately after `</scope>` — not at a fallback (1,1).
#[test]
fn lone_root_element_missing_span_is_in_bounds_and_exact() {
    let src = "<scope>\nprops: { }\n</scope>\n";
    assert_eq!(src.len(), 28, "fixture pinned at 28 bytes");

    let err = compile_element(src, "empty-thing").expect_err("zero roots must be rejected");
    assert!(
        err.message.contains("exactly one root element"),
        "msg: {}",
        err.message
    );

    // The span must exist, and must never exceed the source it is rendered
    // against — this is the literal "past end-of-file" failure mode.
    let span = err.span.expect("zero-root error must carry a span");
    assert!(
        span.start <= src.len() && span.end <= src.len(),
        "span {span:?} must stay within the {}-byte source",
        src.len()
    );
    // Exact, pinned location: end of file (nothing follows `</scope>`) —
    // the sole sensible in-bounds place to point when there's no element.
    assert_eq!(span.start, 28);
    assert_eq!(span.end, 28);

    let d = Diagnostic::from_compile_error(&err, src);
    assert_eq!((d.line, d.col), (4, 1), "diag: {d:?}");
    assert_eq!((d.end_line, d.end_col), (4, 1), "diag: {d:?}");
}

/// Two root elements: the diagnostic must point at the SECOND root — the
/// one that actually violates "exactly one root" — not past EOF, not (1,1).
#[test]
fn multi_root_element_span_points_at_the_offending_second_root() {
    let src = "<scope>\nprops: { }\n</scope>\n\n<span>a</span>\n<span>b</span>\n";
    assert_eq!(src.len(), 59, "fixture pinned at 59 bytes");

    let err = compile_element(src, "two-things").expect_err("two roots must be rejected");
    assert!(
        err.message.contains("ONE static root element"),
        "msg: {}",
        err.message
    );

    let span = err.span.expect("multi-root error must carry a span");
    assert!(
        span.start <= src.len() && span.end <= src.len(),
        "span {span:?} must stay within the {}-byte source (this is exactly \
         where the bug reused a PREPROCESSED-source offset — 92 — against \
         this 59-byte original and landed past its end)",
        src.len()
    );
    // Exact, pinned location: the tag-name token of the SECOND `<span>`.
    assert_eq!(&src[span.start..span.end], "span");
    assert_eq!((span.start, span.end), (45, 49));

    let d = Diagnostic::from_compile_error(&err, src);
    assert_eq!((d.line, d.col), (6, 2), "diag: {d:?}");
    assert_eq!((d.end_line, d.end_col), (6, 6), "diag: {d:?}");
}
