//! SPEC-SPINNAKER-2 — concern table is the setter table.

use sail_compiler::compile_element;
use spinnaker::{StyleDiagnostic, clash, parse};
use spinnaker_css::{lower_style, lower_style_source};
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

fn parse_diags(src: &str) -> Vec<StyleDiagnostic> {
    parse(src).1
}

fn lower_must_fail(src: &str) -> Vec<StyleDiagnostic> {
    let mut css = String::new();
    let mut seen = std::collections::HashSet::new();
    match lower_style_source(src, &HashMap::new(), &mut css, &mut seen) {
        Err(diags) => diags,
        Ok(classes) => panic!("expected hard fail, got classes={classes} css={css}"),
    }
}

fn retired_names_concern(src: &str, concern: &str) {
    let diags = parse_diags(src);
    assert!(
        diags.iter().any(|d| {
            matches!(d, StyleDiagnostic::RetiredSetter { .. }) && d.message().contains(concern)
        }),
        "{src} must RetiredSetter naming `{concern}`: {:?}",
        diags.iter().map(|d| d.message()).collect::<Vec<_>>()
    );
}

#[test]
fn concern_resolve_border_subtle() {
    let css = css_of("style({ border(subtle) })");
    assert!(
        css.contains("var(--border-subtle-color)"),
        "color member: {css}"
    );
    assert!(
        css.contains("var(--border-subtle-width)"),
        "width member: {css}"
    );
    assert!(
        css.contains("var(--border-subtle-style)"),
        "style member: {css}"
    );
}

#[test]
fn concern_resolve_border_dialog_unqualified() {
    // component.* packs are gone. `dialog` is not a border composite.
    let diags = parse_diags("style({ border(dialog) })");
    assert!(
        !diags.is_empty(),
        "border(dialog) must not resurrect a deleted pack: {:?}",
        diags.iter().map(|d| d.message()).collect::<Vec<_>>()
    );
}

#[test]
fn slot_object_border() {
    let css = css_of("style({ border({ color: muted, width: hairline, style: solid }) })");
    assert!(css.contains("var(--muted)"), "{css}");
    assert!(css.contains("var(--border-width)"), "{css}");
    assert!(
        css.contains("border-style") && css.contains("solid"),
        "{css}"
    );
}

#[test]
fn scoped_name_shadows_global() {
    let (style, diags) = parse("style({ border(subtle) })");
    assert!(diags.is_empty(), "{diags:?}");
    let css = lower_style(&style, &HashMap::new()).1;
    assert!(
        css.contains("--border-subtle-color"),
        "scoped resolve must win: {css}"
    );
}

#[test]
fn old_property_setters_are_retired() {
    retired_names_concern("style({ borderWidth(1).border(primary) })", "border");
}

#[test]
fn when_override_allowed() {
    let (style, diags) =
        parse("style({ border(subtle), when: { hover: border({ color: primary }) } })");
    assert!(diags.is_empty(), "{diags:?}");
    let clashes = clash::detect_clashes(&style, &HashMap::new());
    assert!(clashes.is_empty(), "when: override is legal: {clashes:?}");
    let css = lower_style(&style, &HashMap::new()).1;
    assert!(css.contains(":hover"), "{css}");
}

#[test]
fn radius_stays_out_of_border() {
    let diags = lower_must_fail("style({ border({ radius: md }) })");
    assert!(
        diags.iter().any(|d| {
            matches!(d, StyleDiagnostic::UnknownConcernSlot { slot, .. } if slot == "radius")
                && d.message().contains("radius(…)")
        }),
        "{:?}",
        diags.iter().map(|d| d.message()).collect::<Vec<_>>()
    );
    let err = compile_element(r#"<div style({ border({ radius: md }) })>x</div>"#, "card")
        .expect_err("radius on border must fail compilation");
    assert!(
        err.message.contains("radius") && err.message.contains("radius(…)"),
        "{}",
        err.message
    );
}

#[test]
fn duplicate_slot_fails_compilation() {
    let diags = lower_must_fail("style({ border({ color: muted, color: primary }) })");
    let msg = diags
        .iter()
        .find(|d| matches!(d, StyleDiagnostic::DuplicateSlot { .. }))
        .map(|d| d.message())
        .unwrap_or_else(|| {
            panic!(
                "{:?}",
                diags.iter().map(|d| d.message()).collect::<Vec<_>>()
            )
        });
    assert!(
        msg.contains("style `border` assigns `color` twice"),
        "{msg}"
    );
}

#[test]
fn slot_color_unknown_pack_name_is_bad_token() {
    let diags = lower_must_fail("style({ border({ color: dialog-border }) })");
    assert!(
        diags.iter().any(|d| match d {
            StyleDiagnostic::BadToken { value, .. } if value == "dialog-border" => true,
            StyleDiagnostic::ThemeOwnedToken { name, .. } if name == "dialog-border" => true,
            _ => false,
        }),
        "{:?}",
        diags.iter().map(|d| d.message()).collect::<Vec<_>>()
    );
}

#[test]
fn size_slot_object_and_named_full() {
    let css = css_of("style({ size({ w: 100%, h: 100% }) })");
    assert!(css.contains("width") && css.contains("100%"), "{css}");
    let css = css_of("style({ size(grow) })");
    assert!(css.contains("flex-grow"), "{css}");
    assert!(
        !css.contains("width:100%") && !css.contains("width: 100%"),
        "size(grow) must not emit width 100%: {css}"
    );
}

#[test]
fn size_basis_slot_lowers() {
    let css = css_of("style({ size({ basis: 0, min-w: 0 }) })");
    assert!(css.contains("flex-basis") && css.contains("0"), "{css}");
    retired_names_concern("style({ basis(0) })", "size");
}

#[test]
fn type_italic_nums_wrap_prewrap() {
    let css = css_of("style({ type({ italic: true, nums: tabular, wrap: pre-wrap }) })");
    assert!(
        css.contains("font-style") && css.contains("italic"),
        "{css}"
    );
    assert!(
        css.contains("font-variant-numeric") && css.contains("tabular-nums"),
        "{css}"
    );
    assert!(
        css.contains("white-space") && css.contains("pre-wrap"),
        "{css}"
    );
}

#[test]
fn ink_gradient_and_motion_transform_bind() {
    let css = css_of("style({ fg({ gradient: primary }) })");
    assert!(css.contains("--gradient-primary"), "{css}");
    assert!(css.contains("background-clip"), "{css}");
    let css = css_of("style({ transform(bind(worldTx)) })");
    assert!(
        css.contains("transform") && css.contains("--world-tx"),
        "{css}"
    );
}

#[test]
fn gradient_molecules_paint_bg_fg_border() {
    for name in ["primary", "accent", "surface"] {
        let css = css_of(&format!("style({{ bg({{ gradient: {name} }}) }})"));
        assert!(
            css.contains(&format!("--gradient-{name}")),
            "bg {name}: {css}"
        );
        let css = css_of(&format!("style({{ fg({{ gradient: {name} }}) }})"));
        if name == "surface" {
            assert!(
                css.contains("background-clip") && css.contains("border-box"),
                "fg surface must not clip ink to none: {css}"
            );
            assert!(
                !css.contains("transparent"),
                "fg surface must keep authored color: {css}"
            );
        } else {
            assert!(
                css.contains(&format!("--gradient-{name}")) && css.contains("background-clip"),
                "fg {name}: {css}"
            );
        }
        let css = css_of(&format!("style({{ border({{ gradient: {name} }}) }})"));
        assert!(
            css.contains(&format!("--gradient-{name}")) && css.contains("border-image"),
            "border {name}: {css}"
        );
    }
    let css = css_of("style({ bg({ gradient: none }) })");
    assert!(
        css.contains("background-image") && css.contains("none"),
        "{css}"
    );
}

#[test]
fn gradient_unknown_names_the_table() {
    let diags = parse_diags("style({ bg({ gradient: rainbow }) })");
    let msg = diags
        .iter()
        .map(|d| d.message())
        .find(|m| m.contains("gradient molecule"))
        .expect("must name the table");
    assert!(
        msg.contains("rainbow")
            && msg.contains("primary")
            && msg.contains("accent")
            && msg.contains("surface")
            && msg.contains("none"),
        "{msg}"
    );
}

#[test]
fn structured_ref_and_op_paint() {
    let css = css_of("style({ bg({ ref: primary }) })");
    assert!(css.contains("var(--primary)"), "{css}");
    let css = css_of("style({ fg({ color: { ref: accent } }) })");
    assert!(css.contains("var(--accent)"), "{css}");
    let css = css_of("style({ bg({ op: mix, from: primary, onto: accent, amount: 50 }) })");
    assert!(
        css.contains("color-mix") && css.contains("--primary") && css.contains("--accent"),
        "{css}"
    );
}

#[test]
fn structured_unknown_op_names_the_form() {
    let diags = parse_diags("style({ bg({ op: blur, from: primary, onto: accent }) })");
    let msg = diags
        .iter()
        .map(|d| d.message())
        .find(|m| m.contains("unknown op"))
        .expect("must name the form");
    assert!(msg.contains("`blur`") && msg.contains("{op: mix}"), "{msg}");
}

#[test]
fn structured_mix_missing_from_names_the_form() {
    let diags = parse_diags("style({ bg({ op: mix, onto: accent }) })");
    let msg = diags
        .iter()
        .map(|d| d.message())
        .find(|m| m.contains("mix op requires"))
        .expect("must name the form");
    assert!(msg.contains("`from`"), "{msg}");
}

#[test]
fn structured_dangling_ref_names_the_form() {
    let diags = parse_diags("style({ bg({ ref: no-such }) })");
    let msg = diags
        .iter()
        .map(|d| d.message())
        .find(|m| m.contains("dangling ref"))
        .expect("must name the form");
    assert!(msg.contains("`{no-such}`"), "{msg}");
}

#[test]
fn kind_stroke_plus_ring_coexist() {
    let (style, diags) = parse("style({ border(subtle).border({ kind: ring, color: primary }) })");
    assert!(diags.is_empty(), "{diags:?}");
    let clashes = clash::detect_clashes(&style, &HashMap::new());
    assert!(
        clashes.is_empty(),
        "stroke + ring must coexist: {clashes:?}"
    );
    let css = lower_style(&style, &HashMap::new()).1;
    assert!(
        css.contains("border-color") || css.contains("--border-subtle"),
        "stroke lives: {css}"
    );
    assert!(
        css.contains("box-shadow") && css.contains("primary"),
        "ring lives: {css}"
    );
}

#[test]
fn kind_same_twice_clashes() {
    let (style, diags) = parse("style({ border(subtle).border({ color: primary }) })");
    assert!(diags.is_empty(), "{diags:?}");
    let clashes = clash::detect_clashes(&style, &HashMap::new());
    let clash = clashes
        .iter()
        .find(|d| matches!(d, StyleDiagnostic::Clash { .. }))
        .expect("same-kind twice must clash");
    let msg = clash.message();
    assert!(msg.contains("border(subtle)"), "{msg}");
    assert!(msg.contains("primary") || msg.contains("border({"), "{msg}");
}

#[test]
fn slot_merge_partial_hover() {
    let css = css_of("style({ border(subtle), when: { hover: border({ color: primary }) } })");
    assert!(css.contains(":hover"), "{css}");
    // Hover rule emits only the named slot — not a whole-concern rewrite.
    let hover = css.split(":hover").nth(1).unwrap_or("");
    assert!(
        hover.contains("primary") || hover.contains("border-color"),
        "hover writes color: {css}"
    );
    assert!(
        !hover.contains("border-subtle-width") && !hover.contains("border-subtle-style"),
        "hover must not rewrite subtle width/style: {css}"
    );
    // Base still has subtle width/style.
    assert!(
        css.contains("border-subtle-width") || css.contains("border-subtle-style"),
        "base keeps subtle members: {css}"
    );
}

#[test]
fn size_and_space_do_not_clash() {
    let (style, diags) = parse("style({ size({ w: '100%' }).pad({ x: 3 }) })");
    assert!(diags.is_empty(), "{diags:?}");
    let clashes = clash::detect_clashes(&style, &HashMap::new());
    assert!(clashes.is_empty(), "{clashes:?}");
    let css = lower_style(&style, &HashMap::new()).1;
    assert!(css.contains("width"), "{css}");
    assert!(css.contains("padding"), "{css}");
}

#[test]
fn size_has_no_pad_slot() {
    let diags = parse_diags("style({ size({ pad-x: 3 }) })");
    assert!(
        diags.iter().any(|d| {
            matches!(d, StyleDiagnostic::UnknownConcernSlot { slot, .. } if slot == "pad-x")
        }),
        "{:?}",
        diags.iter().map(|d| d.message()).collect::<Vec<_>>()
    );
}

#[test]
fn truncate_and_gap_stay_retired() {
    retired_names_concern("style({ truncate() })", "type");
    let diags = parse_diags("style({ gap(4) })");
    assert!(
        diags.iter().any(|d| {
            let m = d.message();
            m.contains("gap") && (m.contains("<row") || m.contains("layout"))
        }),
        "{:?}",
        diags.iter().map(|d| d.message()).collect::<Vec<_>>()
    );
}

#[test]
fn constructor_chain_hard_fails() {
    let diags = parse_diags("style.padX(3).radius(full)");
    assert!(
        diags.iter().any(|d| {
            let m = d.message();
            m.contains("retired") || m.contains("concern")
        }),
        "{:?}",
        diags.iter().map(|d| d.message()).collect::<Vec<_>>()
    );
}

#[test]
fn sx_attribute_hard_fails() {
    let err = compile_element(r#"<div sx="p-4 bg-muted">x</div>"#, "card")
        .expect_err("sx= must not compile");
    let msg = err.message.to_lowercase();
    assert!(
        msg.contains("sx")
            || msg.contains("unknown")
            || msg.contains("attribute")
            || msg.contains("style"),
        "{}",
        err.message
    );
}

#[test]
fn intra_channel_chain_stays() {
    let css = css_of("style({ size({ w: '100%' }).position(absolute) })");
    assert!(css.contains("width"), "{css}");
    assert!(
        css.contains("position") && css.contains("absolute"),
        "{css}"
    );
}
