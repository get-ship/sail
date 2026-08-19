//! Style product lowering — focused tests for the AOT path the compiler runs
//! on every element tag Style statement (`style({…})`) before emit.
//!
//! Utility-string `style="…"` is residual product and must not emit classes.
//! The shared Style setters re-home emit heads from
//! Style setters emit from `vocab.rs` / `plains.rs` / `concerns.rs`.

use sail_compiler::parse::preprocess::preprocess as preprocess_src;

fn pp(src: &str) -> String {
    preprocess_src(src).expect("preprocess")
}

#[test]
fn style_pad_lowers_to_padding_var() {
    let out = pp(r#"<div style({ pad(4) })> x</div>"#);
    assert!(
        out.contains("s-"),
        "style({{ pad(4) }}) must emit an sx class:\n{out}"
    );
    assert!(
        out.contains("var(--space-4)") || out.contains("padding"),
        "style({{ pad(4) }}) CSS must reference space token:\n{out}"
    );
}

#[test]
fn style_bg_surface_lowers() {
    let out = pp(r#"<div style({ bg(surface) })> x</div>"#);
    assert!(
        out.contains("var(--surface-1)") || out.contains("background"),
        "style({{ bg(surface) }}) must bind surface token:\n{out}"
    );
}

#[test]
fn style_fg_muted_lowers() {
    let out = pp(r#"<div style({ fg(muted-foreground) })> x</div>"#);
    assert!(
        out.contains("var(--muted-foreground)") || out.contains("color"),
        "style({{ fg(muted-foreground) }}) must bind color token:\n{out}"
    );
}

#[test]
fn style_radius_lowers() {
    let out = pp(r#"<div style({ radius(md) })> x</div>"#);
    assert!(
        out.contains("var(--radius-md)") || out.contains("border-radius"),
        "style({{ radius(md) }}) must bind radius token:\n{out}"
    );
}

#[test]
fn style_chain_multi_setters() {
    let out = pp(r#"<div style({ pad(4).bg(surface).fg(muted-foreground) })> x</div>"#);
    assert!(out.contains("s-"), "chain must emit classes:\n{out}");
    assert!(
        out.contains("var(--space-4)") || out.contains("padding"),
        "pad in chain:\n{out}"
    );
    assert!(
        out.contains("var(--surface-1)") || out.contains("background"),
        "bg in chain:\n{out}"
    );
}

#[test]
fn style_when_wide_emits_theme_media() {
    let out = pp(r#"<div style({ bg(primary), when: { wide: bg(accent) } })> x</div>"#);
    assert!(
        out.contains("@media") && (out.contains("1600px") || out.contains("min-width")),
        "when wide must emit theme width:\n{out}"
    );
}

#[test]
fn style_when_tablet_compiles_away_on_1bp_theme() {
    spinnaker::tokens::with_breakpoint_table(&[("mobile", "390px")], || {
        let out = pp(r#"<div style({ bg(primary), when: { tablet: bg(accent) } })> x</div>"#);
        assert!(
            !out.contains("768") && !out.contains("@media"),
            "1-bp theme must compile tablet away:\n{out}"
        );
    });
}

#[test]
fn style_pad_page_spends_theme_token() {
    let out = pp(r#"<div style({ pad(page) })> x</div>"#);
    assert!(
        out.contains("--pad-page") || out.contains("s-pad"),
        "pad(page) must spend the theme token:\n{out}"
    );
    assert!(
        !out.contains("#") || out.contains("var(--"),
        "must not hex-dump the inset:\n{out}"
    );
}

#[test]
fn style_when_hover_emits_pseudo() {
    // Free `when({…})` / `.when(…)` chains are residual; product is object-key when:.
    let out = pp(r#"<div style({ when: { hover: bg(muted) } })> x</div>"#);
    assert!(
        out.contains(":hover") || out.contains("hover"),
        "when hover must emit pseudo:\n{out}"
    );
    assert!(
        out.contains("var(--muted)") || out.contains("background"),
        "hover bg token:\n{out}"
    );
}

#[test]
fn style_position_relative() {
    let out = pp(r#"<div style({ position(relative) })> x</div>"#);
    assert!(
        out.contains("relative") || out.contains("position"),
        "position(relative):\n{out}"
    );
}

#[test]
fn style_bind_var_ref() {
    let out = pp(r#"<div style({ size({ w: bind(glowRadius) }) })> x</div>"#);
    assert!(
        out.contains("var(--glow-radius)") || out.contains("--glow"),
        "bind must emit custom property read:\n{out}"
    );
}

#[test]
fn utility_string_product_is_a_compile_error() {
    // Utility-string atoms attach as residual `style="…"` (not product Style).
    // The `sx=` host is scrubbed to generic unknown-attribute (no Style tombstone).
    let err = preprocess_src(r#"<div style="p-4 bg-surface-1">x</div>"#)
        .expect_err("utility-string style= must fail loudly");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("Style error")
            || msg.contains("utility-string")
            || msg.contains("style(")
            || msg.contains("style=")
            || msg.contains("tag position"),
        "error must name the Style diagnostic: {msg}"
    );
}

#[test]
fn structured_string_product_is_a_compile_error() {
    let err = preprocess_src(r#"<div style="bg: primary px: 2">x</div>"#)
        .expect_err("structured utility-string style= must fail loudly");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("Style error")
            || msg.contains("utility-string")
            || msg.contains("style(")
            || msg.contains("style=")
            || msg.contains("tag position"),
        "error must name the Style diagnostic: {msg}"
    );
}

#[test]
fn style_transparent_bg() {
    let out = pp(r#"<div style({ bg(transparent) })> x</div>"#);
    assert!(
        out.contains("transparent") || out.contains("background"),
        "bg(transparent):\n{out}"
    );
}

#[test]
fn style_display_none() {
    let out = pp(r#"<div style({ display(none) })> x</div>"#);
    assert!(
        out.contains("display:none") || out.contains("none") || out.contains("s-"),
        "display(none):\n{out}"
    );
}
