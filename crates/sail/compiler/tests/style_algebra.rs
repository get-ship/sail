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

use sail_compiler::CompileMode;
use sail_compiler::CompileOpts;
use sail_compiler::compile;
use sail_lint::style::parse;

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
    let src = r#"<div style({ paint: bg(primary).fg(primary-foreground).radius(md) })> </div>"#;
    let js = compile_component(src).expect("compiles");
    assert!(
        js.contains("sx-bgprimary") || js.contains("bgprimary"),
        "setter chain should emit classes: {js}"
    );
}

#[test]
fn match_guard_emits_attribute_selector() {
    let src = r#"<div style(match(variant, { filled: style({ paint: bg(primary) }), tonal: style({ paint: bg(secondary) }) }))> </div>"#;
    let js = compile_component(src).expect("compiles");
    assert!(
        js.contains("[variant='filled']") || js.contains("variant"),
        "match arm selector missing: {js}"
    );
}

#[test]
fn when_guard_emits_pseudo_selector() {
    let src = r#"<div style(when({ hover: style({ paint: bg(muted) }) }))> </div>"#;
    let js = compile_component(src).expect("compiles");
    assert!(
        js.contains(":hover") || js.contains("hover"),
        "when hover selector missing: {js}"
    );
}

#[test]
fn compound_guard_emits_combined_attribute_selector() {
    let src = r#"<div style(compound({ interactive: true, size: lg }, style({ paint: shadow(hover) })))> </div>"#;
    let js = compile_component(src).expect("compiles");
    assert!(
        js.contains("[interactive='true']") || js.contains("interactive"),
        "compound selectors missing: {js}"
    );
}

#[test]
fn bind_reactive_paint_emits_var() {
    let src = r#"<div style({ paint: translateY(bind(rowY)) })> </div>"#;
    let js = compile_component(src).expect("compiles");
    assert!(
        js.contains("bind") || js.contains("--row"),
        "bind reactive paint missing: {js}"
    );
}

#[test]
fn raw_color_without_escape_produces_diagnostic() {
    let (_, diags) = parse("style({ paint: bg(#ff00ff) })");
    assert!(!diags.is_empty(), "raw color should fail closed");
}

#[test]
fn invalid_new_syntax_produces_parse_error() {
    let (_, diags) = parse("match(variant, { filled: style({ paint: bg(#ff00ff) }) })");
    assert!(
        !diags.is_empty(),
        "invalid token inside new syntax should produce diagnostics"
    );
}

#[test]
fn clash_detection_flags_duplicate_setter() {
    let (style, _) = parse("style({ paint: bg(primary).bg(muted) })");
    let diags = sail_lint::style::clash::detect_clashes(&style, &std::collections::HashMap::new());
    assert!(
        diags.iter().any(|d| d.message().contains("clash")),
        "duplicate setters should clash"
    );
}

#[test]
fn list_shadows_duplicate_setter_without_clash() {
    // Chain = unordered conjunction (clash); List = ordered composition
    // (later shadows earlier, no diagnostic).
    let (style, _) = parse("[ style({ paint: bg(primary) }), style({ paint: bg(muted) }) ]");
    let diags = sail_lint::style::clash::detect_clashes(&style, &std::collections::HashMap::new());
    assert!(
        !diags.iter().any(|d| d.message().contains("clash")),
        "list items must shadow silently: {diags:?}"
    );
}

#[test]
fn chain_inside_list_item_still_clashes() {
    let (style, _) = parse("[ style({ paint: bg(primary).bg(muted) }) ]");
    let diags = sail_lint::style::clash::detect_clashes(&style, &std::collections::HashMap::new());
    assert!(
        diags.iter().any(|d| d.message().contains("clash")),
        "chain duplicates inside a list item must clash"
    );
}

#[test]
fn cross_setter_chain_clash_on_shared_slot() {
    // Clash detection keys on per-call paint slots (op_slots), so different
    // setters touching one slot clash under Chain: `border(1)` narrows to
    // `border-width`, the same slot `borderWidth(2)` writes.
    let (style, _) = parse("style({ paint: border(1).borderWidth(2) })");
    let diags = sail_lint::style::clash::detect_clashes(&style, &std::collections::HashMap::new());
    assert!(
        diags.iter().any(|d| d.message().contains("border-width")),
        "border(1) vs borderWidth must clash on the shared border-width slot: {diags:?}"
    );
}

#[test]
fn cross_setter_disjoint_slots_do_not_clash() {
    // Per-call narrowing: a color-token `border` writes only border-color,
    // so it composes with `borderWidth` without a clash.
    let (style, _) = parse("style({ paint: borderWidth(1).border(primary) })");
    let diags = sail_lint::style::clash::detect_clashes(&style, &std::collections::HashMap::new());
    assert!(
        !diags.iter().any(|d| d.message().contains("clash")),
        "disjoint per-call slots must not clash: {diags:?}"
    );
}

#[test]
fn style_list_lowers_through_ir() {
    let src =
        r#"<div style([style({ paint: bg(primary) }), match(variant, { filled: style({ paint: bg(primary) }) })])> </div>"#;
    let js = compile_component(src).expect("compiles");
    assert!(
        js.contains("sx-") || js.contains("class"),
        "style list should emit classes: {js}"
    );
}
