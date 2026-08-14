//! SPEC-SPINNAKER-2 — freeze language fixtures (zero-not-baseline).
//!
//! These tests are the contract. They fail until the concern table is the
//! setter table. Covered property names, constructor `style.foo()`, and `sx=`
//! do not resolve. Do not encode property-setter fallback as a passing test.

use sail_compiler::compile_element;
use sail_lint::style::{
    Style, StyleDiagnostic, clash, lower_style, lower_style_source, parse, parse_style_const_rhs,
};
use sail_lint::tokens;
use std::collections::HashMap;

fn css_of(src: &str) -> String {
    let (style, diags) = parse(src);
    assert!(
        diags.is_empty(),
        "{src}: {:?}",
        diags.iter().map(|d| d.message()).collect::<Vec<_>>()
    );
    lower_style(&style, &HashMap::new()).1
}

fn lower_must_fail(src: &str) -> Vec<StyleDiagnostic> {
    let mut css = String::new();
    let mut seen = std::collections::HashSet::new();
    match lower_style_source(src, &HashMap::new(), &mut css, &mut seen) {
        Err(diags) => diags,
        Ok(classes) => panic!("expected hard fail, got classes={classes} css={css}"),
    }
}

fn diag_messages(diags: &[StyleDiagnostic]) -> Vec<String> {
    diags.iter().map(|d| d.message()).collect()
}

fn names_concern(diags: &[StyleDiagnostic], concern: &str) -> bool {
    diags.iter().any(|d| match d {
        StyleDiagnostic::RetiredSetter {
            setter,
            replacement,
            ..
        } => {
            setter.contains(concern)
                || replacement.contains(concern)
                || d.message().contains(concern)
        }
        _ => d.message().contains(concern),
    })
}

fn must_retire(src: &str, concern: &str) -> Vec<StyleDiagnostic> {
    let diags = lower_must_fail(src);
    assert!(
        names_concern(&diags, concern),
        "expected compile/lower error naming `{concern}`: {:?}",
        diag_messages(&diags)
    );
    diags
}

fn hover_rule(css: &str) -> String {
    let Some(at) = css.find(":hover") else {
        return String::new();
    };
    let rest = &css[at..];
    let Some(open) = rest.find('{') else {
        return rest.to_string();
    };
    let after = &rest[open + 1..];
    let close = after.find('}').unwrap_or(after.len());
    after[..close].to_string()
}

fn first_composite(ty: &str, candidates: &[&'static str]) -> Option<&'static str> {
    for name in candidates {
        if tokens::find_composite(name).is_some() {
            return Some(*name);
        }
    }
    tokens::composites_of_type(ty).first().map(|c| c.name)
}

fn concern_call_arg(composite_name: &str, concern: &str) -> String {
    let prefix = format!("{concern}.");
    let suffix = format!(".{concern}");
    if let Some(rest) = composite_name.strip_prefix(&prefix) {
        rest.to_string()
    } else if let Some(rest) = composite_name.strip_suffix(&suffix) {
        rest.to_string()
    } else {
        composite_name.to_string()
    }
}

fn is_constructor_product(style: &Style) -> bool {
    fn collect(style: &Style, names: &mut Vec<String>) {
        match style {
            Style::Set(op) => names.push(op.setter.clone()),
            Style::Chain(items) | Style::List(items) => {
                for item in items {
                    collect(item, names);
                }
            }
            _ => {}
        }
    }
    let mut names = Vec::new();
    collect(style, &mut names);
    names.iter().any(|n| n == "padX") && names.iter().any(|n| n == "radius")
}

// ---------------------------------------------------------------------------
// Retired property names — compile/lower error naming the concern
// ---------------------------------------------------------------------------

#[test]
fn retired_border_width_names_concern() {
    let diags = must_retire("style({ borderWidth(1) })", "border");
    assert!(
        diags
            .iter()
            .any(|d| matches!(d, StyleDiagnostic::RetiredSetter { .. })
                || d.message().contains("retired")
                || d.message().contains("border")),
        "{:?}",
        diag_messages(&diags)
    );
    let err = compile_element(r#"<div style({ borderWidth(1) })>x</div>"#, "card")
        .expect_err("borderWidth must fail compilation");
    assert!(
        err.message.contains("border"),
        "compile error must name `border`: {}",
        err.message
    );
}

#[test]
fn bg_and_fg_are_live_author_names() {
    let _ = css_of("style({ bg(primary) })");
    let _ = css_of("style({ fg(foreground) })");
    compile_element(r#"<div style({ bg(primary) })>x</div>"#, "card").expect("bg is live v3");
    compile_element(r#"<div style({ fg(foreground) })>x</div>"#, "card").expect("fg is live v3");
}

#[test]
fn retired_pad_x_names_pad() {
    let diags = must_retire("style({ padX(3) })", "pad");
    assert!(
        names_concern(&diags, "pad"),
        "padX(3) must name `pad`: {:?}",
        diag_messages(&diags)
    );
}

#[test]
fn retired_font_size_names_type() {
    let diags = must_retire("style({ fontSize(sm) })", "type");
    assert!(
        names_concern(&diags, "type"),
        "fontSize(sm) must name `type`: {:?}",
        diag_messages(&diags)
    );
}

#[test]
fn retired_w_names_size() {
    let diags = must_retire("style({ w(full) })", "size");
    assert!(
        names_concern(&diags, "size"),
        "w(full) must name `size`: {:?}",
        diag_messages(&diags)
    );
}

#[test]
fn retired_shadow_names_elevation() {
    let diags = must_retire("style({ shadow(sm) })", "elevation");
    assert!(
        names_concern(&diags, "elevation"),
        "shadow(sm) must name `elevation`: {:?}",
        diag_messages(&diags)
    );
}

#[test]
fn retired_gap_is_not_style() {
    let diags = lower_must_fail("style({ gap(4) })");
    let joined = diag_messages(&diags).join(" | ");
    assert!(joined.contains("gap"), "gap error must name gap: {joined}");
    assert!(
        joined.contains("<row gap>")
            || (joined.contains("row") && joined.to_ascii_lowercase().contains("layout")),
        "gap is a layout attr (`<row gap>`), not Style: {joined}"
    );
    assert!(
        joined.to_ascii_lowercase().contains("layout")
            || joined.contains("<row gap>")
            || joined.contains("not Style")
            || joined.to_ascii_lowercase().contains("not a style"),
        "message must say gap is not Style: {joined}"
    );
    let err = compile_element(r#"<div style({ gap(4) })>x</div>"#, "card")
        .expect_err("gap must fail compilation");
    assert!(
        err.message.contains("gap")
            && (err.message.contains("row") || err.message.to_ascii_lowercase().contains("layout")),
        "compile error must teach `<row gap>`: {}",
        err.message
    );
}

#[test]
fn retired_constructor_chain() {
    let src = "style.padX(3).radius(full)";
    let (style, diags) = parse(src);
    assert!(
        !is_constructor_product(&style),
        "constructor chain must not parse as product Style: {style:?}"
    );
    assert!(
        !diags.is_empty() || matches!(style, Style::Empty),
        "style.padX(3).radius(full) must not be a silent product: style={style:?} diags={diags:?}"
    );
    let (rhs, rhs_diags) = parse_style_const_rhs(src);
    assert!(
        !is_constructor_product(&rhs),
        "declaration RHS constructor chain must not parse as product Style: {rhs:?}"
    );
    assert!(
        !rhs_diags.is_empty() || matches!(rhs, Style::Empty),
        "const RHS style.padX(3).radius(full) must not be a silent product: {rhs:?}"
    );
    let _ = lower_must_fail(src);
}

#[test]
fn retired_sx_attribute() {
    let brace = compile_element(
        r#"<div sx={style({ bg({ color: primary }) })}>x</div>"#,
        "card",
    )
    .expect_err("sx={{…}} must fail compilation (burial)");
    assert!(
        brace.message.contains("sx"),
        "sx={{…}} burial must name `sx`: {}",
        brace.message
    );

    let quoted = compile_element(r#"<div sx="bg({ color: primary })">x</div>"#, "card")
        .expect_err("sx=\"…\" must fail compilation (burial)");
    assert!(
        quoted.message.contains("sx"),
        "sx=\"…\" burial must name `sx`: {}",
        quoted.message
    );
}

#[test]
fn retired_border_color_poke() {
    let _ = css_of("style({ border(primary) })");

    let poke = lower_must_fail("style({ border(dialog.border) })");
    assert!(
        !poke.is_empty(),
        "dotted pack poke must fail closed: {:?}",
        diag_messages(&poke)
    );
}

// ---------------------------------------------------------------------------
// kind family key — coexist vs same-kind clash
// ---------------------------------------------------------------------------

#[test]
fn kind_stroke_plus_ring_coexist() {
    let src = "style({ border(subtle).border({ kind: ring, color: primary }) })";
    let (style, diags) = parse(src);
    assert!(
        diags.is_empty(),
        "stroke + ring must parse: {:?}",
        diag_messages(&diags)
    );
    let clashes = clash::detect_clashes(&style, &HashMap::new());
    assert!(
        clashes.is_empty(),
        "different kinds coexist; clash is per-(concern, kind): {:?}",
        diag_messages(&clashes)
    );
    let css = lower_style(&style, &HashMap::new()).1;
    let norm = css.replace(' ', "");
    assert!(
        norm.contains("border-color")
            || norm.contains("--border-subtle")
            || norm.contains("border-width"),
        "stroke (default) must live in CSS: {css}"
    );
    assert!(
        norm.contains("box-shadow")
            || norm.contains("--ring")
            || norm.contains("outline")
            || norm.to_ascii_lowercase().contains("ring"),
        "ring kind must live in CSS: {css}"
    );

    let hover_src =
        "style({ border(subtle), when: { hover: border({ kind: ring, color: primary }) } })";
    let (hover_style, hover_diags) = parse(hover_src);
    assert!(
        hover_diags.is_empty(),
        "base stroke + hover ring must parse: {:?}",
        diag_messages(&hover_diags)
    );
    assert!(
        clash::detect_clashes(&hover_style, &HashMap::new()).is_empty(),
        "hover ring is a separate kind, not a clash"
    );
    let hover_css = lower_style(&hover_style, &HashMap::new()).1;
    assert!(
        hover_css.contains(":hover"),
        "hover ring must emit :hover: {hover_css}"
    );
}

#[test]
fn kind_same_twice_clashes() {
    let src = "style({ border(subtle).border({ color: primary }) })";
    let (style, parse_diags) = parse(src);
    assert!(
        parse_diags.is_empty(),
        "same-kind pair must parse so clash can name both sites: {:?}",
        diag_messages(&parse_diags)
    );
    let clashes = clash::detect_clashes(&style, &HashMap::new());
    let clash = clashes
        .iter()
        .find(|d| matches!(d, StyleDiagnostic::Clash { .. }))
        .expect("two stroke (default) borders in one channel scope clash");
    let msg = clash.message();
    assert!(
        msg.contains("border(subtle)"),
        "clash must name first site: {msg}"
    );
    assert!(
        msg.contains("border({ color: primary })") || msg.contains("color: primary"),
        "clash must name second site: {msg}"
    );

    let lower_diags = lower_must_fail(src);
    assert!(
        lower_diags
            .iter()
            .any(|d| matches!(d, StyleDiagnostic::Clash { .. })),
        "same-kind clash is a hard fail, not last-wins: {:?}",
        diag_messages(&lower_diags)
    );
}

// ---------------------------------------------------------------------------
// Slot-merge / size vs space / wrap / member access / intra-channel chain
// ---------------------------------------------------------------------------

#[test]
fn slot_merge_partial_hover() {
    let src = "style({ border(subtle), when: { hover: border({ color: primary }) } })";
    let (style, diags) = parse(src);
    assert!(diags.is_empty(), "{:?}", diag_messages(&diags));
    assert!(
        clash::detect_clashes(&style, &HashMap::new()).is_empty(),
        "when: override is a separate scope"
    );
    let css = lower_style(&style, &HashMap::new()).1;
    assert!(css.contains(":hover"), "hover rule missing: {css}");
    assert!(
        css.contains("var(--border-subtle-width)") || css.contains("--border-subtle-width"),
        "base must keep subtle width: {css}"
    );
    assert!(
        css.contains("var(--border-subtle-style)") || css.contains("--border-subtle-style"),
        "base must keep subtle style: {css}"
    );

    let hover = hover_rule(&css);
    assert!(!hover.is_empty(), "expected a :hover rule: {css}");
    assert!(
        hover.contains("primary") || hover.contains("var(--primary)"),
        "hover CSS must have primary color: {hover} (full: {css})"
    );
    let hover_norm = hover.replace(' ', "");
    if hover_norm.contains("border-width") {
        assert!(
            hover_norm.contains("border-subtle-width"),
            "hover must not rewrite width away from subtle: {hover}"
        );
    }
    if hover_norm.contains("border-style") {
        assert!(
            hover_norm.contains("border-subtle-style"),
            "hover must not rewrite style away from subtle: {hover}"
        );
    }
}

#[test]
fn size_and_space_do_not_clash() {
    let src = "style({ size({ w: full }).pad({ x: 3 }) })";
    let (style, diags) = parse(src);
    assert!(
        diags.is_empty(),
        "size + space must parse: {:?}",
        diag_messages(&diags)
    );
    let clashes = clash::detect_clashes(&style, &HashMap::new());
    assert!(
        clashes.is_empty(),
        "size and space do not clash: {:?}",
        diag_messages(&clashes)
    );
    let css = lower_style(&style, &HashMap::new()).1;
    let norm = css.replace(' ', "");
    assert!(
        norm.contains("width") && (norm.contains("100%") || norm.contains("--size")),
        "size({{ w: full }}) must lower: {css}"
    );
    assert!(
        norm.contains("padding-inline") || norm.contains("padding"),
        "pad({{ x: 3 }}) must lower: {css}"
    );
}

#[test]
fn size_has_no_pad_slot() {
    for src in [
        "style({ size({ pad: 3 }) })",
        "style({ size({ pad-x: 3 }) })",
    ] {
        let diags = lower_must_fail(src);
        assert!(
            diags.iter().any(|d| {
                matches!(
                    d,
                    StyleDiagnostic::UnknownConcernSlot { setter, slot, .. }
                        if setter == "size" && (slot == "pad" || slot == "pad-x")
                )
            }),
            "size pad slots are UnknownConcernSlot: {src} {:?}",
            diag_messages(&diags)
        );
    }
}

#[test]
fn fill_and_ink_named() {
    let slot_src = "style({ bg({ color: primary }).fg({ color: muted-foreground }) })";
    let slot_css = css_of(slot_src);
    let slot_norm = slot_css.replace(' ', "");
    assert!(
        slot_norm.contains("background-color") && slot_norm.contains("var(--primary)"),
        "bg({{ color: primary }}) must lower: {slot_css}"
    );
    assert!(
        slot_norm.contains("var(--muted-foreground)")
            && (slot_norm.contains("color:var(--muted-foreground)")
                || slot_css.contains("color: var(--muted-foreground)")),
        "fg({{ color: muted-foreground }}) must lower: {slot_css}"
    );

    // Named resolve only when catalog identities exist.
    let fill_id = first_composite("fill", &["fill.surface", "surface"]);
    let ink_id = first_composite("ink", &["ink.foreground", "foreground"]);
    if let (Some(fill), Some(ink)) = (fill_id, ink_id) {
        let fill_arg = concern_call_arg(fill, "fill");
        let ink_arg = concern_call_arg(ink, "ink");
        let named = format!("style({{ bg({fill_arg}).fg({ink_arg}) }})");
        let css = css_of(&named);
        assert!(
            css.contains("background")
                || css.contains(fill_arg.as_str())
                || css.contains("surface"),
            "bg({fill_arg}) must lower: {css}"
        );
        assert!(
            css.contains("color") || css.contains(ink_arg.as_str()) || css.contains("foreground"),
            "fg({ink_arg}) must lower: {css}"
        );
    }
}

#[test]
fn type_wrap_not_truncate_marker() {
    let diags = lower_must_fail("style({ truncate() })");
    let joined = diag_messages(&diags).join(" | ");
    assert!(
        joined.contains("type") || joined.contains("wrap") || joined.contains("truncate"),
        "truncate() must error; type({{ wrap: truncate }}) is the form: {joined}"
    );
    assert!(
        diags
            .iter()
            .any(|d| matches!(d, StyleDiagnostic::RetiredSetter { .. })
                || d.message().contains("type({ wrap: truncate })")
                || d.message().contains("wrap")),
        "truncate marker is deleted in favor of type.wrap: {joined}"
    );

    let css = css_of("style({ type({ wrap: truncate }) })");
    let norm = css.replace(' ', "");
    assert!(
        norm.contains("ellipsis") || norm.contains("text-overflow") || norm.contains("nowrap"),
        "type({{ wrap: truncate }}) must lower wrap policy: {css}"
    );
}

#[test]
fn member_access_does_not_resolve() {
    let src = "style({ size(control-md).h })";
    let diags = lower_must_fail(src);
    let joined = diag_messages(&diags).join(" | ");
    let lower = joined.to_ascii_lowercase();
    assert!(
        joined.contains("size(control-md).h")
            || lower.contains("member access")
            || lower.contains("does not resolve")
            || (joined.contains(".h") && lower.contains("use site")),
        "size(control-md).h is member access at the use site and must not resolve: {joined}"
    );
}

#[test]
fn intra_channel_chain_stays() {
    let src = "style({ size({ w: full }).position(absolute) })";
    let (style, diags) = parse(src);
    assert!(
        diags.is_empty(),
        "intra-channel concern chain must parse (this is not the constructor chain): {:?}",
        diag_messages(&diags)
    );
    let css = lower_style(&style, &HashMap::new()).1;
    let norm = css.replace(' ', "");
    assert!(
        norm.contains("width") && (norm.contains("100%") || norm.contains("--size")),
        "size({{ w: full }}) must lower: {css}"
    );
    assert!(
        norm.contains("position:absolute") || norm.contains("position: absolute"),
        "position(absolute) must lower: {css}"
    );
}
