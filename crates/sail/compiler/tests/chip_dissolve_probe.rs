//! Compile probes for chip / badge / card / alert on the Style product path.
use sail_compiler::compile_element;

#[test]
fn chip_compiles_with_style_and_capability_mixins() {
    let src = include_str!("../stdlib/chip.sail");
    let out = compile_element(src, "chip").expect("chip compiles");
    assert_eq!(out.root_class, "sail-chip");
    assert!(
        out.structure_html.contains("use:pressable")
            && out.structure_html.contains("use:pressed")
            && out.structure_html.contains("use:dismissible"),
        "capability mix-ins preserved: {}",
        out.structure_html
    );
    assert!(
        !out.structure_html.contains("use:chip"),
        "no element brain: {}",
        out.structure_html
    );
    assert!(
        out.skin_css.contains("variant='solid'")
            && out.skin_css.contains("variant='soft'")
            && (out.skin_css.contains("var(--chip)")
                || out.skin_css.contains("var(--primary)")
                || out.structure_html.contains("s-bgchip")),
        "variant paint via Style tokens: {}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("size='sm'")
            && (out.skin_css.contains("min-height:var(--control-h-2xs)")
                || out.skin_css.contains("control-h-2xs")),
        "size via Style: {}",
        out.skin_css
    );
    let attrs = out.manifest["attributes"].as_array().unwrap();
    let variant = attrs.iter().find(|a| a["name"] == "variant").unwrap();
    assert_eq!(variant["default"], "solid", "PE solid default: {variant}");
    let size = attrs.iter().find(|a| a["name"] == "size").unwrap();
    assert_eq!(size["default"], "md", "PE size=md default: {size}");
    // tone is optional — no default member / no forced tone attr
    let tone = attrs.iter().find(|a| a["name"] == "tone");
    if let Some(tone) = tone {
        assert!(
            tone.get("default").is_none()
                || tone["default"].is_null()
                || tone["default"] == serde_json::Value::Null,
            "tone must not default to a fake member: {tone}"
        );
    }
    assert!(
        out.structure_html.contains("s-")
            && (out.structure_html.contains("s-minhcontrolhxs")
                || out.skin_css.contains("control-h-xs")
                || out.structure_html.contains("controlhxs")),
        "attr-absent PE size=md + solid base: {}",
        out.structure_html
    );
}

#[test]
fn chip_framework_skins_emit_chip_paint() {
    let skins = sail_compiler::stdlib::skins_css();
    assert!(
        skins.contains(".sail-chip")
            && skins.contains("variant='solid'")
            && (skins.contains("var(--chip)") || skins.contains("var(--primary)"))
            && skins.contains("size='md'"),
        "chip paint must emit via Style compile"
    );
}

#[test]
fn badge_and_card_compile_style() {
    let badge = compile_element(include_str!("../stdlib/badge.sail"), "badge").expect("badge");
    assert!(
        badge.skin_css.contains("variant='solid'"),
        "{}",
        badge.skin_css
    );
    let battrs = badge.manifest["attributes"].as_array().unwrap();
    let bsize = battrs.iter().find(|a| a["name"] == "size").unwrap();
    assert_eq!(bsize["default"], "md", "PE badge size=md: {bsize}");
    assert!(
        badge.structure_html.contains("s-")
            && (badge.structure_html.contains("s-minhcontrolh2xs")
                || badge.skin_css.contains("control-h-2xs")),
        "attr-absent PE badge size=md + soft base: {}",
        badge.structure_html
    );

    let card = compile_element(include_str!("../stdlib/card.sail"), "card").expect("card");
    assert!(
        card.skin_css.contains("variant='outline'") && card.skin_css.contains("padding='md'"),
        "card Style match: {}",
        card.skin_css
    );
    let cattrs = card.manifest["attributes"].as_array().unwrap();
    let pad = cattrs.iter().find(|a| a["name"] == "padding").unwrap();
    assert_eq!(pad["default"], "md", "PE card padding=md: {pad}");
    assert!(
        card.structure_html.contains("s-")
            && (card.structure_html.contains("s-py4") || card.skin_css.contains("padding")),
        "attr-absent PE card padding=md + surface base: {}",
        card.structure_html
    );
    assert!(
        card.skin_css.contains("status-rail")
            && (card.skin_css.contains("border-inline-start")
                || card.skin_css.contains("border-strong")),
        "status-rail Style compound: {}",
        card.skin_css
    );
}

#[test]
fn alert_compiles_with_style_capability_mixins() {
    let src = include_str!("../stdlib/alert.sail");
    let out = compile_element(src, "alert").expect("alert compiles");
    assert!(
        out.structure_html.contains("use:liveRegion")
            && out.structure_html.contains("use:dismissControl"),
        "capability mix-ins: {}",
        out.structure_html
    );
    assert!(
        !out.structure_html.contains("use:alert"),
        "no element brain"
    );
    assert!(
        out.skin_css.contains("::before")
            && (out.skin_css.contains("var(--alert-stripe)")
                || out.skin_css.contains("alert-stripe")),
        "stripe PE on ::before: {}",
        out.skin_css
    );
}

#[test]
fn chip_non_solid_variants_clear_solid_shadow() {
    let out = compile_element(include_str!("../stdlib/chip.sail"), "chip").expect("chip");
    for key in ["soft", "outline", "ghost"] {
        assert!(
            out.skin_css.contains(&format!("variant='{key}'")),
            "missing {key}: {}",
            out.skin_css
        );
    }
    assert!(
        out.skin_css.contains("variant='soft'") && out.skin_css.contains("box-shadow:none"),
        "non-solid variants must clear solid host shadow-xs: {}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("variant='solid'")
            && (out.skin_css.contains("box-shadow:var(--shadow-xs)")
                || out.skin_css.contains("shadow-xs")),
        "solid keeps shadow-xs: {}",
        out.skin_css
    );
}
