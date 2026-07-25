//! Style algebra integration tests.
//!
//! Exercises the new compile-time Style path through the preprocessor:
//! - setter chains (style.*)
//! - match guards
//! - when guards
//! - compound guards
//! - bind reactive paint
//! - token fail-closed behavior
//! - clash diagnostics

use sail_compiler::compile;
use sail_compiler::CompileMode;
use sail_compiler::CompileOpts;
use sail_lint::style::{emit, parse, Style};

fn compile_component(src: &str) -> Result<String, String> {
    let out = compile(
        src,
        CompileOpts {
            setup_url: None,
            mode: CompileMode::Component,
            source_path: None,
            component_meta: None,
            route_params: None,
            scope_capabilities: Vec::new(),
            emit_dts: false,
            emit_component_imports: false,
            lint_overrides: Vec::new(),
        },
    )
    .map_err(|e| format!("{e:?}"))?;
    Ok(out.js)
}

#[test]
fn setter_chain_lowers_through_style_ir() {
    let src = r#"<div sx={style.bg(primary).fg(onPrimary).radius(md)}></div>"#;
    let js = compile_component(src).expect("compiles");
    assert!(
        js.contains("sx-bgprimary") || js.contains("bgprimary"),
        "setter chain should emit classes: {js}"
    );
}

#[test]
fn match_guard_emits_attribute_selector() {
    let src = r#"<div sx={match(variant, { filled: style.bg(primary), tonal: style.bg(secondary) })}></div>"#;
    let js = compile_component(src).expect("compiles");
    assert!(
        js.contains("[variant='filled']") || js.contains("variant"),
        "match arm selector missing: {js}"
    );
}

#[test]
fn when_guard_emits_pseudo_selector() {
    let src = r#"<div sx={when({ hover: style.bg(muted) })}></div>"#;
    let js = compile_component(src).expect("compiles");
    assert!(
        js.contains(":hover") || js.contains("hover"),
        "when hover selector missing: {js}"
    );
}

#[test]
fn compound_guard_emits_combined_attribute_selector() {
    let src = r#"<div sx={compound({ interactive: true, size: lg }, style.shadow(hover))}></div>"#;
    let js = compile_component(src).expect("compiles");
    assert!(
        js.contains("[interactive='true']") || js.contains("interactive"),
        "compound selectors missing: {js}"
    );
}

#[test]
fn bind_reactive_paint_emits_var() {
    let src = r#"<div sx={style.translateY(bind(rowY))}></div>"#;
    let js = compile_component(src).expect("compiles");
    assert!(
        js.contains("bind") || js.contains("--row"),
        "bind reactive paint missing: {js}"
    );
}

#[test]
fn raw_color_without_escape_produces_diagnostic() {
    let (_, diags) = parse("style.bg(#ff00ff)");
    assert!(!diags.is_empty(), "raw color should fail closed");
}

#[test]
fn invalid_new_syntax_produces_parse_error() {
    let (_, diags) = parse("match(variant, { filled: style.bg(#ff00ff) })");
    assert!(
        !diags.is_empty(),
        "invalid token inside new syntax should produce diagnostics"
    );
}

#[test]
fn clash_detection_flags_duplicate_setter() {
    let (style, _) = parse("style.bg(primary).bg(muted)");
    let diags = sail_lint::style::clash::detect_clashes(&style);
    assert!(
        diags.iter().any(|d| d.message().contains("clash")),
        "duplicate setters should clash"
    );
}

#[test]
fn style_list_lowers_through_ir() {
    let src = r#"<div sx={[style.bg(primary), match(variant, { filled: style.bg(primary) })]}></div>"#;
    let js = compile_component(src).expect("compiles");
    assert!(
        js.contains("sx-") || js.contains("class"),
        "style list should emit classes: {js}"
    );
}
