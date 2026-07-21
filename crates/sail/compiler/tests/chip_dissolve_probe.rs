//! Chip dissolve — paint source + residue burn gate.
use sail_compiler::compile_element;

#[test]
fn chip_compiles_with_cva_and_no_residue_matrix() {
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
    // No L1 controller / use:chip brain
    assert!(!out.structure_html.contains("use:chip"), "{}", out.structure_html);
    // Variant/size axes from CVA-in-props
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
    // Tone is channel read only — no local --tone-bg-solid brand rewrites
    assert!(
        !out.skin_css.contains("--tone-bg-solid")
            && !out.skin_css.contains("--tone-bg-hover")
            && !out.skin_css.contains("--tone-outline-border"),
        "no local tone channel writes: {}",
        out.skin_css
    );
    // Host base sx present
    assert!(
        out.structure_html.contains("sx-") || out.skin_css.contains("font-family"),
        "host sx base: struct={} skin={}",
        out.structure_html,
        out.skin_css
    );
    // Manifest reflects CVA axes
    let attrs = out.manifest["attributes"].as_array().unwrap();
    let variant = attrs.iter().find(|a| a["name"] == "variant").unwrap();
    assert_eq!(variant["default"], "solid");
}

#[test]
fn chip_residue_section_burned() {
    let residues = include_str!("../stdlib/_residues.css");
    assert!(
        !residues.contains("/* --- chip ")
            && !residues.contains(".sail-chip {")
            && !residues.contains(".sail-chip[")
            && !residues.contains("--chip-h")
            && !residues.contains("--chip-px"),
        "chip residual matrix must be deleted from _residues.css"
    );
    let skins = sail_compiler::stdlib::skins_css();
    // Residual-only channels gone
    assert!(
        !skins.contains("--chip-h")
            && !skins.contains("--chip-px")
            && !skins.contains("--tone-bg-solid")
            && !skins.contains("--tone-bg-hover")
            && !skins.contains("--tone-outline-border")
            && !skins.contains("--tone-ghost-bg-hover"),
        "framework skins must not carry residual chip local channels"
    );
    // Dissolved paint still emits: CVA-in-props attr rules via sx atoms
    assert!(
        skins.contains(".sail-chip")
            && skins.contains("variant='solid'")
            && skins.contains("background-color:var(--tone)")
            && skins.contains("size='md'"),
        "dissolved chip paint must emit via element CVA/sx compile"
    );
}

#[test]
fn badge_and_card_compile_cva_no_residue() {
    let badge = compile_element(include_str!("../stdlib/badge.sail"), "badge").expect("badge");
    assert!(badge.skin_css.contains("variant='solid'"), "{}", badge.skin_css);
    assert!(
        !include_str!("../stdlib/_residues.css").contains("/* --- badge "),
        "badge residue section must be gone"
    );

    let card = compile_element(include_str!("../stdlib/card.sail"), "card").expect("card");
    assert!(
        card.skin_css.contains("variant='outline'") || card.skin_css.contains("padding='md'"),
        "card CVA: {}",
        card.skin_css
    );
    assert!(
        !include_str!("../stdlib/_residues.css").contains("/* --- card "),
        "card residue section must be gone"
    );
}

#[test]
fn alert_compiles_and_residue_burned() {
    let src = include_str!("../stdlib/alert.sail");
    let out = compile_element(src, "alert").expect("alert compiles");
    assert!(
        out.structure_html.contains("use:liveRegion")
            && out.structure_html.contains("use:dismissControl"),
        "capability mix-ins: {}",
        out.structure_html
    );
    assert!(!out.structure_html.contains("use:alert"), "no element brain");
    let residues = include_str!("../stdlib/_residues.css");
    assert!(
        !residues.contains("/* --- alert ")
            && !residues.contains(".sail-alert {")
            && !residues.contains(".sail-alert["),
        "alert residual matrix must be deleted"
    );
    // PE default tone fills moved to contracts
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
    ));
    assert!(
        contracts.contains(".sail-alert:not([tone])")
            && contracts.contains("--tone-bg: var(--tone-muted-bg)"),
        "alert PE defaults in contracts"
    );
}
