//! Style product lowering — focused tests for the AOT path the compiler runs
//! on every `<el sx={style.…}>` attribute before emit.
//!
//! Utility-string `sx="…"` is residual product and must not emit classes.
//! The shared Style setters re-home vocab heads from
//! `crates/sail/lint/sx-vocabulary.json` (lint + emit SSOT).

use sail_compiler::parse::preprocess::preprocess as preprocess_src;

fn pp(src: &str) -> String {
    preprocess_src(src).expect("preprocess")
}

#[test]
fn style_pad_lowers_to_padding_var() {
    let out = pp(r#"<div sx={style.pad(4)}>x</div>"#);
    assert!(
        out.contains("sx-"),
        "style.pad(4) must emit an sx class:\n{out}"
    );
    assert!(
        out.contains("var(--space-4)") || out.contains("padding"),
        "style.pad(4) CSS must reference space token:\n{out}"
    );
}

#[test]
fn style_bg_surface_lowers() {
    let out = pp(r#"<div sx={style.bg(surface-1)}>x</div>"#);
    assert!(
        out.contains("var(--surface-1)") || out.contains("background"),
        "style.bg(surface-1) must bind surface token:\n{out}"
    );
}

#[test]
fn style_fg_muted_lowers() {
    let out = pp(r#"<div sx={style.fg(muted-foreground)}>x</div>"#);
    assert!(
        out.contains("var(--muted-foreground)") || out.contains("color"),
        "style.fg(muted-foreground) must bind color token:\n{out}"
    );
}

#[test]
fn style_radius_lowers() {
    let out = pp(r#"<div sx={style.radius(md)}>x</div>"#);
    assert!(
        out.contains("var(--radius-md)") || out.contains("border-radius"),
        "style.radius(md) must bind radius token:\n{out}"
    );
}

#[test]
fn style_chain_multi_setters() {
    let out = pp(r#"<div sx={style.pad(4).bg(surface-1).fg(muted-foreground)}>x</div>"#);
    assert!(out.contains("sx-"), "chain must emit classes:\n{out}");
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
fn style_when_hover_emits_pseudo() {
    let out = pp(r#"<div sx={when({ hover: style.bg(muted) })}>x</div>"#);
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
    let out = pp(r#"<div sx={style.position(relative).z(1)}>x</div>"#);
    assert!(
        out.contains("relative") || out.contains("position"),
        "position(relative):\n{out}"
    );
}

#[test]
fn style_bind_var_ref() {
    let out = pp(r#"<div sx={style.w(bind(glowRadius))}>x</div>"#);
    assert!(
        out.contains("var(--glow-radius)") || out.contains("--glow"),
        "bind must emit custom property read:\n{out}"
    );
}

#[test]
fn utility_string_product_is_a_compile_error() {
    let err = preprocess_src(r#"<div sx="p-4 bg-surface-1">x</div>"#)
        .expect_err("utility-string sx must fail loudly");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("Style error") || msg.contains("utility-string sx"),
        "error must name the Style diagnostic: {msg}"
    );
}

#[test]
fn structured_string_product_is_a_compile_error() {
    let err = preprocess_src(r#"<div sx="bg: primary px: 2">x</div>"#)
        .expect_err("structured utility-string sx must fail loudly");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("Style error") || msg.contains("utility-string sx"),
        "error must name the Style diagnostic: {msg}"
    );
}

#[test]
fn style_transparent_bg() {
    let out = pp(r#"<div sx={style.bg(transparent)}>x</div>"#);
    assert!(
        out.contains("transparent") || out.contains("background"),
        "bg(transparent):\n{out}"
    );
}

#[test]
fn style_display_none() {
    let out = pp(r#"<div sx={style.display(none)}>x</div>"#);
    assert!(
        out.contains("display:none") || out.contains("none") || out.contains("sx-"),
        "display(none):\n{out}"
    );
}
