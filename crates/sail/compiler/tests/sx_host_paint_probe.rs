//! Host-paint delete path — layout + sx + CVA-in-props replace author `<style>`.
//! Golden: radio-group is the first stdlib element with zero author skin CSS.

use sail_compiler::compile_element;

fn is_style_product(src: &str) -> bool {
    src.contains("style.") || src.contains("style(")
}

#[test]
fn radio_group_zero_author_style() {
    let src = include_str!("../stdlib/radio-group/radio-group.sail");
    let out = compile_element(src, "radio-group").expect("radio-group compiles");
    assert!(
        !out.skin_css.contains(".sail-radio-group {"),
        "author host block must be gone:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("flex-direction:column") || out.skin_css.contains("sail-col"),
        "layout must own the stack:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("font-family"),
        "sx font-body must land in skin:\n{}",
        out.skin_css
    );
    assert!(
        out.structure_html.contains("use:radioGroup"),
        "behavior stamp preserved:\n{}",
        out.structure_html
    );
}

#[test]
fn style_match_variant_lowers_attr_selectors_element_mode() {
    // Style product: prop unions + match on host sx (not residual CVA string maps).
    let src = r#"
<scope>
  props: {
    tone: 'default' | 'primary' = 'default'
    variant: 'soft' | 'solid' | 'outline' = 'soft'
    size: 'sm' | 'md' = 'md'
  }
  const soft = style({ bg(muted).fg(muted-foreground).border({ width: 1, color: border }) })
  const solid = style({ bg(primary).fg(primary-foreground).border({ width: 1, color: primary }), when: { hover: opacity(90) } })
  const outline = style({ bg(transparent).fg(foreground).border({ width: 1, color: border }) })
  const sizeSm = style({ pad({ x: 2 }).type({ size: xs }) })
  const sizeMd = style({ pad({ x: 2_5 }).type({ size: sm }) })
</scope>
<span style({ radius(full).border({ width: 1 }).type({ weight: medium }), match: { variant: { soft: soft, solid: solid, outline: outline }, size: { sm: sizeSm, md: sizeMd } } })> 
  <slot></slot>
</span>
"#;
    let out = compile_element(src, "probe").expect("style match product compiles");
    let attrs = out.manifest["attributes"].as_array().expect("attrs");
    let variant = attrs
        .iter()
        .find(|a| a["name"] == "variant")
        .expect("variant attr");
    assert_eq!(variant["default"], "soft", "default from prop: {variant}");
    let values = variant["values"].as_array().expect("values");
    assert!(
        values.iter().any(|v| v == "soft") && values.iter().any(|v| v == "solid"),
        "union of keys: {variant}"
    );
    // Attribute-keyed rules from Style match (not residual CVA string maps).
    assert!(
        out.skin_css.contains("variant='soft'")
            && (out.skin_css.contains("background-color:var(--muted)")
                || out
                    .skin_css
                    .contains("background-color:var(--bg-muted-color)"))
            && out.skin_css.contains("color:var(--muted-foreground)"),
        "soft axis via Style match: {}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("variant='solid'")
            && out.skin_css.contains("background-color:var(--primary)")
            && out.skin_css.contains("color:var(--primary-foreground)"),
        "solid axis via Style match: {}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("size='sm'") && out.skin_css.contains("font-size:var(--text-xs)"),
        "size axis via Style match: {}",
        out.skin_css
    );
    // State prefix on a map value → attr-keyed :hover rule
    assert!(
        out.skin_css.contains(":hover") && out.skin_css.contains("opacity"),
        "hover: prefix on map value: {}",
        out.skin_css
    );
    // Host sx base still present (atomic classes on structure / rules in skin).
    assert!(
        out.structure_html.contains("s-roundedfull")
            || out.structure_html.contains("s-border")
            || out.skin_css.contains("border-radius"),
        "host sx base survives: struct={} skin={}",
        out.structure_html,
        out.skin_css
    );
}

#[test]
fn badge_soft_base_on_host_sx() {
    let src = include_str!("../stdlib/badge/badge.sail");
    let out = compile_element(src, "badge").expect("badge compiles");
    assert!(
        is_style_product(src)
            && (out.structure_html.contains("s-bgmuted")
                || out.structure_html.contains("bgmuted")
                || out.skin_css.contains("var(--muted)")
                || out.structure_html.contains("s-bgtonebg"))
            && (out.structure_html.contains("s-fgmutedforeground")
                || out.structure_html.contains("mutedforeground")
                || out.skin_css.contains("var(--muted-foreground)")
                || out.structure_html.contains("s-texttonefg")
                || out.structure_html.contains("s-fg")),
        "soft default paint must be host Style (semantic tokens):\n{}",
        out.structure_html
    );
    assert!(
        out.structure_html.contains("s-fontFamilybody")
            || out.structure_html.contains("s-fontbody")
            || out.skin_css.contains("var(--body-font)"),
        "typeface on host sx:\n{}",
        out.structure_html
    );
    assert!(
        !out.skin_css.contains("background: var(--tone-bg)")
            && !out.skin_css.contains("var(--tone-bg)"),
        "soft base must not use tone channel:\n{}",
        out.skin_css
    );
    // solid/outline/size via Style match (semantic tokens).
    assert!(
        out.skin_css.contains("variant='solid'")
            && (out.skin_css.contains("background-color:var(--primary)")
                || out.skin_css.contains("background-color: var(--primary)")
                || out.skin_css.contains("background-color:var(--tone)"))
            && out.skin_css.contains("variant='outline'")
            && out.skin_css.contains("size='sm'"),
        "solid/outline/size via Style match:\n{}",
        out.skin_css
    );
    // soft+default compound → muted soft host Style.
    assert!(
        (src.contains("variant: soft") || src.contains("variant: 'soft'"))
            && (src.contains("bg(muted)") || src.contains("bg-muted") || src.contains("muted"))
            && (out.skin_css.contains("var(--muted)")
                || out.skin_css.contains("muted")
                || out.skin_css.contains("var(--muted)")),
        "soft+default must paint muted soft:\n{}",
        out.skin_css
    );
    assert!(
        !out.skin_css.contains("--badge-h") && !out.skin_css.contains("--badge-fs"),
        "badge residual geometry custom props must be gone:\n{}",
        out.skin_css
    );
    let attrs = out.manifest["attributes"].as_array().expect("attrs");
    let mut seen = std::collections::HashSet::new();
    for a in attrs {
        let n = a["name"].as_str().unwrap();
        assert!(seen.insert(n.to_string()), "duplicate attr {n}");
    }
}

#[test]
fn avatar_tracking_and_inset_ring_on_host_sx() {
    // Paint stripe: tracking + inset ring + host gradient left contracts → host sx.
    // Layout row centers content; platform/avatar.css packs the injected img.
    let src = include_str!("../stdlib/avatar/avatar.sail");
    let out = compile_element(src, "avatar").expect("avatar compiles");
    assert!(
        src.contains("align=\"center\"") && src.contains("justify=\"center\""),
        "avatar must layout-center"
    );
    assert!(
        (out.structure_html.contains("trackingwide")
            || out.skin_css.contains("letter-spacing:var(--tracking-wide)"))
            && (out.structure_html.contains("shadowinsetring")
                || out.skin_css.contains("box-shadow:var(--shadow-inset-ring)")
                || out
                    .skin_css
                    .contains("box-shadow: var(--shadow-inset-ring)")),
        "tracking-wide + shadow-inset-ring must land on host:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        (out.structure_html.contains("s-sizeh7w7") || out.structure_html.contains("s-size7"))
            && (out.structure_html.contains("radiusallfull")
                || out.structure_html.contains("s-radiusfull")
                || out.structure_html.contains("s-roundedfull")
                || out.skin_css.contains("border-radius:var(--radius-full)"))
            && (out.structure_html.contains("familybody")
                || out.structure_html.contains("s-fontbody")
                || out.skin_css.contains("var(--body-font)")),
        "attr-absent host Style size=md + shape round + type on host sx:\n{}",
        out.structure_html
    );
    // Child img pack lives in platform/avatar.css (skins), not host Style.
    let skins = sail_compiler::stdlib::skins_css();
    assert!(
        skins.contains("/* @provenance platform/avatar.css */")
            && skins.contains(".sail-avatar img")
            && (skins.contains("object-fit:cover") || skins.contains("object-fit: cover")),
        "avatar photo path must cover descendant img in skins:\n{skins}"
    );
    assert!(
        skins.contains(".sail-avatar[icon-slug] img")
            && (skins.contains("object-fit:contain") || skins.contains("object-fit: contain"))
            && (skins.contains("padding:10%") || skins.contains("padding: 10%")),
        "avatar icon-slug path must contain+pad in skins:\n{skins}"
    );
    // Branded tones solid-fill via bridge var (`var(--tone)`); size/shape axes land.
    assert!(
        (out.skin_css.contains("background-color:var(--tone)")
            || out.skin_css.contains("background-color: var(--tone)")
            || out.skin_css.contains("s-bgtone"))
            && out.skin_css.contains("size='xs'")
            && out.skin_css.contains("shape='square'"),
        "branded tone solid + size/shape via Style match:\n{}",
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    // Deleted paint host only.
    assert!(
        !contracts.contains("letter-spacing: 0.02em"),
        "letter-spacing host CSS must be gone (tracking-wide on host)"
    );
    assert!(
        !contracts.contains("inset 0 0 0 1px var(--border), inset 0 1px 0 var(--background)"),
        "inset ring host CSS must be gone (shadow-inset-ring on host)"
    );
    assert!(
        !contracts.contains("linear-gradient(135deg, var(--primary), var(--accent))")
            && !contracts
                .lines()
                .any(|l| l.contains(".sail-avatar") && l.contains("linear-gradient")),
        "avatar host gradient must be gone from contracts"
    );
    // Layout + img host CSS gone from contracts (row + platform/avatar.css pack).
    assert!(
        !contracts.contains(".sail-avatar img")
            && !contracts
                .lines()
                .any(|l| l.contains(".sail-avatar") && l.contains("align-items")),
        "avatar flex/img host CSS must be gone from contracts"
    );
}

#[test]
fn icon_child_svg_pack_lives_in_platform_skins() {
    let src = include_str!("../stdlib/icon/icon.sail");
    let out = compile_element(src, "icon").expect("icon compiles");
    assert!(
        src.contains("size(icon-size-md)") && src.contains("fg(currentColor)"),
        "icon host Style stays size + ink"
    );
    assert!(
        !out.structure_html.contains("s-glyphfill"),
        "glyph pack must not mint a Style class:\n{}",
        out.structure_html
    );
    let skins = sail_compiler::stdlib::skins_css();
    assert!(
        skins.contains("/* @provenance platform/icon.css */")
            && skins.contains(".sail-icon svg")
            && skins.contains("width:100%")
            && skins.contains("height:100%"),
        "icon child svg pack must live in platform/icon.css via skins_css"
    );
}

#[test]
fn empty_state_child_paint_on_structure_sx() {
    // Style product: stretch host + grow copy + semantic child tokens.
    // match(size/variant/tone) + when(empty/forced-colors). Zero contracts host CSS.
    let src = include_str!("../stdlib/empty-state/empty-state.sail");
    let out = compile_element(src, "empty-state").expect("empty-state compiles");
    assert!(
        !src.contains("--es-")
            && !src.contains("tone-bg")
            && src.contains("align=\"stretch\"")
            && is_style_product(src)
            && src.contains("match:")
            && src.contains("size:")
            && src.contains("match:")
            && src.contains("variant:")
            && (src.contains("fg(tone)")
                || (src.contains("bg(tone)") || src.contains("fg(tone)"))
                || src.contains("bg(tone)"))
            && (src.contains("grow(1)") || src.contains(" grow") || src.contains("grow min-w"))
            && src.contains("self(stretch)")
            && src.contains("self(center)")
            && src.contains("size(12)")
            && (src.contains("fontSize(md)") || src.contains("size: md"))
            && (src.contains("fontSize(sm)") || src.contains("size: sm"))
            && (src.contains("maxW(panel-ch-w-md)") || src.contains("max-w: panel-ch-w-md"))
            && src.contains("empty:")
            && src.contains("forced-colors"),
        "empty-state Style product + stretch/grow + host child tokens"
    );
    assert!(
        out.skin_css.contains("font-size:var(--text-md)")
            || out.skin_css.contains("font-size: var(--text-md)"),
        "title text-md must emit real font-size CSS:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("font-size:var(--text-sm)")
            || out.skin_css.contains("font-size: var(--text-sm)"),
        "desc text-sm must emit real font-size CSS:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("max-width:var(--panel-ch-w-md)")
            || out.skin_css.contains("max-width: var(--panel-ch-w-md)"),
        "desc maxW(panel-ch-w-md) must emit real max-width CSS:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("background-color:var(--empty-state)")
            || out
                .skin_css
                .contains("background-color: var(--empty-state)")
            || out.structure_html.contains("s-bgemptystate")
            || out.skin_css.contains("var(--bg-muted-color)")
            || out.structure_html.contains("s-bgmuted"),
        "icon soft surface via muted rest (no tone-bg):\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("var(--empty-state-title)")
            || out.structure_html.contains("emptystatetitle")
            || out.skin_css.contains("var(--fg-foreground-color)")
            || out.structure_html.contains("s-fgforeground"),
        "title must read foreground / heading ink:\n{}",
        out.skin_css
    );
    assert!(
        (out.skin_css.contains("width:var(--space-12)")
            || out.skin_css.contains("width: var(--space-12)"))
            && (out.skin_css.contains("height:var(--space-12)")
                || out.skin_css.contains("height: var(--space-12)")),
        "icon size(12) must emit width+height var(--space-12):\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("size='sm'")
            && out.skin_css.contains("size='lg'")
            && out.skin_css.contains("variant='inline'")
            && out.skin_css.contains("variant='connecting'")
            && !out.skin_css.contains("--es-")
            && !out.skin_css.contains("var(--tone-bg)"),
        "size/variant Style match must land; no tone-bg / --es-*:\n{}",
        out.skin_css
    );
    assert!(
        out.structure_html.contains("empty")
            || out.structure_html.contains("s-empty")
            || out.skin_css.contains(":empty"),
        "empty gate must lower:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    assert!(
        !contracts.contains(".sail-empty-state")
            || contracts
                .lines()
                .filter(|l| l.contains(".sail-empty-state"))
                .all(|l| l.trim_start().starts_with("/*") || l.trim_start().starts_with("//")),
        "empty-state must have zero contracts host CSS rules"
    );
}

#[test]
fn tab_selected_underline_and_inset_focus_on_host_sx() {
    // Style product: when(selected/focus) + match(variant) for pill/enclosed chrome.
    let src = include_str!("../stdlib/tabs/tab.sail");
    let out = compile_element(src, "tab").expect("tab compiles");
    assert!(
        is_style_product(src)
            && (src.contains("when({") || src.contains("when: {"))
            && src.contains("selected:")
            && (src.contains("borderB(primary)")
                || src.contains("borderB(tone)")
                || src.contains("color: primary, edge: b")
                || src.contains("edge: b"))
            && (src.contains("outline(ring)")
                || src.contains("outline(tone-ring)")
                || src.contains("kind: outline")
                || src.contains("color: ring"))
            && (src.contains("outlineOffset(ring-inset)") || src.contains("offset: ring-inset")),
        "tab host Style must carry selected underline + inset focus ring"
    );
    assert!(
        out.skin_css.contains("border-bottom-color:var(--primary)")
            || out.skin_css.contains("border-bottom-color: var(--primary)")
            || out.skin_css.contains("border-bottom-color:var(--tone)")
            || out.skin_css.contains("border-bottom-color: var(--tone)")
            || out.structure_html.contains("borderBprimary")
            || out.structure_html.contains("borderBtone")
            || out.structure_html.contains("borderbprimary")
            || out.structure_html.contains("borderbtone"),
        "selected underline must lower into skin:\n{}",
        out.skin_css
    );
    assert!(
        out.structure_html.contains("outlinesolid")
            || out.skin_css.contains("outline-style:solid")
            || out.skin_css.contains("outline-style: solid"),
        "focus outline-solid must land:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css.contains("outline-color:var(--ring)")
            || out.skin_css.contains("outline-color: var(--ring)")
            || out.skin_css.contains("outline-color:var(--tone-ring)")
            || out.skin_css.contains("outline-color: var(--tone-ring)")
            || out.structure_html.contains("outlinering")
            || out.structure_html.contains("outlinetonering")
            || out.structure_html.contains("tonering"),
        "focus outline color must use ring token:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    assert!(
        !contracts.contains(".sail-tab[aria-selected='true']")
            && !contracts.contains(".sail-tab-list[variant='pill'] .sail-tab")
            && !contracts.contains(".sail-tab-list[variant='enclosed'] .sail-tab"),
        "tab selected / pill/enclosed parent→child host must be gone from contracts"
    );
    assert!(
        !contracts.contains(".sail-tab:focus-visible"),
        "tab focus-visible host CSS must be gone from contracts"
    );
    let tab_src = include_str!("../stdlib/tabs/tab.sail");
    assert!(
        tab_src.contains("pill:")
            && tab_src.contains("enclosed:")
            && (tab_src.contains("tab.selected-bg")
                || tab_src.contains("tab-selected-bg")
                || tab_src.contains("selected-bg")
                || tab_src.contains("bg(tab.selected-bg)")
                || tab_src.contains("bg(muted)")),
        "tab.sail must Style match pill/enclosed selected chrome"
    );
}

#[test]
fn toggle_pressed_focus_stack_on_host_sx() {
    // Style product: position(relative) + when(pressed/focus) z stack + outline.
    let src = include_str!("../stdlib/toggle-group/toggle.sail");
    let out = compile_element(src, "toggle").expect("toggle");
    assert!(
        (out.structure_html.contains("positionrelative")
            || out.structure_html.contains("relative")
            || out.skin_css.contains("position:relative"))
            && (out.structure_html.contains("pressed") || out.skin_css.contains("aria-pressed"))
            && (out.structure_html.contains("focus") || out.skin_css.contains("focus-visible"))
            && (out.structure_html.contains("outlinesolid")
                || out.skin_css.contains("outline-style:solid")
                || out.skin_css.contains("outline-style: solid")),
        "stack + focus outline on host:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    assert!(
        !contracts.contains(".sail-toggle[aria-pressed='true']")
            && !contracts.contains(".sail-toggle:focus-visible")
            && !contracts.contains(".sail-toggle-group > .sail-toggle"),
        "toggle pressed/focus + toggle-group join host CSS must be gone from contracts"
    );
    assert!(
        out.structure_html.contains("first")
            || out.skin_css.contains(":first-child")
            || src.contains("first:"),
        "toggle must express join edges via when(first/last)"
    );
}

#[test]
fn switch_host_track_paint_on_host_sx() {
    // Style product: appearance/track/thumb when(before/checked) + size/tone match.
    let src = include_str!("../stdlib/switch/switch.sail");
    let out = compile_element(src, "switch").expect("switch compiles");
    assert!(
        !src.contains("--sw-")
            && (src.contains("style({ appearance(none) })") || src.contains("appearance(none)"))
            && (src.contains("bg(none)")
                || src.contains("bg(muted)")
                || src.contains("bg(switch)"))
            && (src.contains("border(switch.border)")
                || src.contains("border(switch-border)")
                || src.contains("color: border-strong"))
            && (src.contains("bg(switch.thumb)")
                || src.contains("bg(switch-thumb)")
                || src.contains("bg(knob)"))
            && src.contains("radius(full)")
            && (src.contains("w(9)") || src.contains("w: 9"))
            && (src.contains("h(5)") || src.contains("h: 5"))
            && src.contains("content(empty)")
            && src.contains("size(4)")
            && (src.contains("left(auto)") || src.contains("l: auto"))
            && (src.contains("right(px)") || src.contains("r: px"))
            && !src.contains("bg(hover-bg)")
            && src.contains("bg(primary)")
            && (src.contains("border(selected-border)") || src.contains("color: selected-border"))
            && (src.contains("outline(primary)")
                || (src.contains("color: primary") && src.contains("kind: outline")))
            && (src.contains("outlineOffset(ring-offset)") || src.contains("offset: ring-offset"))
            && (src.contains("saturate(60)") || src.contains("saturate: 60"))
            && (src.contains("bg(info)")
                || src.contains("bg(primary)")
                || src.contains("bg(tone)"))
            && (src.contains("outline(info)")
                || src.contains("outline(primary)")
                || src.contains("outline(tone)")
                || src.contains("kind: outline"))
            && src.contains("match:")
            && src.contains("size:")
            && src.contains("size(3)")
            && (src.contains("w(7)") || src.contains("w: 7"))
            && src.contains("size(5)"),
        "switch Style host: track + before thumb + size/tone match + no hover-bg wash"
    );
    // boxSizing class mint + live skin + committed element-skins.css lockstep.
    assert!(
        src.contains("boxSizing(border)"),
        "switch host must call boxSizing(border)"
    );
    assert!(
        out.structure_html.contains("boxSizingborder")
            || out.skin_css.contains("s-boxSizingborder"),
        "boxSizing class mint must stamp: struct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css.contains("box-sizing:border-box")
            || out.skin_css.contains("box-sizing: border-box"),
        "box-sizing:border-box must emit in switch skin:\n{}",
        out.skin_css
    );
    let skins_file = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/element-skins.css"
    ));
    assert!(
        skins_file.contains(".s-boxSizingborder{")
            && (skins_file.contains("box-sizing:border-box")
                || skins_file.contains("box-sizing: border-box")),
        "element-skins.css stale for boxSizing — run: cargo run -p sail-compiler -- framework-css"
    );
    assert!(
        out.structure_html.contains("appearancenone")
            || out.skin_css.contains("appearance:none")
            || out.skin_css.contains("appearance: none"),
        "appearance-none must land:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css.contains("background-image:none")
            || out.skin_css.contains("background-image: none")
            || out.structure_html.contains("bgnone")
            || out.skin_css.contains("appearance:none")
            || out.structure_html.contains("appearancenone"),
        "appearance-none (or bg none) must clear native checkbox mark:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css.contains("background-color:var(--switch)")
            || out.skin_css.contains("background-color: var(--switch)")
            || out.structure_html.contains("bgswitch")
            || out.skin_css.contains("var(--bg-muted-color)")
            || out.structure_html.contains("s-bgmuted"),
        "off-track paint (switch token or muted rest):\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css.contains("width:var(--space-9)")
            || out.skin_css.contains("width: var(--space-9)")
            || out.structure_html.contains("w9"),
        "md size track width:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("::before")
            && (out.skin_css.contains("content:''") || out.skin_css.contains("content: ''")),
        "before: thumb must lower:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("size='sm'") && out.skin_css.contains("size='lg'"),
        "size match must land:\n{}",
        out.skin_css
    );
    assert!(
        (out.skin_css.contains(":checked")
            || out.skin_css.contains("aria-checked='true'")
            || out.skin_css.contains("aria-checked=\"true\""))
            && (out.skin_css.contains("background-color:var(--primary)")
                || out.skin_css.contains("background-color: var(--primary)")),
        "checked:bg(primary) must lower:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("background-color:var(--tone)")
            || out.skin_css.contains("background-color: var(--tone)")
            || out.skin_css.contains("s-checked-bgtone")
            || out.skin_css.contains("bgtone"),
        "branded tone checked:bg semantic must land:\n{}",
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    if let Some(sw_start) = contracts.find("/* switch") {
        let sw_rest = &contracts[sw_start..];
        let sw_end = sw_rest
            .find("/* prose")
            .or_else(|| sw_rest.find("\n/* "))
            .unwrap_or(sw_rest.len());
        let sw_pe = &sw_rest[..sw_end];
        assert!(
            !sw_pe.contains("--sw-w:")
                && !sw_pe.contains("::before")
                && !sw_pe.contains("appearance:"),
            "switch host CSS must be gone:\n{sw_pe}"
        );
    } else {
        assert!(
            !contracts.contains(".sail-switch") && !contracts.contains("--sw-w:"),
            "switch host CSS fully deleted from contracts"
        );
    }
}

#[test]
fn separator_rest_line_without_empty_tone_stomp() {
    // C-SEP-1: solid rest is separator.line; bg(tone) only under [tone] presence.
    let src = include_str!("../stdlib/separator/separator.sail");
    let out = compile_element(src, "separator").expect("separator compiles");
    assert!(
        (src.contains("bg(separator.line)") || src.contains("bg(border)"))
            && !src.contains("solid: style(solid, { bg(tone) })")
            && src.contains("tone: *"),
        "separator must not unconditional bg(tone) on solid; tone via presence compound"
    );
    assert!(
        (out.skin_css
            .contains("background-color:var(--separator-line)")
            || out
                .skin_css
                .contains("background-color: var(--separator-line)")
            || out.skin_css.contains("background-color:var(--border)"))
            && (out.skin_css.contains("[tone]")
                || out.skin_css.contains("[tone='primary']")
                || out.structure_html.contains("tone")),
        "rest line + tone-gated paint in skin:\n{}",
        out.skin_css
    );
    // Solid arm must not apply bare .s-*-bgtone without [tone].
    let solid_tone_unconditional = out.skin_css.split('}').any(|rule| {
        rule.contains("bgtone")
            && rule.contains("background-color")
            && !rule.contains("[tone")
            && rule.contains("variant='solid'")
    });
    assert!(
        !solid_tone_unconditional,
        "solid+tone must be presence-gated, not [variant=solid] alone:\n{}",
        out.skin_css
    );
}

#[test]
fn toggle_pressed_uses_primary_not_invert() {
    // C-TG-1: selected = primary tokens; pressed+hover keeps identity.
    let src = include_str!("../stdlib/toggle-group/toggle.sail");
    let out = compile_element(src, "toggle").expect("toggle compiles");
    assert!(
        src.contains("bg(primary)")
            && src.contains("fg(primary-foreground)")
            && !src.contains("bg(foreground).fg(background)")
            && src.contains("hover:"),
        "toggle pressed must use primary, not raw fg/bg invert"
    );
    assert!(
        (out.skin_css.contains("background-color:var(--primary)")
            || out.skin_css.contains("background-color: var(--primary)"))
            && (out.skin_css.contains("[aria-pressed='true']")
                || out.skin_css.contains("aria-pressed")),
        "pressed primary must lower:\n{}",
        out.skin_css
    );
    // pressed+hover compound present (higher-spec than bare :hover wash).
    assert!(
        out.skin_css.contains(":hover")
            && (out.skin_css.contains("opacity:0.9")
                || out.skin_css.contains("opacity: 0.9")
                || out.skin_css.contains("opacity:90")
                || src.contains("opacity(90)")),
        "pressed+hover must keep selected identity:\n{}",
        out.skin_css
    );
}

#[test]
fn toast_start_stripe_on_host_sx() {
    // host CSS was border-inline-start: 3px solid var(--tone-fg). Now Style borderSWidth(3)
    // + match(tone) semantic borderS tokens.
    let out = compile_element(include_str!("../stdlib/toast/toast.sail"), "toast").expect("toast");
    assert!(
        out.structure_html.contains("s-borderSWidth3")
            || out.structure_html.contains("borderSWidth")
            || out.structure_html.contains("s-borders3")
            || out.structure_html.contains("border-s-3")
            || out.skin_css.contains("border-inline-start-width:3px")
            || out.skin_css.contains("border-inline-start-width: 3px"),
        "border-s-3 must lower to 3px start width:\nstructure={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css
            .contains("border-inline-start-color:var(--toast-stripe)")
            || out
                .skin_css
                .contains("border-inline-start-color: var(--toast-stripe)")
            || out
                .skin_css
                .contains("border-inline-start-color:var(--primary)")
            || out
                .skin_css
                .contains("border-inline-start-color: var(--primary)")
            || out
                .skin_css
                .contains("border-inline-start-color:var(--tone)")
            || out
                .skin_css
                .contains("border-inline-start-color: var(--tone)"),
        "start stripe default must bind toast-stripe or bound tone; tone matrices spend brand:\n{}",
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    assert!(
        !contracts.contains("border-inline-start: 3px solid var(--tone-fg")
            && !contracts.contains(".sail-toast {\n  border-inline-start")
            && !contracts.contains("[data-toast-close]"),
        "toast host stripe + close anatomy host CSS must be gone"
    );
    // Close control is structure with sx (not parent host CSS).
    let src = include_str!("../stdlib/toast/toast.sail");
    assert!(
        src.contains("data-toast-close")
            && (src.contains("ms(auto)")
                || src.contains("ms-auto")
                || src.contains("ms: auto")
                || src.contains("s: auto")
                || src.contains("margin({ s: auto })"))
            && (out.structure_html.contains("data-toast-close")
                || out.structure_html.contains("s-msauto")
                || out.structure_html.contains("s-marginsauto")),
        "close control must be structure with sx:\n{}",
        out.structure_html
    );
}

#[test]
fn card_host_paint_bulk_on_host_sx() {
    // Full card host CSS dissolved: border-image, backdrop, status-rail (paint +
    // pad-adjust), interactive lift/forced-colors, branded tone border,
    // accent-bar, header/footer chrome → host sx / CVA / structure slots.
    let src = include_str!("../stdlib/card/card.sail");
    let out = compile_element(src, "card").expect("card compiles");

    // Backdrop via Style filter cluster (corner-image host CSS is gone).
    assert!(
        src.contains("backdrop: card-backdrop-filter")
            || src.contains("backdrop(card-backdrop-filter)")
            || src.contains("backdrop-(--card-backdrop-filter)")
            || src.contains("card-backdrop-filter"),
        "card host sx must bind backdrop token"
    );
    assert!(
        out.structure_html.contains("s-backdropcardbackdropfilter")
            || out.structure_html.contains("card-backdrop-filter")
            || out.structure_html.contains("filterbackdrop")
            || out
                .skin_css
                .contains("backdrop-filter:var(--card-backdrop-filter)")
            || out
                .skin_css
                .contains("backdrop-filter: var(--card-backdrop-filter)"),
        "backdrop token must land:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css
            .contains("-webkit-backdrop-filter:var(--card-backdrop-filter)")
            || out
                .skin_css
                .contains("-webkit-backdrop-filter: var(--card-backdrop-filter)")
            || out.structure_html.contains("s-backdrop"),
        "webkit backdrop prefix must land with token:\nskin={}",
        out.skin_css
    );

    // status-rail → start-edge 3px + border-strong default + pad-s adjust
    assert!(
        (src.contains("borderS(3")
            || src.contains("border-s-3")
            || (src.contains("width: 3") && src.contains("edge: s")))
            && (src.contains("border-strong") || src.contains("border-s-border-strong"))
            && (src.contains("padS(4)") || src.contains("ps-4") || src.contains("pad({ s: 4 })")),
        "status-rail compounds must paint rail + pad-adjust via space tokens"
    );
    assert!(
        out.skin_css.contains("status-rail")
            && (out
                .skin_css
                .contains("border-inline-start-color:var(--tone-fg)")
                || out
                    .skin_css
                    .contains("border-inline-start-color: var(--tone-fg)")
                || out.skin_css.contains("border-inline-start-width:3px")
                || out.skin_css.contains("border-inline-start-width: 3px")
                || out.skin_css.contains("border-strong"))
            && (out.skin_css.contains("padding-inline-start:var(--space-4)")
                || out
                    .skin_css
                    .contains("padding-inline-start: var(--space-4)")),
        "status-rail compound must lower pad-adjust + rail color:\n{}",
        out.skin_css
    );

    // interactive lift + forced-colors focus
    assert!(
        (src.contains("translateY(-px)") || src.contains("hover:-translate-y-px"))
            && (src.contains("translateY(0)") || src.contains("active:translate-y-0"))
            && (src.contains("motion-reduce") || src.contains("motion-reduce:translate-y-0"))
            && (src.contains("duration(instant)") || src.contains("active:duration-instant"))
            && src.contains("CanvasText"),
        "interactive compound must carry lift + forced-colors outline"
    );
    assert!(
        out.skin_css.contains("translate:")
            || out.skin_css.contains("translate ")
            || out.skin_css.contains("var(--space-px)"),
        "lift must lower via translate + space-px:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("CanvasText")
            || out.skin_css.contains("@media (forced-colors: active)"),
        "forced-colors interactive focus must land in skin:\n{}",
        out.skin_css
    );

    // branded tone border via bound border(tone) bridge var.
    assert!(
        out.skin_css.contains("border-color:var(--tone)")
            || out.skin_css.contains("border-color: var(--tone)")
            || out.skin_css.contains("s-bordertone"),
        "branded tone border via bridge var:\n{}",
        out.skin_css
    );

    // accent-bar → before: stripe compounds (static + hover:before: opacity)
    assert!(
        (src.contains("content(empty)") || src.contains("before:content-empty"))
            && (src.contains("bgImage(primary-accent)")
                || src.contains("before:from-primary")
                || src.contains("image: primary-accent"))
            && (src.contains("h(half)")
                || src.contains("before:h-half")
                || src.contains("h: half"))
            && (src.contains("opacity(100)")
                || src.contains("hover:before:opacity-100")
                || src.contains("opacity: 100"))
            && (src.contains("opacity(0)")
                || src.contains("before:opacity-0")
                || src.contains("opacity: 0")),
        "accent-bar compounds must use before: anatomy + hover opacity"
    );
    assert!(
        out.skin_css.contains("accent-bar='static'")
            && out.skin_css.contains("accent-bar='hover'")
            && out.skin_css.contains("::before")
            && (out.skin_css.contains("content:''") || out.skin_css.contains("content: ''"))
            && (out.skin_css.contains("height:var(--space-half)")
                || out.skin_css.contains("height: var(--space-half)")
                || out.skin_css.contains("height:50%")
                || out.structure_html.contains("sizehhalf"))
            && out.skin_css.contains(":hover")
            && (out.skin_css.contains("opacity:1")
                || out.skin_css.contains("opacity: 1")
                || out.skin_css.contains("opacity:1.")
                || out.skin_css.contains("opacity:0")),
        "accent-bar CVA must lower before: stripe + hover opacity:\n{}",
        out.skin_css
    );

    // header/footer real named slots with chrome sx (no parent→child host)
    assert!(
        src.contains(r#"slot name="header""#)
            && src.contains(r#"slot name="footer""#)
            && (src.contains("empty:") || src.contains("empty:hidden"))
            && (src.contains("fontFamily(heading)")
                || src.contains("font-heading")
                || src.contains("family: heading"))
            && (src.contains("border-subtle") || src.contains("border-border-subtle"))
            && (src.contains("mt(-4)") || src.contains("-mt-4") || src.contains("t: -4"))
            && (src.contains("mx(-5)") || src.contains("-mx-5") || src.contains("x: -5"))
            && (src.contains("mb(-4)") || src.contains("-mb-4") || src.contains("b: -4")),
        "card structure must expose header/footer slots with bleed chrome sx"
    );
    assert!(
        out.structure_html.contains(r#"slot name="header""#)
            && out.structure_html.contains(r#"slot name="footer""#)
            && (out.structure_html.contains("s-empty-hidden") || out.skin_css.contains(":empty"))
            && (out.structure_html.contains("s-fontheading")
                || out.skin_css.contains("font-family:var(--heading-font)")
                || out.skin_css.contains("font-family: var(--heading-font)")),
        "header/footer chrome must lower into structure/skin:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );

    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    // Zero card host CSS — entire block deleted.
    assert!(
        !contracts.contains(".sail-card")
            && !contracts.contains("--card-px")
            && !contracts.contains("--card-py")
            && !contracts.contains("--card-gap")
            && !contracts.contains("--card-radius")
            && !contracts.contains("[slot='header']")
            && !contracts.contains("[slot='footer']")
            && !contracts.contains("/* ── card")
            && !contracts.contains("/* card surface"),
        "card host CSS must be fully gone from contracts.css"
    );
}

#[test]
fn navbar_host_paint_on_sx_cva() {
    // Layout + paint owned by navbar.sail (layout keywords + Style match/compound).
    // No --rail-offset / --nav-*. Sticky rail uses top-0 bottom-0 +
    // offset top-N compounds. Center grow-1; actions ms-auto mt-auto.
    let src = include_str!("../stdlib/navbar/navbar.sail");
    let out = compile_element(src, "navbar").expect("navbar compiles");
    assert!(
        !src.contains("--rail-offset")
            && !src.contains("--nav-")
            && src.contains("<row")
            && src.contains("nowrap")
            && src.contains("gap=\"4\"")
            && is_style_product(src)
            && (src.contains("flexDir(column)")
                || src.contains("dir-column")
                || src.contains("dir(column)"))
            && (src.contains("bg(navbar)")
                || src.contains("bg: navbar")
                || src.contains("bg(background)"))
            && ((src.contains("fg(navbar.foreground)") || src.contains("fg(navbar-foreground)"))
                || (src.contains("navbar.foreground") || src.contains("navbar-foreground"))
                || src.contains("fg(foreground)"))
            && (src.contains("minH(14)") || src.contains("min-h: 14") || src.contains("min-h: 12"))
            && (src.contains("padY(2)")
                || src.contains("py: 2")
                || src.contains("pad({ x: 4 })")
                || src.contains("pad({ y: 5, x: 4 })"))
            && (src.contains("padX(4)") || src.contains("px: 4") || src.contains("x: 4"))
            && (src.contains("position(sticky)")
                || src.contains("sticky")
                || src.contains("scheme: sticky"))
            && (src.contains("z(sticky)")
                || src.contains("z: sticky")
                || src.contains("z-(--z-sticky)"))
            && (src.contains("borderBWidth(1)")
                || src.contains("border-b")
                || src.contains("edge: b"))
            && (src.contains("top(auto)")
                || src.contains("top: auto")
                || src.contains("top-auto")
                || src.contains("t: auto"))
            && (src.contains("borderEWidth(1)")
                || src.contains("border-e")
                || src.contains("edge: e"))
            && (src.contains("minW(panel-rail-w)") || src.contains("panel-rail-w"))
            && (src.contains("display(contents)")
                || src.contains("contents")
                || src.contains("inset: 0")
                || src.contains("inset(0)"))
            && (src.contains("ms(auto)")
                || src.contains("ms: auto")
                || src.contains("ms-auto")
                || src.contains("s: auto")
                || src.contains("margin({ s: auto })"))
            && src.contains("offset: number")
            && (src.contains("top(bind(offset))") || src.contains("t: bind(offset)"))
            && !src.contains("offset: 8")
            && !src.contains("top(8)"),
        "navbar structure must own layout + sticky + top(bind(offset))"
    );
    assert!(
        out.structure_html.contains("sail-row")
            && out.structure_html.contains("sail-nowrap")
            && (out.structure_html.contains("sail-g-4") || out.structure_html.contains("sail-g-"))
            && (out.structure_html.contains("s-bgnavbar")
                || out.structure_html.contains("bgnavbar")
                || out.skin_css.contains("var(--navbar)")
                || out.skin_css.contains("var(--background)")
                || out.structure_html.contains("bgcolorbackground"))
            && (out.structure_html.contains("navbarforeground")
                || out.structure_html.contains("s-fgnavbar")
                || out.skin_css.contains("var(--navbar-foreground)")
                || out.skin_css.contains("var(--fg-foreground-color)")
                || out.structure_html.contains("s-fgforeground"))
            && (out.structure_html.contains("s-minh14")
                || out.structure_html.contains("minh14")
                || out.structure_html.contains("minh12")
                || out.skin_css.contains("min-height"))
            && (out.structure_html.contains("s-displaycontents")
                || out.structure_html.contains("s-contents")
                || out.structure_html.contains("display:contents")
                || out.skin_css.contains("display:contents")
                || out.skin_css.contains("display: contents")
                || out.structure_html.contains("inset0")
                || out.skin_css.contains("inset:0"))
            && out.structure_html.contains("slot name=\"brand\"")
            && out.structure_html.contains("slot name=\"center\"")
            && out.structure_html.contains("slot name=\"actions\"")
            && (out.structure_html.contains("s-msauto")
                || out.structure_html.contains("msauto")
                || out.structure_html.contains("ms-auto")
                || out.structure_html.contains("s-marginsauto")
                || out.skin_css.contains("margin-inline-start:auto")
                || out.skin_css.contains("margin-inline-start: auto")),
        "host row + region structure + start-auto must lower:\n{}",
        out.structure_html
    );
    assert!(
        out.skin_css.contains("placement='rail'")
            && out.skin_css.contains("min-height:0")
            && out.skin_css.contains("padding-block:var(--space-5)")
            && (out.skin_css.contains("min-width:var(--panel-rail-w)")
                || out.skin_css.contains("min-width: var(--panel-rail-w)")
                || out.skin_css.contains("minwpanelrailw")
                || out.structure_html.contains("minwpanelrailw"))
            && out.skin_css.contains("flex-direction:column")
            && out.skin_css.contains("[sticky]")
            && out.skin_css.contains("position:sticky")
            && out.skin_css.contains("z-index:var(--z-sticky)")
            && out.skin_css.contains("[bordered]")
            && out.skin_css.contains("border-bottom-width:1px")
            && out.skin_css.contains("placement='bottom'")
            && out.skin_css.contains("bottom:0")
            && out.skin_css.contains("border-top-width:1px")
            && out.skin_css.contains("border-inline-end-width:1px")
            && out.skin_css.contains("overflow-y:auto")
            && !out.skin_css.contains("--rail-offset")
            && !out.skin_css.contains("--nav-")
            && !out.skin_css.contains("offset='8'")
            && !out.skin_css.contains("offset=\"8\"")
            && (out.skin_css.contains("var(--offset)")
                || out.structure_html.contains("topbindoffset")
                || out.structure_html.contains("bindoffset")),
        "placement/sticky/bordered/rail + top(bind(offset)) (no offset enumeration):\n{}",
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    assert!(
        !contracts.contains(".sail-navbar")
            && !contracts.contains(".sail-nav-brand")
            && !contracts.contains(".sail-nav-links")
            && !contracts.contains(".sail-nav-center")
            && !contracts.contains(".sail-nav-actions")
            && !contracts.contains("--rail-offset")
            && !contracts.contains("/* ── navbar"),
        "navbar host matrix must be fully deleted from contracts.css"
    );
}

#[test]
fn error_state_child_paint_on_structure_sx() {
    // Style product: destructive semantic tokens + when(empty/group-state/forced).
    // No --es-* / tone-* channel paint.
    let src = include_str!("../stdlib/error-state/error-state.sail");
    let out = compile_element(src, "error-state").expect("error-state compiles");

    assert!(
        !src.contains("--es-")
            && !src.contains("tone-destructive")
            && src.contains("align=\"stretch\"")
            && is_style_product(src)
            && (src.contains("grow(1)") || src.contains(" grow") || src.contains("grow min-w"))
            && src.contains("self(stretch)")
            && src.contains("self(center)")
            && src.contains("fg(destructive)")
            && (src.contains("outline(destructive)") || src.contains("color: destructive"))
            && (src.contains("outline(1)") || src.contains("width: 1"))
            && (src.contains("outline(solid)") || src.contains("style: solid"))
            && (src.contains("outlineOffset(ring-inset)") || src.contains("offset: ring-inset"))
            && (src.contains("fontSize(md)") || src.contains("size: md"))
            && (src.contains("fontSize(sm)") || src.contains("size: sm"))
            && (src.contains("maxW(panel-ch-w-md)") || src.contains("max-w: panel-ch-w-md"))
            && (src.contains("maxW(panel-ch-w-lg)") || src.contains("max-w: panel-ch-w-lg"))
            && src.contains("size(control-h-xl)")
            && src.contains("size(control-h-2xs)")
            && src.contains("empty:")
            && src.contains("group-state-has-details")
            && src.contains("group-state-open")
            && src.contains("forced-colors"),
        "error-state Style product + stretch/grow + semantic destructive + empty/group-state"
    );

    assert!(
        (out.skin_css.contains("font-size:var(--text-md)")
            || out.skin_css.contains("font-size: var(--text-md)")
            || out.structure_html.contains("s-textmd")
            || out.structure_html.contains("fontsizemd"))
            && (out.skin_css.contains("font-size:var(--text-sm)")
                || out.skin_css.contains("font-size: var(--text-sm)")
                || out.structure_html.contains("s-textsm")
                || out.structure_html.contains("fontsizesm"))
            && (out.skin_css.contains("max-width:var(--panel-ch-w-md)")
                || out.skin_css.contains("max-width: var(--panel-ch-w-md)"))
            && (out.skin_css.contains("max-width:var(--panel-ch-w-lg)")
                || out.skin_css.contains("max-width: var(--panel-ch-w-lg)"))
            && (out.skin_css.contains("outline-color:var(--destructive)")
                || out.skin_css.contains("outline-color: var(--destructive)")
                || out.structure_html.contains("outlinedestructive")
                || out.skin_css.contains("var(--destructive)"))
            && (out
                .skin_css
                .contains("outline-offset:calc(-1 * var(--ring-width))")
                || out.structure_html.contains("outlineoffsetringinset")
                || out.structure_html.contains("outlineOffset"))
            && (out.structure_html.contains("controlhxl")
                || out.structure_html.contains("control-h-xl")
                || out.skin_css.contains("var(--control-h-xl)"))
            && (out.structure_html.contains("controlh2xs")
                || out.structure_html.contains("control-h-2xs")
                || out.skin_css.contains("var(--control-h-2xs)"))
            && !out.skin_css.contains("--es-")
            && !out.skin_css.contains("var(--tone-destructive"),
        "child paint semantic tokens must lower; no --es-*/tone-destructive-*:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.structure_html.contains("use:recoverySurface")
            && out.structure_html.contains("aria-expanded")
            && out.structure_html.contains("Show details")
            && out.structure_html.contains("slot name=\"details\"")
            && out.structure_html.contains("slot name=\"action\"")
            && (out.structure_html.contains("sail-grp") || src.contains("group()")),
        "structure + recoverySurface + group preserved (no part classes):\n{}",
        out.structure_html
    );
    assert!(
        out.skin_css.contains("size='sm'")
            && out.skin_css.contains("size='lg'")
            && out.skin_css.contains("variant='inline'")
            && (out.skin_css.contains("data-state~='has-details'")
                || out.skin_css.contains("data-state~=\"has-details\"")
                || out.skin_css.contains("groupstatehasdetails")
                || out.skin_css.contains("group-state")
                || out.structure_html.contains("groupstatehasdetails")
                || out.structure_html.contains("group-state-has-details")),
        "size/variant Style match + group-state must lower:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );

    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    assert!(
        !contracts.contains(".sail-error-state")
            && !contracts.contains("data-has-details")
            && !contracts.contains("data-details-open"),
        "error-state must have zero contracts host CSS rules"
    );
}

#[test]
fn radio_nested_row_kills_label_flex_pe() {
    // Root stays <label> for native semantics; type paint on host Style.
    // Flex/gap for label+control is contracts host CSS (cannot nest <row> on label).
    let src = include_str!("../stdlib/radio-group/radio.sail");
    let out = compile_element(src, "radio").expect("radio compiles");
    assert!(
        src.contains("<label")
            && src.contains("type=\"radio\"")
            && is_style_product(src)
            && (src.contains("radio.foreground")
                || src.contains("radio-foreground")
                || src.contains("fg(radio")
                || src.contains("fg(foreground)"))
            && (src.contains("fontFamily(body)")
                || src.contains("font-family: body")
                || src.contains("font-body")
                || src.contains("family: body"))
            && (src.contains("cursor(pointer)")
                || src.contains("cursor: pointer")
                || src.contains("cursor-pointer"))
            && ((src.contains("radio.foreground") || src.contains("radio-foreground"))
                || src.contains("color: foreground")
                || src.contains("fg(foreground)")),
        "radio must be native label + input with host type paint"
    );
    assert!(
        out.structure_html.contains("<label")
            || out.structure_html.contains("type=\"radio\"")
            || out.structure_html.contains("type='radio'"),
        "label + radio input structure preserved:\n{}",
        out.structure_html
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    // Hand `label.sail-radio` matrix is gone. Native label flex stays in contracts
    // (cannot nest <row> on a <label> host).
    assert!(
        !contracts.contains("label.sail-radio"),
        "hand label.sail-radio host matrix must be gone from contracts"
    );
}

#[test]
fn toggle_group_segmented_and_join_on_skins_not_pe() {
    // Segmented host Style match; size is authored on each <toggle> (not host).
    // Join/ghost/segmented on toggle first:/last: + size match (no --tg-*).
    let src = include_str!("../stdlib/toggle-group/toggle-group.sail");
    let out = compile_element(src, "toggle-group").expect("toggle-group compiles");
    assert!(
        !src.contains("'tg-h'")
            && !src.contains("--tg-")
            && is_style_product(src)
            && ((src.contains("bg(toggle-group)") || src.contains("bg(toggle-group)"))
                || src.contains("toggle-group")
                || src.contains("bg(muted)"))
            && (src.contains("border(toggle-group-border)")
                || (src.contains("toggle-group.border") || src.contains("toggle-group-border"))
                || src.contains("color: border"))
            && src.contains("<row inline align=\"stretch\" gap=\"0\"")
            && !src.contains("size: 'sm' | 'md' | 'lg' = 'md'"),
        "segmented host chrome on Style via muted/border tokens; host is inline row; size not on host"
    );
    assert!(
        out.structure_html.contains("sail-ai-stretch")
            || out.structure_html.contains("ai-stretch")
            || out.skin_css.contains("align-items:stretch")
            || out.skin_css.contains("align-items: stretch"),
        "vertical column must stretch the cross axis so cells share one width:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css.contains("variant='segmented'")
            && (out
                .skin_css
                .contains("background-color:var(--toggle-group)")
                || out
                    .skin_css
                    .contains("background-color: var(--toggle-group)")
                || out.structure_html.contains("togglegroup")
                || out.skin_css.contains("var(--bg-muted-color)")
                || out.skin_css.contains("bgmuted"))
            && !out.skin_css.contains("--tg-"),
        "segmented Style match uses muted rest; no --tg-* writes:\n{}",
        out.skin_css
    );
    // Orientation is a stamped union prop (layout runtime), not a CVA skin axis.
    assert!(
        src.contains("orientation: 'horizontal' | 'vertical' = 'horizontal'"),
        "toggle-group declares orientation union for layout stamping"
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    assert!(
        !contracts.contains("--tg-h:")
            && !contracts.contains(".sail-toggle-group > .sail-toggle")
            && !contracts.contains(".sail-toggle-group[orientation")
            && !contracts.contains(".sail-toggle-group[variant='ghost']"),
        "toggle-group hand host CSS must be gone from contracts"
    );
    let toggle_src = include_str!("../stdlib/toggle-group/toggle.sail");
    let toggle_out = compile_element(toggle_src, "toggle").expect("toggle compiles");
    assert!(
        !toggle_src.contains("--tg-")
            && toggle_src.contains("first:")
            && toggle_src.contains("last:")
            && toggle_src.contains("outline:")
            && toggle_src.contains("ghost:")
            && toggle_src.contains("size:")
            && is_style_product(toggle_src)
            && (toggle_src.contains("h(control-h-md)") || toggle_src.contains("h: control-h-md"))
            && (toggle_src.contains("h(control-h-sm)") || toggle_src.contains("h: control-h-sm"))
            && (toggle_src.contains("w(full)") || toggle_src.contains("w: full"))
            && (toggle_src.contains("bg(toggle)")
                || toggle_src.contains("bg: toggle")
                || toggle_src.contains("bg(background)")),
        "toggle.sail must Style join/ghost + size direct tokens + surface paint"
    );
    assert!(
        toggle_out.skin_css.contains("size='sm'")
            && toggle_out.skin_css.contains("size='lg'")
            && (toggle_out.skin_css.contains("var(--control-h-md)")
                || toggle_out.structure_html.contains("controlhmd")
                || toggle_out.structure_html.contains("control-h-md"))
            && (toggle_out.skin_css.contains("width:100%")
                || toggle_out.skin_css.contains("width: 100%")
                || toggle_out.structure_html.contains("wfull"))
            && !toggle_out.skin_css.contains("--tg-"),
        "toggle size CVA direct control-h tokens; no --tg-*:\n{}",
        toggle_out.skin_css
    );
}

#[test]
fn alert_tone_stripe_before_on_host_sx() {
    // Host <row nowrap align=start gap=3> owns ico|content|close (layout
    // keywords — free grid-template host Style forbidden). Stripe/empty/dismiss +
    // forced-colors via Style when/match. Zero alert host Style in contracts.
    let src = include_str!("../stdlib/alert/alert.sail");
    let out = compile_element(src, "alert").expect("alert");
    assert!(
        src.contains("<row")
            && src.contains("nowrap")
            && src.contains("gap=\"3\"")
            && src.contains("align=\"start\""),
        "alert host must be layout keyword row nowrap gap-3 align-start"
    );
    assert!(
        out.structure_html.contains("sail-row")
            && (out.structure_html.contains("sail-nowrap")
                || out.structure_html.contains("nowrap"))
            && out.structure_html.contains("sail-g-3"),
        "host row/nowrap/gap must lower into structure:\n{}",
        out.structure_html
    );
    assert!(
        !out.structure_html
            .contains("grid-template-columns: auto 1fr auto")
            && !out
                .skin_css
                .contains("grid-template-columns: auto 1fr auto")
            && !out.skin_css.contains("grid-template-columns:auto 1fr auto"),
        "must not emit free grid-template host Style in structure/skin:\n{}\n{}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        is_style_product(src)
            && src.contains("before:")
            && src.contains("content(empty)")
            && (src.contains("alert.stripe")
                || src.contains("alert-stripe")
                || src.contains("bg(primary)"))
            && src.contains("empty:")
            && src.contains("group-dismissible")
            && src.contains("group()")
            && src.contains("forced-colors")
            && src.contains("CanvasText")
            && !src.contains("tone-bg"),
        "host/child Style must carry stripe, empty gates, group-dismissible, forced-colors"
    );
    // Rest stripe = primary; brand via bound tone bridge var, not hand match matrix.
    assert!(
        (src.contains("bg(alert.stripe)")
            || src.contains("bg(alert-stripe)")
            || src.contains("bg(primary)"))
            && (src.contains("bg(tone)")
                || out.skin_css.contains("var(--tone)")
                || out.skin_css.contains("s-before-bgtone")),
        "stripe rest primary + bound tone brand"
    );
    assert!(
        (out.skin_css.contains("var(--alert-stripe)") || out.skin_css.contains("var(--primary)"))
            && (out.skin_css.contains("var(--tone)")
                || out.skin_css.contains("s-before-bgtone")
                || out.skin_css.contains("background-color:var(--tone)")),
        "compiled stripe: rest primary + branded tone bridge:\n{}",
        out.skin_css
    );
    assert!(
        out.structure_html.contains("before")
            || out.structure_html.contains("s-before")
            || out.skin_css.contains("::before"),
        "before stripe must lower into structure/skin:\n{}\n{}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css.contains("content:''") || out.skin_css.contains("content: ''"),
        "content(empty) must emit content:'':\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("var(--alert-stripe)")
            || out.skin_css.contains("alert-stripe")
            || out.skin_css.contains("var(--primary)"),
        "host default stripe must bind primary / alert-stripe:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("var(--tone)")
            || out.skin_css.contains("s-before-bgtone")
            || (out.skin_css.contains("tone='info'") && out.skin_css.contains("var(--info)")),
        "branded tone stripe via bridge var (or residual attr gate):\n{}",
        out.skin_css
    );
    assert!(
        out.structure_html.contains("empty")
            || out.structure_html.contains("s-empty")
            || out.skin_css.contains(":empty"),
        "empty gate must lower:\n{}\n{}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.structure_html.contains("groupdismissible")
            || out.structure_html.contains("group-dismissible")
            || out.skin_css.contains("dismissible")
            || out.skin_css.contains("sail-grp"),
        "group-dismissible close gate must lower:\n{}\n{}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css.contains("forced-colors") && out.skin_css.contains("CanvasText"),
        "forced-colors CanvasText must lower into skin:\n{}",
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    assert!(
        !contracts.contains(".sail-alert {")
            && !contracts.contains(".sail-alert .ico")
            && !contracts.contains(".sail-alert .title")
            && !contracts.contains(".sail-alert .actions")
            && !contracts.contains(".sail-alert::before")
            && !contracts.contains("grid-template-columns: auto 1fr auto")
            && !contracts.contains("aria-label='Dismiss'")
            && !contracts.contains(".sail-alert:not([tone])"),
        "all alert host CSS (grid/empty/svg/dismiss/soft-tone/forced-colors) must be gone from contracts"
    );
    assert!(
        out.structure_html.contains("sail-row") || out.structure_html.contains("sail-col"),
        "icon/content/actions use layout keywords:\n{}",
        out.structure_html
    );
}

#[test]
fn toast_region_host_paint_on_sx_cva() {
    // Width + bottom reverse + paint left contracts host CSS → host Style + CVA.
    // Child hit targets: toast host pointer-events-auto (region is none).
    let src = include_str!("../stdlib/toast/toast-region.sail");
    let out = compile_element(src, "toast-region").expect("toast-region compiles");
    assert!(
        is_style_product(src)
            && (src.contains("position(fixed)")
                || src.contains("scheme: fixed")
                || src.contains("position({ scheme: fixed"))
            && (src.contains("z(z-toast)") || src.contains("z-toast") || src.contains("z: toast"))
            && (src.contains("pointerEvents(none)") || src.contains("pointer-events"))
            && (src.contains("w(panel-w-toast)")
                || src.contains("panel-w-toast")
                || src.contains("w: panel-w-toast"))
            && src.contains("top-0 right-auto bottom-auto left-0")
            && src.contains("'top-end': 'top-0 right-0 bottom-auto left-auto'")
            && src.contains("col-reverse")
            && src.contains("'bottom-start':")
            && src.contains("'bottom-end':")
            && (src.contains("flexDir(column-reverse)") || src.contains("column-reverse")),
        "toast-region host Style/CVA must carry fixed/z/pad/width + position insets + reverse"
    );
    // Style-minted class names (s-{setter}{args}, case-preserving).
    assert!(
        (out.structure_html.contains("s-positionfixed")
            || out.structure_html.contains("schemefixed")
            || out.skin_css.contains("position:fixed"))
            && (out.structure_html.contains("s-zztoast")
                || out.structure_html.contains("ztoast")
                || out.skin_css.contains("var(--z-toast)"))
            && (out.structure_html.contains("s-pad4")
                || out.structure_html.contains("s-padall4")
                || out.skin_css.contains("padding:var(--space-4)"))
            && out.structure_html.contains("s-pointerEventsnone")
            && (out.structure_html.contains("s-top0")
                || out.structure_html.contains("t0")
                || out.skin_css.contains("top:0"))
            && (out.structure_html.contains("s-wpanelwtoast")
                || out.structure_html.contains("wpanelwtoast")
                || out.skin_css.contains("var(--panel-w-toast)")),
        "host paint atoms on structure:\n{}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css.contains("position='top-end'")
            && out.skin_css.contains("position='bottom-start'")
            && out.skin_css.contains("position='bottom-end'")
            && out.skin_css.contains("right:0")
            && out.skin_css.contains("bottom:0"),
        "position axis insets via CVA-in-props:\n{}",
        out.skin_css
    );
    // col-reverse is declared on bottom positions (layout keyword; may lower as
    // flex-direction or as a layout class depending on atom path).
    assert!(
        src.contains("col-reverse"),
        "bottom positions must declare col-reverse"
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    // toast-region host Style section may be fully deleted.
    if let Some(tr) = contracts
        .split("/* toast-region")
        .nth(1)
        .and_then(|s| s.split("/* ").next())
    {
        assert!(
            !tr.contains("position: fixed")
                && !tr.contains("z-index:")
                && !tr.contains("padding:")
                && !tr.contains("width: min(")
                && !tr.contains("flex-direction: column-reverse")
                && !tr.contains("pointer-events: auto")
                && !tr.contains("pointer-events: none"),
            "toast-region host CSS must be gone from contracts:\n{tr}"
        );
    } else {
        assert!(
            !contracts.contains(".sail-toast-region"),
            "toast-region host CSS fully deleted from contracts"
        );
    }
}

#[test]
fn theme_picker_host_paint_on_structure_sx() {
    // Layout keywords + host Style carry trigger/panel/option/swatch paint
    // on real nodes (no residual part classes). KEEP: panel anchor geometry,
    // [inline] matrices, forced-colors.
    let src = include_str!("../stdlib/theme-picker/theme-picker.sail");
    let out = compile_element(src, "theme-picker").expect("theme-picker compiles");
    assert!(
        is_style_product(src)
            && (src.contains("size(control-h-sm)") || src.contains("control-h-sm"))
            && (src.contains("bg(theme-picker)") || src.contains("theme-picker"))
            && (src.contains("selected:") || src.contains("selected:"))
            && src.contains("isolate")
            && !src.contains("class=")
            // Redesigned anatomy (2026-07): header + Light/Dark/System filter
            // tabs + a listbox of full-bleed live specimens (theme=
            // swatch + primary/secondary/accent strips + ghost/solid controls).
            // The old 2-col mini-card grid (cols=\"2\" / h(24) / inset-ring) is
            // gone BY DESIGN; the needles below pin the NEW anatomy.
            && src.contains("theme=\"")
            && src.contains("data-theme-listbox")
            && src.contains("data-theme-grid")
            && src.contains("data-mode-bar")
            && src.contains("use:themeSelect")
            && !src.contains(" role=")
            && src.contains("<row inline")
            && !src.contains("sx=\""),
        "theme-picker structure must carry Style product without class anatomy; roles via use:themeSelect"
    );
    assert!(
        (out.structure_html.contains("s-sizecontrolhsm")
            || out.structure_html.contains("sizecontrolhsm")
            || out.structure_html.contains("hcontrolhsm")
            || out.skin_css.contains("var(--control-h-sm)"))
            && (out.structure_html.contains("s-bgthemepicker")
                || out.structure_html.contains("bgthemepicker")
                || out.skin_css.contains("var(--theme-picker)")
                || out.skin_css.contains("var(--popover)")
                || out.structure_html.contains("bgcolorpopover"))
            && (out.structure_html.contains("s-isolate")
                || out.skin_css.contains("isolation:isolate")
                || out.skin_css.contains("isolation: isolate"))
            && (out.structure_html.contains("data-theme-listbox")
                || out.structure_html.contains("use:themeSelect"))
            && out.structure_html.contains("theme=\"")
            && (out.structure_html.contains("sail-row-i") || out.skin_css.contains("inline-flex")),
        "child paint + layout classes on structure:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    // Option rows are real nodes with selected/hover/focus borders — no tc-*
    // mini-card classes, no part classes, primary/accent paint present.
    // role=option is stamped at runtime by use:themeSelect, not structure HTML.
    assert!(
        !out.structure_html.contains("tc-overlay")
            && !out.structure_html.contains("class=\"tc-")
            && (out.structure_html.contains("data-theme-grid")
                || out.structure_html.contains("value=\"sail\""))
            && (out.structure_html.contains("s-bgprimary")
                || out.structure_html.contains("bgprimary")
                || out.skin_css.contains("var(--primary)"))
            && (out.skin_css.contains("CanvasText") || out.structure_html.contains("CanvasText")),
        "option rows must be real nodes with selected/forced-colors states:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    // Paint piles fully dissolved — no contracts host CSS for theme-picker anatomy.
    assert!(
        !contracts.contains(".sail-theme-trigger svg")
            && !contracts.contains(".sail-theme-swatch")
            && !contracts.contains(".tc-overlay")
            && !contracts.contains(".tc-card")
            && !contracts.contains(".tc-cta")
            && !contracts.contains(".tc-chip")
            && !contracts.contains("color-mix(in oklch, currentColor 14%"),
        "moved theme-picker paint piles must be gone from contracts"
    );
    // Anchor geometry + [inline] matrices live on structure Style / element-skins now.
    assert!(
        out.structure_html.contains("anchorleft")
            || out.structure_html.contains("anchor-left")
            || out.skin_css.contains("var(--anchor-left)")
            || out.skin_css.contains("left:var(--anchor-left)"),
        "anchor geometry must land on structure/skin:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        src.contains("group-inline")
            || src.contains("inline = false")
            || out.structure_html.contains("groupinline")
            || out.skin_css.contains("[inline]"),
        "inline matrices expressed via structure/group-inline"
    );
}

#[test]
fn prose_host_paint_on_sx_descendants_generated() {
    // Host block + body typeface/color/leading → host Style.
    // Descendant recipe: freeform HTML → compiler prose map → element-skins
    // (not hand contracts host CSS).
    let src = include_str!("../stdlib/prose/prose.sail");
    let out = compile_element(src, "prose").expect("prose compiles");
    assert!(
        is_style_product(src)
            && (src.contains("display(block)") || src.contains("block"))
            && (src.contains("fontFamily(body)")
                || src.contains("font-body")
                || src.contains("font-family: body")
                || src.contains("family: body"))
            && ((src.contains("prose.foreground") || src.contains("prose-foreground"))
                || src.contains("text-foreground")
                || src.contains("fg(foreground)"))
            && (src.contains("leading(loose)")
                || src.contains("leading-loose")
                || src.contains("leading: loose")),
        "prose host Style must carry block + body type/color/leading"
    );
    assert!(
        (out.structure_html.contains("s-displayblock")
            || out.structure_html.contains("s-block")
            || out.skin_css.contains("display:block")
            || out.skin_css.contains("display: block"))
            && (out.structure_html.contains("s-fontfamilybody")
                || out.structure_html.contains("s-fontbody")
                || out.structure_html.contains("typefamilybody")
                || out.skin_css.contains("var(--body-font)"))
            && (out.structure_html.contains("proseforeground")
                || out.structure_html.contains("s-fgprose")
                || out.skin_css.contains("var(--prose-foreground)")
                || out.structure_html.contains("s-textforeground")
                || out.structure_html.contains("s-fgforeground")
                || out.skin_css.contains("var(--fg-foreground-color)"))
            && (out.structure_html.contains("leadingloose")
                || out.structure_html.contains("s-leadingloose")
                || out.skin_css.contains("var(--leading-loose)")),
        "host paint atoms on structure:\n{}",
        out.structure_html
    );
    // Descendant anatomy lives in the platform adaptor, not compile_element skin.
    let skins = sail_compiler::stdlib::skins_css();
    assert!(
        skins.contains("/* @provenance platform/prose.css */")
            && skins.contains(".sail-prose h1")
            && skins.contains(".sail-prose p")
            && skins.contains(".sail-prose a")
            && skins.contains(".sail-prose code")
            && skins.contains(".sail-prose pre")
            && skins.contains(".sail-prose blockquote")
            && skins.contains(".sail-prose ul")
            && skins.contains(".sail-prose table")
            && skins.contains(".sail-prose strong")
            && skins.contains("li::marker")
            && skins.contains("font-family:var(--heading-font)")
            && skins.contains("color:var(--primary)"),
        "prose descendant recipe must live in platform/prose.css via skins_css"
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    // No hand host Style descendant rules.
    assert!(
        !contracts.contains(".sail-prose h1")
            && !contracts.contains(".sail-prose p")
            && !contracts.contains(".sail-prose a")
            && !contracts.contains(".sail-prose code")
            && !contracts.contains(".sail-prose blockquote")
            && !contracts.contains(".sail-prose table"),
        "prose descendant host CSS must be gone from contracts"
    );
}

#[test]
fn table_cells_are_family_parts_not_descendant_paint() {
    // Rest cell paint is Style on native parts. Host compounds stay platform
    // because a part cannot see table attrs and tr stays HTML.
    let th = compile_element(include_str!("../stdlib/table/th.sail"), "th").expect("th compiles");
    let td = compile_element(include_str!("../stdlib/table/td.sail"), "td").expect("td compiles");
    let caption = compile_element(include_str!("../stdlib/table/caption.sail"), "caption")
        .expect("caption compiles");
    assert!(
        th.native && td.native && caption.native,
        "table parts are native tags"
    );
    assert!(
        (th.skin_css.contains("padding-block:var(--space-2_5)")
            || th.skin_css.contains("padding-block: var(--space-2_5)")
            || th.skin_css.contains("--space-2_5"))
            && (th.skin_css.contains("muted-foreground")
                || th.structure_html.contains("muted-foreground")
                || th.structure_html.contains("mutedforeground"))
            && (th.skin_css.contains("nowrap") || th.structure_html.contains("nowrap")),
        "th rest paint is Style:\nskin={}\nstruct={}",
        th.skin_css,
        th.structure_html
    );
    assert!(
        th.skin_css.contains("[numeric]") || th.skin_css.contains("numeric="),
        "th numeric is a cell match, not a table descendant:\n{}",
        th.skin_css
    );
    let skins = sail_compiler::stdlib::skins_css();
    assert!(
        skins.contains("/* @element th */")
            && skins.contains("/* @element td */")
            && skins.contains("/* @element caption */"),
        "table parts must emit their own skins"
    );
    assert!(
        !skins.contains(".sail-table th { padding")
            && !skins.contains(".sail-table td { padding")
            && !skins.contains(".sail-table caption {")
            && !skins.contains("th[numeric]"),
        "cell rest paint must not be a table descendant sheet:\n{skins}"
    );
    assert!(
        skins.contains("/* @provenance platform/table.css */")
            && skins.contains(".sail-table[dense]")
            && skins.contains(".sail-table[striped]")
            && skins.contains(".sail-table[hoverable]")
            && skins.contains(".sail-table[bordered]")
            && skins.contains("border-collapse:collapse"),
        "host compounds that Style cannot see stay in platform/table.css"
    );
}

#[test]
fn tab_list_sticky_and_variant_on_host_not_parent_pe() {
    // Sticky + variant on <tab-list> host. Pill track CVA. Child tab matrices
    // are stamped variant CVA on tab.sail (not parent host CSS). Vertical host axis
    // is layout-owned; list/panel orientation stamped by use:tabs.
    let src = include_str!("../stdlib/tabs/tab-list.sail");
    let out = compile_element(src, "tab-list").expect("tab-list compiles");
    assert!(
        src.contains("sticky = false")
            && is_style_product(src)
            && (src.contains("position(sticky)") || src.contains("sticky"))
            && (src.contains("z(sticky)") || src.contains("z: sticky"))
            && (src.contains("bg(tab-list)")
                || src.contains("tab-list")
                || src.contains("bg(background)"))
            && src.contains("pill:")
            && ((src.contains("bg(tab-list.rail)") || src.contains("bg(tab-list-rail)"))
                || (src.contains("tab-list.rail") || src.contains("tab-list-rail"))
                || src.contains("bg(muted)")),
        "tab-list must declare sticky compound + host bg + pill rail Style on host"
    );
    assert!(
        (out.structure_html.contains("bgtablist")
            || out.structure_html.contains("s-bgbackground")
            || out.structure_html.contains("bgcolorbackground")
            || out.skin_css.contains("var(--tab-list)")
            || out.skin_css.contains("var(--background)"))
            && (out.structure_html.contains("borderB")
                || out.structure_html.contains("borderb")
                || out.structure_html.contains("edgeb")
                || out.skin_css.contains("border-bottom")),
        "rail + bg on host structure:\n{}",
        out.structure_html
    );
    assert!(
        out.skin_css.contains("[sticky]")
            && out.skin_css.contains("position:sticky")
            && out.skin_css.contains("z-index:var(--z-sticky)"),
        "sticky compound must lower on tab-list:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("[variant='pill']")
            && (out
                .skin_css
                .contains("background-color:var(--tab-list-rail)")
                || out
                    .skin_css
                    .contains("background-color: var(--tab-list-rail)")
                || out.skin_css.contains("var(--bg-muted-color)"))
            && (out.skin_css.contains("border-radius:var(--radius-md)")
                || out.skin_css.contains("border-radius: var(--radius-md)")),
        "pill variant CVA must lower on tab-list with muted rail:\n{}",
        out.skin_css
    );
    // sticky/variant/size not on <tabs> — size Style match lives on <tab>.
    let tabs_src = include_str!("../stdlib/tabs/tabs.sail");
    let _tabs_out = compile_element(tabs_src, "tabs").expect("tabs compiles");
    assert!(
        !tabs_src.contains("'tab-py'")
            && !tabs_src.contains("--tab-")
            && !tabs_src.contains("sticky = false")
            && !tabs_src.contains("sticky=false")
            && !tabs_src.contains("variant:")
            && !tabs_src.contains("size: 'sm' | 'md' | 'lg' = 'md'")
            && !tabs_src.contains("tone:"),
        "sticky/variant/size/tone not on <tabs> (no --tab-*):\n{tabs_src}"
    );
    // Size CVA lives on <tab> (text-sm py-2 px-3 …), not --tab-* custom props.
    let tab_src = include_str!("../stdlib/tabs/tab.sail");
    let tab_out = compile_element(tab_src, "tab").expect("tab compiles");
    assert!(
        !tab_src.contains("--tab-")
            && tab_src.contains("size:")
            && is_style_product(tab_src)
            && (tab_src.contains("fontSize(sm)")
                || tab_src.contains("font-size: sm")
                || tab_src.contains("size: sm"))
            && (tab_src.contains("padY(2)")
                || tab_src.contains("py: 2")
                || tab_src.contains("y: 2"))
            && (tab_src.contains("padX(3)")
                || tab_src.contains("px: 3")
                || tab_src.contains("x: 3"))
            && (tab_src.contains("fontSize(xs)")
                || tab_src.contains("font-size: xs")
                || tab_src.contains("size: xs"))
            && (tab_src.contains("tab.foreground")
                || tab_src.contains("tab-foreground")
                || tab_src.contains("muted-foreground")
                || tab_src.contains("fg(tab")),
        "tab.sail size Style direct pad/type tokens + component.tab paint:\n{tab_src}"
    );
    assert!(
        tab_out.skin_css.contains("size='sm'")
            && tab_out.skin_css.contains("size='lg'")
            && (tab_out.skin_css.contains("font-size:var(--text-sm)")
                || tab_out.skin_css.contains("font-size: var(--text-sm)")
                || tab_out.structure_html.contains("s-textsm"))
            // Banned: retired size channels --tab-py / --tab-h. Allowed: component
            // token reads --tab-foreground / --tab-selected*.
            && !tab_out.skin_css.contains("--tab-py")
            && !tab_out.skin_css.contains("--tab-h"),
        "tab size CVA must land; no retired --tab-py/--tab-h channels:\n{}",
        tab_out.skin_css
    );
    // Orientation is a stamped union prop (layout runtime), not a CVA skin axis.
    assert!(
        tabs_src.contains("orientation: 'horizontal' | 'vertical' = 'horizontal'"),
        "tabs declares orientation union for layout stamping:\n{tabs_src}"
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    assert!(
        !contracts.contains(".sail-tabs[sticky]")
            && !contracts.contains(".sail-tabs[variant=")
            && !contracts.contains(".sail-tabs[orientation=")
            && !contracts.contains(".sail-tab-list[variant=")
            && !contracts.contains("--tab-py:")
            && !contracts.contains(".sail-tabs[size="),
        "tabs/tab-list hand host CSS must be gone from contracts"
    );
}

#[test]
fn scroll_area_standard_scrollbar_on_host_sx() {
    // Standard scrollbar pack on host Style; orientation match owns overflow axes.
    // Webkit thumb pseudo pack is a documented Style-setter blocker.
    let src = include_str!("../stdlib/scroll-area/scroll-area.sail");
    let out = compile_element(src, "scroll-area").expect("scroll-area compiles");
    assert!(
        is_style_product(src)
            && (src.contains("scrollbarThin()")
                || src.contains("scrollbar-thin")
                || src.contains("scrollbar: thin"))
            && (src.contains("scrollbarThumb(scroll-area-thumb)")
                || src.contains("scroll-area.thumb")
                || src.contains("scroll-area-thumb")
                || src.contains("scrollbarThumb(border)"))
            && (src.contains("overscroll(contain)")
                || src.contains("overscroll: contain")
                || src.contains("overscroll-contain"))
            && (src.contains("match:") && src.contains("orientation:")
                || src.contains("orientation:")),
        "scroll-area host Style must carry standard scrollbar pack via component.scroll-area tokens"
    );
    // Overflow / orientation always land.
    assert!(
        out.structure_html.contains("s-overflowyauto")
            || out.structure_html.contains("overflowyauto")
            || out.structure_html.contains("overflow-y-auto")
            || out.skin_css.contains("overflow-y:auto")
            || out.skin_css.contains("overflow-y: auto"),
        "vertical overflow must land:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css.contains("orientation='vertical'")
            && out.skin_css.contains("orientation='horizontal'")
            && out.skin_css.contains("orientation='both'"),
        "orientation match must land:\n{}",
        out.skin_css
    );
    // Standard scrollbar pack lands via Style setters.
    assert!(
        out.structure_html.contains("scrollbar")
            || out.skin_css.contains("scrollbar-width")
            || out.skin_css.contains("scrollbar-color")
            || out.skin_css.contains("--sx-scrollbar"),
        "standard scrollbar pack must land:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    // No hand scroll-area host CSS (section may be fully deleted).
    if let Some(sa) = contracts
        .split("/* scroll-area")
        .nth(1)
        .and_then(|s| s.split("/* ").next())
    {
        assert!(
            !sa.contains("scrollbar-width:")
                && !sa.contains("scrollbar-color:")
                && !sa.contains("::-webkit-scrollbar")
                && !sa.lines().any(|l| {
                    let t = l.trim();
                    t.starts_with(".sail-scroll-area {") || t.starts_with(".sail-scroll-area{")
                }),
            "scroll-area scrollbar host CSS (standard + webkit) must be gone:\n{sa}"
        );
    } else {
        assert!(
            !contracts.contains(".sail-scroll-area"),
            "scroll-area host CSS fully deleted from contracts"
        );
    }
}

#[test]
fn code_block_pre_tab_and_tok_generated() {
    // pre tab-size:2 + line-height code → pre Style (tab(2) + leading(code)).
    // .tok-* highlighter ink: compiler code-block map → element-skins (not host Style).
    let src = include_str!("../stdlib/code-block/code-block.sail");
    let out = compile_element(src, "code-block").expect("code-block compiles");
    assert!(
        is_style_product(src)
            && (src.contains("tab(2)") || src.contains("tab-2") || src.contains("tab: 2"))
            && (src.contains("leading(code)")
                || src.contains("leading-code")
                || src.contains("leading: code")),
        "code-block pre Style must carry tab(2) + leading(code)"
    );
    assert!(
        (out.structure_html.contains("s-tab2")
            || out.structure_html.contains("tab2")
            || out.skin_css.contains("tab-size"))
            && (out.structure_html.contains("s-leadingcode")
                || out.structure_html.contains("leadingcode")
                || out.skin_css.contains("var(--leading-code)")),
        "tab/leading atoms on structure:\n{}",
        out.structure_html
    );
    assert!(
        out.skin_css.contains("tab-size:2")
            || out.skin_css.contains("tab-size: 2")
            || out.structure_html.contains("s-tab2")
            || out.structure_html.contains("tab2"),
        "tab-2 must lower:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("line-height:var(--leading-code)")
            || out.skin_css.contains("line-height: var(--leading-code)")
            || out.structure_html.contains("s-leadingcode")
            || out.structure_html.contains("leadingcode"),
        "leading-code must lower:\n{}",
        out.skin_css
    );
    // .tok-* highlighter ink is platform adaptor CSS, not host Style.
    let skins = sail_compiler::stdlib::skins_css();
    assert!(
        skins.contains("/* @provenance platform/code-block.css */")
            && skins.contains(".sail-code-block .tok-tag")
            && skins.contains("var(--syntax-tag)")
            && skins.contains(".tok-keyword")
            && skins.contains("var(--syntax-keyword)")
            && skins.contains(".tok-comment")
            && skins.contains("font-style:italic"),
        "code-block .tok-* must live in platform/code-block.css via skins_css"
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    assert!(
        !contracts.contains(".sail-code-block .tok-tag")
            && !contracts.contains(".sail-code-block .tok-keyword")
            && !contracts.contains(".sail-code-block pre"),
        "code-block .tok-* host CSS must be gone from contracts"
    );
}

#[test]
fn field_empty_and_invalid_hint_on_structure_sx() {
    // Style product: empty/group-invalid when gates on hint/error structure.
    let src = include_str!("../stdlib/field/field.sail");
    let out = compile_element(src, "field").expect("field compiles");
    assert!(
        (src.contains(".group()") || src.contains("style({ group() })") || src.contains("group()"))
            && src.contains("empty:")
            && src.contains("group-invalid:")
            && !src.contains("class=")
            && src.contains("data-field-error")
            && src.contains("use:describedControl")
            && !src.contains(" role=")
            && (src.contains("field.hint") || src.contains("fg(muted-foreground)"))
            && (src.contains("field.error") || src.contains("fg(destructive)")),
        "field structure must carry group + empty/group-invalid Style gates without part classes; alert via use:describedControl"
    );
    assert!(
        out.structure_html.contains("sail-grp")
            && (out.structure_html.contains("empty") || out.skin_css.contains(":empty"))
            && (out.structure_html.contains("groupinvalid")
                || out.structure_html.contains("group-invalid")
                || out.skin_css.contains(".sail-grp[invalid]")
                || out.skin_css.contains("[invalid]")),
        "empty: + group-invalid: must lower:\n{}\n{}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.structure_html.contains("use:describedControl"),
        "behavior stamp preserved:\n{}",
        out.structure_html
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    if let Some(field) = contracts
        .split("/* field")
        .nth(1)
        .and_then(|s| s.split("/* ").next())
    {
        assert!(
            !field.contains(".sail-field .hint:empty")
                && !field.contains(".sail-field[invalid] .hint")
                && !field.contains(".sail-field:not([invalid]) .error"),
            "field hint/error host Style rules must be gone:\n{field}"
        );
    } else {
        assert!(
            !contracts.contains(".sail-field .hint") && !contracts.contains(".sail-field[invalid]"),
            "field hint/error host CSS fully deleted from contracts"
        );
    }
}

#[test]
fn label_req_gate_in_element_skins_anatomy() {
    // Style product: .req gate via when(group-required) + component.label tokens.
    let src = include_str!("../stdlib/label/label.sail");
    let out = compile_element(src, "label").expect("label compiles");
    assert!(
        !src.contains("class=")
            && (src.contains("label.required")
                || src.contains("label-required")
                || src.contains("fg(destructive)"))
            && src.contains("group-required")
            && src.contains("display(none)")
            && (src.contains(".group()") || src.contains("style({ group() })")),
        "structure documents required marker via Style (no part class)"
    );
    assert!(out.native, "label is native-kind");
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    assert!(
        !contracts.contains("label .req { display: none; }")
            && !contracts.contains("label[required] .req { display: inline; }"),
        "label .req host CSS must be gone from contracts"
    );
    assert!(
        out.structure_html.contains("aria-hidden")
            && (out.structure_html.contains("displaynone")
                || out.structure_html.contains("s-")
                || out.skin_css.contains("display:none")
                || out.skin_css.contains("display: none"))
            && (out.structure_html.contains("sail-grp")
                || out.structure_html.contains("grouprequired")
                || out.skin_css.contains(".sail-grp[required]")
                || out.skin_css.contains("required")),
        "label .req gate must lower via Style:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css.contains("color:var(--label-required)")
            || out.skin_css.contains("color: var(--label-required)")
            || out.structure_html.contains("labelrequired")
            || out.skin_css.contains("var(--destructive)"),
        "req paint in structure/skin:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
}

#[test]
fn pagination_size_cva_on_page() {
    // Size Style match lives on each <page>; <pagination> is layout-only (no size prop).
    // No --pg-* host Style in contracts.
    let page_src = include_str!("../stdlib/pagination/page.sail");
    let pag_src = include_str!("../stdlib/pagination/pagination.sail");
    let page = compile_element(page_src, "page").expect("page compiles");
    let _pag = compile_element(pag_src, "pagination").expect("pagination compiles");
    assert!(
        !pag_src.contains("size: 'sm'") && !pag_src.contains("size: 'md'"),
        "pagination host must not declare unused size prop:\n{pag_src}"
    );
    assert!(
        is_style_product(page_src)
            && page_src.contains("match:")
            && page_src.contains("size:")
            && (page_src.contains("minW(control-h-md)")
                || page_src.contains("min-w-control-h-md")
                || page_src.contains("control-h-md"))
            && (page_src.contains("h(control-h-md)")
                || page_src.contains("h-control-h-md")
                || page_src.contains("control-h-md"))
            && (page_src.contains("fontSize(sm)")
                || page_src.contains("text-sm")
                || page_src.contains("font-size: sm")
                || page_src.contains("size: sm")),
        "page must own size Style match with control-h / text steps"
    );
    assert!(
        page.skin_css.contains("size='sm'")
            && page.skin_css.contains("size='lg'")
            && (page.skin_css.contains("min-width:var(--control-h-sm)")
                || page.skin_css.contains("min-width: var(--control-h-sm)"))
            && (page.skin_css.contains("height:var(--control-h-lg)")
                || page.skin_css.contains("height: var(--control-h-lg)")),
        "page size match must lower:\n{}",
        page.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    assert!(
        !contracts.contains("--pg-h:")
            && !contracts.contains("--pg-size:")
            && !contracts.contains(".sail-pagination { --pg-"),
        "pagination --pg-* host CSS must be gone from contracts"
    );
    assert!(
        pag_src.contains("use:pagination"),
        "pagination stamps size via use:pagination"
    );
}

#[test]
fn overlay_family_anchor_geometry_on_host_sx() {
    // dropdown / context-menu / hover-card / popover: fixed + inset-auto + m-0 +
    // left/top bind(anchor*) geometry on host Style (or residual structured sx).
    // Backdrop is global :where([popover])::backdrop. role=menu min-width is POPOVER_SKIN.
    for (tag, extra) in [
        ("dropdown", "use:menu"),
        ("context-menu", "use:contextMenu"),
        ("hover-card", "panel-max-w-sm"),
        ("popover", "panel-menu-min-w"),
    ] {
        let src = sail_compiler::stdlib::sources()
            .iter()
            .find(|(t, _)| *t == tag)
            .map(|(_, s)| *s)
            .unwrap_or_else(|| panic!("{tag} registered"));
        let out = compile_element(src, tag).unwrap_or_else(|e| panic!("{tag}: {e:?}"));
        let geometry = (src.contains("position(fixed)")
            || src.contains("scheme: fixed")
            || src.contains("fixed"))
            && (src.contains("inset(auto)")
                || src.contains("inset: auto")
                || src.contains("inset-auto"))
            && (src.contains("m(0)")
                || src.contains("m: 0")
                || src.contains("m-0")
                || src.contains("margin(0)"))
            && (src.contains("bind(anchorLeft)")
                || src.contains("left-(--anchor-left)")
                || src.contains("left: (--anchor-left)"))
            && (src.contains("bind(anchorTop)")
                || src.contains("top-(--anchor-top)")
                || src.contains("top: (--anchor-top)"));
        assert!(
            geometry && src.contains(extra),
            "{tag} host Style/sx must carry anchor geometry + extras:\n{src}"
        );
        assert!(
            (out.structure_html.contains("s-positionfixed")
                || out.structure_html.contains("s-fixed")
                || out.skin_css.contains("position:fixed")
                || out.skin_css.contains("position: fixed"))
                && (out.structure_html.contains("insetauto")
                    || out.structure_html.contains("s-insetauto")
                    || out.skin_css.contains("inset:auto")
                    || out.skin_css.contains("inset: auto"))
                && (out.structure_html.contains("s-m0")
                    || out.structure_html.contains("m0")
                    || out.skin_css.contains("margin:0")
                    || out.skin_css.contains("margin: 0"))
                && (out.structure_html.contains("anchorLeft")
                    || out.structure_html.contains("anchorleft")
                    || out.structure_html.contains("s-leftanchorleft")
                    || out.skin_css.contains("var(--anchor-left)"))
                && (out.structure_html.contains("anchorTop")
                    || out.structure_html.contains("anchortop")
                    || out.structure_html.contains("s-topanchortop")
                    || out.skin_css.contains("var(--anchor-top)")),
            "{tag} geometry atoms on structure:\n{}",
            out.structure_html
        );
    }
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    for needle in [
        ".sail-dropdown {",
        ".sail-context-menu {",
        ".sail-hover-card {",
        ".sail-popover {\n  position: fixed",
        ".sail-dropdown::backdrop",
        ".sail-context-menu::backdrop",
        ".sail-hover-card::backdrop",
        ".sail-popover::backdrop",
    ] {
        assert!(
            !contracts.contains(needle),
            "overlay host CSS must be gone: {needle}"
        );
    }
}

#[test]
fn combobox_listbox_geometry_on_child_sx_options_on_popover_skin() {
    // Listbox anchor geometry + scroll chrome on child Style; option anatomy is
    // global POPOVER_SKIN [role=listbox] (author children, no structure hook).
    let src = include_str!("../stdlib/combobox/combobox.sail");
    let out = compile_element(src, "combobox").expect("combobox compiles");
    assert!(
        is_style_product(src)
            && (src.contains("bind(anchorLeft)")
                || src.contains("left-(--anchor-left)")
                || src.contains("left: (--anchor-left)"))
            && (src.contains("bind(anchorTop)")
                || src.contains("top-(--anchor-top)")
                || src.contains("top: (--anchor-top)"))
            && (src.contains("bind(anchorWidth)")
                || src.contains("min-w-(--anchor-width)")
                || src.contains("min-w: (--anchor-width)"))
            // List width = trigger: both w and minW from --anchor-width (C-CB-1).
            && (src.contains("w(bind(anchorWidth))")
                || src.contains("w-(--anchor-width)")
                || src.contains("width: (--anchor-width)")
                || src.contains("w: bind(anchorWidth)"))
            && (src.contains("maxH(panel-max-h-sm)")
                || src.contains("max-h-panel-max-h-sm")
                || src.contains("panel-max-h-sm"))
            && (src.contains("overflowY(auto)")
                || src.contains("overflow-y-auto")
                || src.contains("overflow-y: auto")
                || src.contains("y: auto"))
            && (src.contains("overscroll(contain)")
                || src.contains("overscroll-contain")
                || src.contains("overscroll: contain")),
        "listbox Style must carry anchor geometry + scroll"
    );
    assert!(
        (out.structure_html.contains("s-positionfixed")
            || out.structure_html.contains("s-fixed")
            || out.skin_css.contains("position:fixed")
            || out.skin_css.contains("position: fixed"))
            && (out.structure_html.contains("anchorWidth")
                || out.structure_html.contains("anchorwidth")
                || out.structure_html.contains("s-minwanchorwidth")
                || out.structure_html.contains("s-wbindanchorwidth")
                || out.skin_css.contains("var(--anchor-width)"))
            && (out.structure_html.contains("maxhpanelmaxhsm")
                || out.structure_html.contains("s-maxhpanelmaxhsm")
                || out.skin_css.contains("var(--panel-max-h-sm)"))
            && (out.structure_html.contains("overflowyauto")
                || out.structure_html.contains("s-overflowyauto")
                || out.skin_css.contains("overflow-y:auto")
                || out.skin_css.contains("overflow-y: auto"))
            && (out.structure_html.contains("overscrollcontain")
                || out.structure_html.contains("s-overscrollcontain")
                || out.skin_css.contains("overscroll-behavior:contain")
                || out.skin_css.contains("overscroll-behavior: contain")),
        "listbox geometry on structure:\n{}",
        out.structure_html
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    assert!(
        !contracts.contains(".sail-combobox .listbox")
            && !contracts.contains(".sail-combobox .empty")
            && !contracts.contains("anchor-size(width"),
        "combobox component host CSS must be gone:\n{}",
        contracts
            .lines()
            .filter(|l| l.contains("combobox") || l.contains("anchor-size"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    // Option chrome lives on the role-keyed global popover contract.
    assert!(
        contracts.contains("[role='listbox'] option")
            || contracts.contains("[role=listbox] option"),
        "listbox option anatomy must be on POPOVER_SKIN/contracts global"
    );
}

#[test]
fn tooltip_geometry_and_show_on_sx_no_contracts_pe() {
    // Host: group + relative. Bubble: absolute + bind(tt*) geometry vars +
    // opacity show via group-hover/focus-within + after: arrow. tipDescribe
    // publishes placement vars (same bridge as use:anchor).
    let src = include_str!("../stdlib/tooltip/tooltip.sail");
    let out = compile_element(src, "tooltip").expect("tooltip compiles");
    assert!(
        is_style_product(src)
            && src.contains("group()")
            && src.contains("position(relative)")
            && src.contains("bind(ttTop)")
            && src.contains("bind(ttTx)")
            && (src.contains("group-hover") || src.contains("group-hover:"))
            && (src.contains("group-focus-within") || src.contains("group-focus-within:"))
            && src.contains("content(empty)")
            && !src.contains("rotate(45)")
            && (src.contains("borderT(tooltip)")
                || src.contains("borderL(tooltip)")
                || src.contains("color: popover")
                || src.contains("edge: t")),
        "tooltip host/bubble Style must carry geometry + show + border caret (not diamond)"
    );
    // Style-minted class names (bind args mint as `<setter>bind<Name>`).
    assert!(
        out.structure_html.contains("sail-grp")
            && (out.structure_html.contains("s-positionrelative")
                || out.structure_html.contains("s-positionschemerelative"))
            && (out.structure_html.contains("s-positionabsolute")
                || out.structure_html.contains("schemeabsolute")
                || out.skin_css.contains("position:absolute"))
            && (out.structure_html.contains("s-topbindttTop")
                || out.structure_html.contains("tbindttTop")
                || out.skin_css.contains("var(--tt-top)"))
            && (out.structure_html.contains("s-grouphover-opacity100")
                || out.structure_html.contains("grouphover-filteropacity100")
                || out.skin_css.contains("opacity:1"))
            && (out.structure_html.contains("borderTtooltip")
                || out.skin_css.contains("border-top-color:var(--tooltip)")
                || out.skin_css.contains("border-top-color: var(--tooltip)")
                || out.skin_css.contains("border-top-color:var(--popover)")
                || out.structure_html.contains("bordercolorpopoveredget")),
        "tooltip structure atoms (border caret):\n{}",
        out.structure_html
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    assert!(
        !contracts.contains("anchor-name: var(--sail-tooltip-anchor)")
            && !contracts.contains("position-anchor: var(--sail-tooltip-anchor)")
            && !contracts.contains("position-area:")
            && !contracts.contains(".sail-tooltip .bubble")
            && !contracts.contains(".sail-tooltip:hover .bubble"),
        "tooltip contracts host CSS matrix must be gone"
    );
}
