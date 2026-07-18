//! Scanner robustness regressions.
//!
//! 1. Multibyte chars (e.g. `…` U+2026) anywhere in a `<scope>` block must
//!    produce a diagnostic or compile — never a char-boundary panic.
//! 2. Block markers (`{#for}`, `{#if}`) inside quoted string literals in
//!    template expressions are string content, not control flow.

use sail_compiler::{CompileMode, CompileOpts, ComponentMeta, compile};

fn compile_src(src: &str) -> Result<sail_compiler::CompileOutput, sail_compiler::CompileError> {
    compile(
        src,
        CompileOpts {
            mode: CompileMode::Component,
            component_meta: Some(ComponentMeta { name: "x-probe".into() }),
            ..Default::default()
        },
    )
}

// ── multibyte in <scope> ─────────────────────────────────────────────────────

#[test]
fn multibyte_bare_in_scope_is_a_diagnostic_not_a_panic() {
    // The original crash input: an ellipsis in prose accidentally left in the
    // block. "end byte index 8 is not a char boundary; it is inside '…'".
    let src = "<scope>\n  …the contract…\n</scope>\n<div>hi</div>";
    let err = compile_src(src).expect_err("prose is not a scope member");
    assert!(err.message.contains("…"), "names the offending char: {}", err.message);
}

#[test]
fn multibyte_in_props_position_is_a_diagnostic_not_a_panic() {
    let src = "<scope>\n  props: {\n    … label = 'hi'\n  }\n</scope>\n<div>hi</div>";
    let err = compile_src(src).expect_err("… is not a field name");
    assert!(err.message.contains("…"), "names the offending char: {}", err.message);
}

#[test]
fn multibyte_in_variants_position_is_a_diagnostic_not_a_panic() {
    let src = "<scope>\n  variants: {\n    … tone: { solid: 'bg' }\n  }\n</scope>\n<div>hi</div>";
    let err = compile_src(src).expect_err("… is not an axis name");
    assert!(err.message.contains("…"), "names the offending char: {}", err.message);
}

#[test]
fn multibyte_in_type_alias_and_string_values_compiles() {
    // Multibyte content in legal positions — doc comments, string literal
    // defaults, type alias RHS strings — must simply work.
    let src = "<scope>\n  type Kind = 'café' | 'naïve'\n  props: {\n    /** …the label… */\n    label = 'héllo…'\n    kind: 'café' | 'naïve' = 'café'\n  }\n</scope>\n<div>{label}</div>";
    let out = compile_src(src).expect("multibyte in legal positions compiles");
    assert!(out.js.contains("héllo…"), "string default carried through");
}

// ── block markers inside string expressions ──────────────────────────────────

#[test]
fn for_marker_in_single_quoted_string_expr_is_text() {
    let src = "<div>{'{#for}'}</div>";
    let out = compile_src(src).expect("a quoted {#for} is string content, not a for-block");
    assert!(out.js.contains("{#for}"), "the literal text survives to output");
}

#[test]
fn if_marker_in_double_quoted_string_expr_is_text() {
    let src = "<div>{\"{#if}\"}</div>";
    let out = compile_src(src).expect("a quoted {#if} is string content, not an if-block");
    assert!(out.js.contains("{#if}"), "the literal text survives to output");
}

#[test]
fn close_marker_in_string_expr_does_not_close_a_real_block() {
    // A real {#if} block whose body contains a quoted '{/if}' — the string
    // must not terminate the block early.
    let src = "<div>{#if true}<span>{'{/if}'}</span>{/if}</div>";
    let out = compile_src(src).expect("quoted close marker is inert");
    assert!(out.js.contains("{/if}"), "the literal text survives to output");
}

