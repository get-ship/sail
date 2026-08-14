//! CSS adapter tests for the flat concern chain. Parse-only cases stay in parse.rs.

use std::collections::HashMap;

use spinnaker::parse::{parse, parse_style_const_rhs};
use spinnaker::{Style, StyleDiagnostic};

use super::lower_style;

fn msgs(diags: &[StyleDiagnostic]) -> Vec<String> {
    diags.iter().map(|d| d.message()).collect()
}

#[test]
fn form_card_parses_and_lowers() {
    let (style, diags) = parse_style_const_rhs(
        "{ size(full).pad({ x: 5, y: 4 }).bg(card).border(subtle).radius(lg) }",
    );
    assert!(diags.is_empty(), "{:?}", msgs(&diags));
    assert!(!matches!(style, Style::Empty));
    let (classes, css) = lower_style(&style, &HashMap::new());
    assert!(classes.contains("s-"), "{classes}");
    assert!(
        css.contains("width") || css.contains("100%"),
        "size(full) must emit geometry: {css}"
    );
    assert!(
        css.contains("padding"),
        "pad slots must emit padding: {css}"
    );
    assert!(
        css.contains("background") || css.contains("--fill-card") || css.contains("--card"),
        "bg(card) must emit fill: {css}"
    );
    assert!(
        css.contains("border"),
        "border(subtle) must emit border: {css}"
    );
    assert!(
        css.contains("border-radius") || css.contains("--radius"),
        "radius(lg) must emit radius: {css}"
    );
}

#[test]
fn muted_parses_and_lowers() {
    let (style, diags) = parse_style_const_rhs("{ type(body-sm).fg(muted-foreground) }");
    assert!(diags.is_empty(), "{:?}", msgs(&diags));
    let (_, css) = lower_style(&style, &HashMap::new());
    assert!(
        css.contains("font") || css.contains("--type-body"),
        "type(body-sm) must emit type: {css}"
    );
    assert!(
        css.contains("color") || css.contains("muted-foreground"),
        "fg(muted-foreground) must emit ink: {css}"
    );
}

#[test]
fn when_bare_chain_parses_and_lowers() {
    let (style, diags) = parse("style({ when: { hover: bg(surface-2) } })");
    assert!(diags.is_empty(), "{:?}", msgs(&diags));
    let (_, css) = lower_style(&style, &HashMap::new());
    assert!(css.contains(":hover"), "{css}");
    assert!(
        css.contains("background") || css.contains("surface-2"),
        "hover bg must emit fill: {css}"
    );
}

#[test]
fn chain_then_when_in_declared_object() {
    let (style, diags) = parse_style_const_rhs(
        "{ pad({ x: 3, y: 2 }).radius(md).type(label), when: { hover: bg(surface-2), current: bg(primary).fg(primary-foreground) } }",
    );
    assert!(diags.is_empty(), "{:?}", msgs(&diags));
    let (_, css) = lower_style(&style, &HashMap::new());
    assert!(css.contains(":hover"), "{css}");
    assert!(
        css.contains("aria-current") || css.contains("current") || css.contains("primary"),
        "current when must emit: {css}"
    );
}

#[test]
fn flat_concern_chain_parses() {
    let (style, diags) = parse("style({ size({ w: full }).pad(4).bg(muted) })");
    assert!(diags.is_empty(), "{:?}", msgs(&diags));
    assert!(!matches!(style, Style::Empty));
    let (_, css) = lower_style(&style, &HashMap::new());
    assert!(css.contains("padding") || css.contains("width"), "{css}");
    assert!(css.contains("background") || css.contains("muted"), "{css}");
}

#[test]
fn tone_bound_paint_gates_selector_not_class() {
    let (style, diags) =
        parse("style({ match: { variant: { soft: bg(muted).bg({ color: tone, mix: 15 }) } } })");
    assert!(diags.is_empty(), "{:?}", msgs(&diags));
    let (classes, css) = lower_style(&style, &HashMap::new());
    assert!(
        classes.contains("s-variantsoft-bgmuted"),
        "muted rest class: {classes}"
    );
    assert!(
        classes.contains("s-variantsoft-bgcolortonemix15"),
        "tone wash class: {classes}"
    );
    assert!(
        css.contains("[variant='soft'].s-variantsoft-bgmuted"),
        "muted rest stays variant-gated only: {css}"
    );
    assert!(
        css.contains("[variant='soft'][tone].s-variantsoft-bgcolortonemix15"),
        "tone wash must be [tone]-gated: {css}"
    );
    assert!(
        !css.contains("[variant='soft'].s-variantsoft-bgcolortonemix15{"),
        "ungated tone wash would lose to later muted rest: {css}"
    );
}

#[test]
fn inline_pad_call_parses() {
    let (style, diags) = parse("style(pad({ t: 2 }))");
    assert!(diags.is_empty(), "{:?}", msgs(&diags));
    let (_, css) = lower_style(&style, &HashMap::new());
    assert!(css.contains("padding"), "{css}");
}

#[test]
fn play_colors_fast_resolves() {
    let (style, diags) = parse("style({ animate: { transition: play(colors, fast) } })");
    assert!(
        !diags
            .iter()
            .any(|d| matches!(d, StyleDiagnostic::UnknownVocabulary { .. })),
        "colors is a play prop-set: {:?}",
        msgs(&diags)
    );
    let (_, css) = lower_style(&style, &HashMap::new());
    assert!(
        css.contains("transition-property") || css.contains("transition"),
        "play(colors, fast) must emit transition CSS: {css}"
    );
    assert!(
        css.contains("--duration-fast") || css.contains("duration-fast"),
        "duration token: {css}"
    );
}

#[test]
fn product_emit_is_intent_then_css() {
    use spinnaker::lower_to_intent;

    use super::{emit_intent, emit_style};

    let (style, diags) = parse("style({ pad(4).bg(surface), when: { hover: bg(muted) } })");
    assert!(diags.is_empty(), "{:?}", msgs(&diags));
    let via_style = emit_style(&style, &HashMap::new());
    let via_intent = emit_intent(&lower_to_intent(&style), &HashMap::new());
    assert_eq!(
        via_style, via_intent,
        "emit_style must be emit_intent(lower_to_intent)"
    );
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/emit.rs"),
    )
    .expect("emit.rs");
    assert!(
        !src.contains("match style") && !src.contains("Style::Set"),
        "adapter must not walk Style variants"
    );
}
