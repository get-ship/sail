//! Paint-source gates for L1 dissolve: CVA-in-props + host sx, no residual
//! local tone channels. Does not eulogize deleted intermediate files.
use sail_compiler::compile_element;

#[test]
fn chip_compiles_with_cva_and_capability_mixins() {
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
    assert!(!out.structure_html.contains("use:chip"), "no element brain: {}", out.structure_html);
    assert!(
        out.skin_css.contains("variant='solid'")
            && out.skin_css.contains("variant='soft'")
            && out.skin_css.contains("background-color:var(--tone)"),
        "solid via sx atoms: {}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("size='sm'")
            && (out.skin_css.contains("min-height:var(--control-h-2xs)")
                || out.skin_css.contains("control-h-2xs")),
        "size via sx atoms: {}",
        out.skin_css
    );
    assert!(
        !out.skin_css.contains("--tone-bg-solid")
            && !out.skin_css.contains("--tone-bg-hover")
            && !out.skin_css.contains("--tone-outline-border"),
        "no local tone channel writes: {}",
        out.skin_css
    );
    let attrs = out.manifest["attributes"].as_array().unwrap();
    let variant = attrs.iter().find(|a| a["name"] == "variant").unwrap();
    assert_eq!(variant["default"], "solid", "PE solid default: {variant}");
    let size = attrs.iter().find(|a| a["name"] == "size").unwrap();
    assert_eq!(size["default"], "md", "PE size=md default: {size}");
    assert!(
        out.structure_html.contains("sx-minhcontrolhxs")
            && out.structure_html.contains("sx-px25")
            && out.structure_html.contains("sx-textsm")
            && out.structure_html.contains("sx-bgtone"),
        "attr-absent PE size=md + solid base on host sx: {}",
        out.structure_html
    );
}

#[test]
fn chip_framework_skins_have_no_local_tone_matrix() {
    let skins = sail_compiler::stdlib::skins_css();
    assert!(
        !skins.contains("--chip-h")
            && !skins.contains("--chip-px")
            && !skins.contains("--tone-bg-solid")
            && !skins.contains("--tone-bg-hover")
            && !skins.contains("--tone-outline-border")
            && !skins.contains("--tone-ghost-bg-hover"),
        "framework skins must not carry residual chip local channels"
    );
    assert!(
        skins.contains(".sail-chip")
            && skins.contains("variant='solid'")
            && skins.contains("background-color:var(--tone)")
            && skins.contains("size='md'"),
        "chip paint must emit via element CVA/sx compile"
    );
}

#[test]
fn badge_and_card_compile_cva() {
    let badge = compile_element(include_str!("../stdlib/badge.sail"), "badge").expect("badge");
    assert!(badge.skin_css.contains("variant='solid'"), "{}", badge.skin_css);
    let battrs = badge.manifest["attributes"].as_array().unwrap();
    let bsize = battrs.iter().find(|a| a["name"] == "size").unwrap();
    assert_eq!(bsize["default"], "md", "PE badge size=md: {bsize}");
    // Style product: scope consts + match → atomic classes (not residual tone-* utilities).
    assert!(
        badge.structure_html.contains("sx-minHcontrolh2xs")
            && badge.structure_html.contains("sx-padX2")
            && (badge.structure_html.contains("sx-variantsoft-bgmuted")
                || badge.structure_html.contains("sx-bgmuted")),
        "badge Style path size=md + soft paint: {}",
        badge.structure_html
    );

    // Card still residual (CVA maps + utility host sx) — unions type, no residual paint.
    // Full Style migration of card is out of badge proving-ground scope.
    let card = compile_element(include_str!("../stdlib/card.sail"), "card").expect("card");
    let cattrs = card.manifest["attributes"].as_array().unwrap();
    let pad = cattrs.iter().find(|a| a["name"] == "padding").unwrap();
    assert_eq!(pad["default"], "md", "PE card padding=md: {pad}");
    let variant = cattrs.iter().find(|a| a["name"] == "variant").unwrap();
    assert_eq!(variant["default"], "default", "residual CVA first key default: {variant}");
    assert!(
        card.structure_html.contains("header") || card.structure_html.contains("slot"),
        "card structure still compiles: {}",
        card.structure_html
    );
    // Residual CVA/compound paint is deleted (Style product only). Empty skin is OK here.
    assert!(
        !card.skin_css.contains("border-inline-start-width:3px"),
        "residual status-rail CVA paint must not emit: {}",
        card.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
    ));
    assert!(
        !contracts.contains(".sail-card")
            && !contracts.contains("--card-px")
            && !contracts.contains("border-left-width: 3px")
            && !contracts.contains("border-left-color: var(--tone-fg)"),
        "card PE fully left contracts"
    );
}

#[test]
fn alert_compiles_with_capability_mixins() {
    let src = include_str!("../stdlib/alert.sail");
    let out = compile_element(src, "alert").expect("alert compiles");
    assert!(
        out.structure_html.contains("use:liveRegion")
            && out.structure_html.contains("use:dismissControl"),
        "capability mix-ins: {}",
        out.structure_html
    );
    assert!(!out.structure_html.contains("use:alert"), "no element brain");
    assert!(
        src.contains("before:bg-muted-foreground")
            && src.contains("default: 'before:bg-muted-foreground'"),
        "soft-status muted default is host/CVA, not contracts channel PE"
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
    ));
    assert!(
        !contracts.contains(".sail-alert:not([tone])")
            && !contracts.contains("grid-template-columns: auto 1fr auto"),
        "alert PE defaults/grid must be gone from contracts"
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
            && out.skin_css.contains("box-shadow:var(--shadow-xs)"),
        "solid keeps shadow-xs: {}",
        out.skin_css
    );
}
