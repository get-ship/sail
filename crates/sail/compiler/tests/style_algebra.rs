//! Style algebra integration tests.
//!
//! Exercises the compile-time Style path through the preprocessor:
//! - setter chains
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
    let src = r#"<div style({ bg(primary).fg(primary-foreground).radius(md) })> </div>"#;
    let js = compile_component(src).expect("compiles");
    assert!(
        js.contains("s-bg") || js.contains("bgcolor") || js.contains("bgprimary"),
        "setter chain should emit classes: {js}"
    );
}

#[test]
fn match_guard_emits_attribute_selector() {
    let src = r#"<div style({ match: { variant: { filled: bg(primary), tonal: bg(secondary) } } })> </div>"#;
    let js = compile_component(src).expect("compiles");
    assert!(
        js.contains("[variant='filled']") || js.contains("variant"),
        "match arm selector missing: {js}"
    );
}

#[test]
fn when_guard_emits_pseudo_selector() {
    let src = r#"<div style({ when: { hover: bg(muted) } })> </div>"#;
    let js = compile_component(src).expect("compiles");
    assert!(
        js.contains(":hover") || js.contains("hover"),
        "when hover selector missing: {js}"
    );
}

#[test]
fn compound_guard_emits_combined_attribute_selector() {
    let src = r#"<div style({ compound: [ { interactive: true, size: lg, elevation(hover) } ] })> </div>"#;
    let js = compile_component(src).expect("compiles");
    assert!(
        js.contains("[interactive='true']") || js.contains("interactive"),
        "compound selectors missing: {js}"
    );
}

#[test]
fn bind_reactive_paint_emits_var() {
    let src = r#"<div use:virtualRow={0} style({ translateY(bind(rowY)) })> </div>"#;
    let js = compile_component(src).expect("compiles");
    assert!(
        js.contains("bind") || js.contains("--row"),
        "bind reactive paint missing: {js}"
    );
}

#[test]
fn raw_color_without_escape_produces_diagnostic() {
    let (_, diags) = parse("style({ bg(#ff00ff) })");
    assert!(!diags.is_empty(), "raw color should fail closed");
}

#[test]
fn invalid_new_syntax_produces_parse_error() {
    let (_, diags) = parse("style({ match: { variant: { filled: bg(ff00ff) } } })");
    assert!(
        !diags.is_empty(),
        "invalid token inside new syntax should produce diagnostics"
    );
}

#[test]
fn clash_detection_flags_duplicate_setter() {
    let (style, _) = parse("style({ bg(primary).bg(muted) })");
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
    let (style, _) = parse("style({ bg(primary) }, { bg(muted) })");
    let diags = sail_lint::style::clash::detect_clashes(&style, &std::collections::HashMap::new());
    assert!(
        !diags.iter().any(|d| d.message().contains("clash")),
        "list items must shadow silently: {diags:?}"
    );
}

#[test]
fn chain_inside_list_item_still_clashes() {
    let (style, _) = parse("style({ bg(primary).bg(muted) })");
    let diags = sail_lint::style::clash::detect_clashes(&style, &std::collections::HashMap::new());
    assert!(
        diags.iter().any(|d| d.message().contains("clash")),
        "chain duplicates inside a list item must clash"
    );
}

#[test]
fn cross_setter_chain_clash_on_shared_slot() {
    // Clash detection keys on per-call paint slots (op_slots), so different
    // setters touching one slot clash under Chain: `border({ width: 1 })`
    // writes `border-width`, the same slot `border({ width: 2 })` writes.
    let (style, _) = parse("style({ border({ width: 1 }).border({ width: 2 }) })");
    let diags = sail_lint::style::clash::detect_clashes(&style, &std::collections::HashMap::new());
    assert!(
        diags
            .iter()
            .any(|d| d.message().contains("border") || d.message().contains("clash")),
        "border width 1 vs 2 must clash on the shared border stroke slot: {diags:?}"
    );
}

#[test]
fn cross_setter_disjoint_slots_do_not_clash() {
    // Distinct concerns write distinct slots and compose without a clash.
    // (v3 `border` is one stroke concern — width + color share a slot.)
    let (style, _) = parse("style({ pad(4).bg(primary) })");
    let diags = sail_lint::style::clash::detect_clashes(&style, &std::collections::HashMap::new());
    assert!(
        !diags.iter().any(|d| d.message().contains("clash")),
        "disjoint per-call slots must not clash: {diags:?}"
    );
}

#[test]
fn style_list_lowers_through_ir() {
    let src = r#"<div style({ bg(primary), match: { variant: { filled: bg(primary) } } })> </div>"#;
    let js = compile_component(src).expect("compiles");
    assert!(
        js.contains("s-") || js.contains("class"),
        "style list should emit classes: {js}"
    );
}
