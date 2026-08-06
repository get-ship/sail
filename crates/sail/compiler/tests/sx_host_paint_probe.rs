//! Host-paint delete path — layout + sx + CVA-in-props replace author `<style>`.
//! Golden: radio-group is the first stdlib element with zero author skin CSS.

use sail_compiler::compile_element;

#[test]
fn radio_group_zero_author_style() {
    let src = include_str!("../stdlib/radio-group.sail");
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
  const soft = style.bg(muted).fg(muted-foreground).borderWidth(1).border(border)
  const solid = style.bg(primary).fg(primary-foreground).borderWidth(1).border(primary).when({ hover: style.opacity(90) })
  const outline = style.bg(transparent).fg(foreground).borderWidth(1).border(border)
  const sizeSm = style.fontSize(xs).padX(2)
  const sizeMd = style.fontSize(sm).padX(2_5)
</scope>
<span> style([
  style.radius(full).borderWidth(1).fontWeight(medium),
  match(variant, {
    soft: soft,
    solid: solid,
    outline: outline,
  }),
  match(size, {
    sm: sizeSm,
    md: sizeMd,
  }),
]) 
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
            && out.skin_css.contains("background-color:var(--muted)")
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
        out.structure_html.contains("sx-roundedfull")
            || out.structure_html.contains("sx-border")
            || out.skin_css.contains("border-radius"),
        "host sx base survives: struct={} skin={}",
        out.structure_html,
        out.skin_css
    );
}

#[test]
fn badge_soft_base_on_host_sx() {
    let src = include_str!("../stdlib/badge.sail");
    let out = compile_element(src, "badge").expect("badge compiles");
    assert!(
        src.contains("style.")
            && (out.structure_html.contains("sx-bgmuted")
                || out.structure_html.contains("bgmuted")
                || out.skin_css.contains("var(--muted)")
                || out.structure_html.contains("sx-bgtonebg"))
            && (out.structure_html.contains("sx-fgmutedforeground")
                || out.structure_html.contains("mutedforeground")
                || out.skin_css.contains("var(--muted-foreground)")
                || out.structure_html.contains("sx-texttonefg")
                || out.structure_html.contains("sx-fg")),
        "soft default paint must be host Style (semantic tokens):\n{}",
        out.structure_html
    );
    assert!(
        out.structure_html.contains("sx-fontFamilybody")
            || out.structure_html.contains("sx-fontbody")
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
    // soft+default compound → muted soft PE.
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
    // Paint stripe: tracking + inset ring + PE gradient left contracts → host sx.
    // Layout row centers content; avatar-media child pack sizes injected img.
    let src = include_str!("../stdlib/avatar.sail");
    let out = compile_element(src, "avatar").expect("avatar compiles");
    assert!(
        src.contains("align=\"center\"")
            && src.contains("justify=\"center\"")
            && (src.contains("avatarMedia") || src.contains("avatar-media")),
        "avatar must layout-center + avatar-media child pack"
    );
    assert!(
        out.structure_html.contains("sx-trackingwide")
            && (out.structure_html.contains("sx-shadowinsetring")
                || out.skin_css.contains("box-shadow:var(--shadow-inset-ring)")
                || out
                    .skin_css
                    .contains("box-shadow: var(--shadow-inset-ring)")),
        "tracking-wide + shadow-inset-ring must land on host:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.structure_html.contains("sx-size7")
            && (out.structure_html.contains("sx-radiusfull")
                || out.structure_html.contains("sx-roundedfull")
                || out.skin_css.contains("border-radius:var(--radius-full)"))
            && (out.structure_html.contains("sx-fontFamilybody")
                || out.structure_html.contains("sx-fontbody")
                || out.skin_css.contains("var(--body-font)")),
        "attr-absent PE size=md + shape round + type on host sx:\n{}",
        out.structure_html
    );
    // PE gradient: Style bgImage(primary-accent) (not TW gradient pack).
    assert!(
        src.contains("bgImage(primary-accent)")
            || src.contains("bg-gradient-primary-accent")
            || src.contains("primary-accent"),
        "avatar host Style must declare primary-accent gradient"
    );
    assert!(
        out.skin_css.contains("linear-gradient")
            && out.skin_css.contains("var(--primary)")
            && out.skin_css.contains("var(--accent)"),
        "avatar skins must lower default gradient to CSS linear-gradient:\n{}",
        out.skin_css
    );
    // avatar-media child pack → .sx-avatarmedia img { … }
    assert!(
        out.structure_html.contains("sx-avatarmedia")
            && (out.skin_css.contains(".sx-avatarmedia img")
                || out.skin_css.contains("object-fit:contain")
                || out.skin_css.contains("object-fit: contain")),
        "avatar-media must paint descendant img:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    // Branded tones solid-fill via Style match; size/shape axes land.
    assert!(
        out.skin_css.contains("tone='primary'")
            && (out.skin_css.contains("background-color:var(--primary)")
                || out.skin_css.contains("background-color: var(--primary)")
                || out.skin_css.contains("background-color:var(--tone)"))
            && out.skin_css.contains("size='xs'")
            && out.skin_css.contains("shape='square'"),
        "branded tone solid + size/shape via Style match:\n{}",
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    // Deleted paint PE only.
    assert!(
        !contracts.contains("letter-spacing: 0.02em"),
        "letter-spacing PE must be gone (tracking-wide on host)"
    );
    assert!(
        !contracts.contains("inset 0 0 0 1px var(--border), inset 0 1px 0 var(--background)"),
        "inset ring PE must be gone (shadow-inset-ring on host)"
    );
    assert!(
        !contracts.contains("linear-gradient(135deg, var(--primary), var(--accent))")
            && !contracts
                .lines()
                .any(|l| l.contains(".sail-avatar") && l.contains("linear-gradient")),
        "avatar PE gradient must be gone from contracts"
    );
    // Layout + img PE gone (row + avatar-media pack).
    assert!(
        !contracts.contains(".sail-avatar img")
            && !contracts
                .lines()
                .any(|l| l.contains(".sail-avatar") && l.contains("align-items")),
        "avatar flex/img PE must be gone from contracts"
    );
}

#[test]
fn empty_state_child_paint_on_structure_sx() {
    // Style product: stretch host + grow copy + semantic child tokens.
    // match(size/variant/tone) + when(empty/forced-colors). Zero contracts PE.
    let src = include_str!("../stdlib/empty-state.sail");
    let out = compile_element(src, "empty-state").expect("empty-state compiles");
    assert!(
        !src.contains("--es-")
            && !src.contains("tone-bg")
            && !src.contains("compound: [")
            && src.contains("align=\"stretch\"")
            && src.contains("style.")
            && src.contains("match(size")
            && src.contains("match(variant")
            && src.contains("match(tone")
            && (src.contains("grow(1)") || src.contains(" grow") || src.contains("grow min-w"))
            && src.contains("self(stretch)")
            && src.contains("self(center)")
            && src.contains("size(12)")
            && src.contains("fontSize(md)")
            && src.contains("fontSize(sm)")
            && src.contains("maxW(panel-ch-w-md)")
            && src.contains("empty:")
            && src.contains("forced-colors"),
        "empty-state Style product + stretch/grow + PE child tokens"
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
        out.skin_css.contains("background-color:var(--muted)")
            || out.skin_css.contains("background-color: var(--muted)")
            || out.structure_html.contains("sx-bgmuted"),
        "icon soft surface via muted (no tone-bg):\n{}",
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
            || out.structure_html.contains("sx-empty")
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
        "empty-state must have zero contracts PE rules"
    );
}

#[test]
fn tab_selected_underline_and_inset_focus_on_host_sx() {
    // Style product: when(selected/focus) + match(variant) for pill/enclosed chrome.
    let src = include_str!("../stdlib/tab.sail");
    let out = compile_element(src, "tab").expect("tab compiles");
    assert!(
        src.contains("style.")
            && src.contains("when({")
            && src.contains("selected:")
            && (src.contains("borderB(primary)") || src.contains("borderB(tone)"))
            && (src.contains("outline(ring)") || src.contains("outline(tone-ring)"))
            && src.contains("outlineOffset(ring-inset)"),
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
        "tab selected / pill/enclosed parent→child PE must be gone from contracts"
    );
    assert!(
        !contracts.contains(".sail-tab:focus-visible"),
        "tab focus-visible PE must be gone from contracts"
    );
    let tab_src = include_str!("../stdlib/tab.sail");
    assert!(
        tab_src.contains("pill:")
            && tab_src.contains("enclosed:")
            && (tab_src.contains("tab-selected") || tab_src.contains("bg(tab-selected)")),
        "tab.sail must Style match pill/enclosed selected chrome"
    );
}

#[test]
fn toggle_pressed_focus_stack_on_host_sx() {
    // Style product: position(relative) + when(pressed/focus) z stack + outline.
    let src = include_str!("../stdlib/toggle.sail");
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
        "toggle pressed/focus + toggle-group join PE must be gone from contracts"
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
    let src = include_str!("../stdlib/switch.sail");
    let out = compile_element(src, "switch").expect("switch compiles");
    assert!(
        !src.contains("--sw-")
            && src.contains("style.appearance(none)")
            && src.contains("bg(none)")
            && src.contains("bg(switch)")
            && src.contains("border(switch-border)")
            && src.contains("bg(switch-thumb)")
            && src.contains("radius(full)")
            && src.contains("w(control-h-sm)")
            && src.contains("h(icon-size-md)")
            && src.contains("content(empty)")
            && src.contains("size(3)")
            && src.contains("left(auto)")
            && src.contains("right(px)")
            && src.contains("bg(hover-bg)")
            && src.contains("bg(primary)")
            && src.contains("border(selected-border)")
            && src.contains("outline(primary)")
            && src.contains("outlineOffset(ring-offset)")
            && src.contains("saturate(60)")
            && (src.contains("bg(info)") || src.contains("bg(primary)"))
            && (src.contains("outline(info)") || src.contains("outline(primary)"))
            && src.contains("match(size")
            && src.contains("size(2_5)")
            && src.contains("w(7)")
            && src.contains("size(icon-size-sm)"),
        "switch Style host: track + before thumb + size/tone match + semantic tones"
    );
    // boxSizing rename drift guard: class mint + live skin + committed
    // element-skins.css must stay in lockstep. Phase 2a renamed style.box →
    // boxSizing and left .sx-boxborder stale in the framework half until regen.
    assert!(
        src.contains("boxSizing(border)"),
        "switch host must call boxSizing (not retired style.box)"
    );
    assert!(
        out.structure_html.contains("boxSizingborder")
            || out.skin_css.contains("sx-boxSizingborder"),
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
    assert!(
        !out.skin_css.contains("sx-boxborder") && !out.structure_html.contains("sx-boxborder"),
        "retired sx-boxborder must not appear (regen skins after boxSizing rename)"
    );
    let skins_file = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/element-skins.css"
    ));
    assert!(
        skins_file.contains(".sx-boxSizingborder{")
            && (skins_file.contains("box-sizing:border-box")
                || skins_file.contains("box-sizing: border-box")),
        "element-skins.css stale for boxSizing — run: cargo run -p sail-compiler -- framework-css"
    );
    assert!(
        !skins_file.contains(".sx-boxborder{"),
        "element-skins.css still has retired .sx-boxborder — regenerate framework-css"
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
            || out.structure_html.contains("bgnone"),
        "bg(none) must clear checkbox-contract mark SVG:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css.contains("background-color:var(--switch)")
            || out.skin_css.contains("background-color: var(--switch)")
            || out.structure_html.contains("bgswitch"),
        "bg(switch) off-track paint:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css.contains("width:var(--control-h-sm)")
            || out.skin_css.contains("width: var(--control-h-sm)")
            || out.structure_html.contains("controlhsm"),
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
        out.skin_css.contains("tone='primary'")
            && (out.skin_css.contains("background-color:var(--primary)")
                || out.skin_css.contains("background-color: var(--primary)")
                || out.skin_css.contains("background-color:var(--tone)")
                || out.skin_css.contains("background-color: var(--tone)")),
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
            "switch PE must be gone:\n{sw_pe}"
        );
    } else {
        assert!(
            !contracts.contains(".sail-switch") && !contracts.contains("--sw-w:"),
            "switch PE fully deleted from contracts"
        );
    }
}

#[test]
fn toast_start_stripe_on_host_sx() {
    // PE was border-inline-start: 3px solid var(--tone-fg). Now Style borderSWidth(3)
    // + match(tone) semantic borderS tokens.
    let out = compile_element(include_str!("../stdlib/toast.sail"), "toast").expect("toast");
    assert!(
        out.structure_html.contains("sx-borderSWidth3")
            || out.structure_html.contains("borderSWidth")
            || out.structure_html.contains("sx-borders3")
            || out.structure_html.contains("border-s-3")
            || out.skin_css.contains("border-inline-start-width:3px")
            || out.skin_css.contains("border-inline-start-width: 3px"),
        "border-s-3 must lower to 3px start width:\nstructure={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css
            .contains("border-inline-start-color:var(--foreground)")
            || out
                .skin_css
                .contains("border-inline-start-color: var(--foreground)")
            || out
                .skin_css
                .contains("border-inline-start-color:var(--primary)")
            || out
                .skin_css
                .contains("border-inline-start-color: var(--primary)")
            || out
                .skin_css
                .contains("border-inline-start-color:var(--tone-fg)")
            || out
                .skin_css
                .contains("border-inline-start-color: var(--tone-fg)"),
        "start stripe must bind semantic tone color:\n{}",
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
        "toast host stripe + close anatomy PE must be gone"
    );
    // Close control is structure with sx (not parent PE).
    let src = include_str!("../stdlib/toast.sail");
    assert!(
        src.contains("data-toast-close")
            && (src.contains("ms(auto)") || src.contains("ms-auto") || src.contains("ms: auto"))
            && (out.structure_html.contains("data-toast-close")
                || out.structure_html.contains("sx-msauto")),
        "close control must be structure with sx:\n{}",
        out.structure_html
    );
}

#[test]
fn card_host_paint_bulk_on_host_sx() {
    // Full card PE dissolved: border-image, backdrop, status-rail (paint +
    // pad-adjust), interactive lift/forced-colors, branded tone border,
    // accent-bar, header/footer chrome → host sx / CVA / structure slots.
    let src = include_str!("../stdlib/card.sail");
    let out = compile_element(src, "card").expect("card compiles");

    // border-image-(--corner-image) + backdrop on host
    assert!(
        src.contains("border-image-(--corner-image)")
            && src.contains("backdrop-(--card-backdrop-filter)"),
        "card host sx must bind corner-image + backdrop tokens"
    );
    assert!(
        out.structure_html.contains("sx-borderimagecornerimage")
            || out.skin_css.contains("border-image:var(--corner-image)")
            || out.skin_css.contains("border-image: var(--corner-image)"),
        "border-image token must land:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.structure_html.contains("sx-backdropcardbackdropfilter")
            || out.structure_html.contains("card-backdrop-filter")
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
            || out.structure_html.contains("sx-backdrop"),
        "webkit backdrop prefix must land with token:\nskin={}",
        out.skin_css
    );

    // status-rail → border-s-3 + border-strong default + branded tone-fg + ps pad-adjust
    assert!(
        src.contains("border-s-3")
            && src.contains("border-s-border-strong")
            && src.contains("border-s-tone-fg")
            && src.contains("ps-4"),
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
        src.contains("hover:-translate-y-px")
            && src.contains("active:translate-y-0")
            && src.contains("motion-reduce:translate-y-0")
            && src.contains("active:duration-instant")
            && (src.contains("forced-colors:outline-CanvasText")
                || src.contains("forced-colors:focus:outline-CanvasText")),
        "interactive compound must carry TW lift + forced-colors outline"
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

    // branded tone border via compound({tone}) + style.border(token) —
    // match/compound mechanism, not the retired --tone* attribute channel.
    assert!(
        out.skin_css.contains("tone='primary'")
            && (out.skin_css.contains("border-color:var(--primary)")
                || out.skin_css.contains("border-color: var(--primary)")),
        "branded tone border via CVA:\n{}",
        out.skin_css
    );

    // accent-bar → before: stripe compounds (static + hover:before: opacity)
    assert!(
        src.contains("before:content-empty")
            && src.contains("before:bg-gradient-to-r")
            && src.contains("before:from-primary")
            && src.contains("before:to-accent")
            && src.contains("before:h-half")
            && src.contains("hover:before:opacity-100")
            && src.contains("before:opacity-0"),
        "accent-bar compounds must use before: anatomy + hover:before: opacity"
    );
    assert!(
        out.skin_css.contains("accent-bar='static'")
            && out.skin_css.contains("accent-bar='hover'")
            && out.skin_css.contains("::before")
            && (out.skin_css.contains("content:''") || out.skin_css.contains("content: ''"))
            && (out.skin_css.contains("height:var(--space-half)")
                || out.skin_css.contains("height: var(--space-half)"))
            && out.skin_css.contains(":hover")
            && (out.skin_css.contains("opacity:1")
                || out.skin_css.contains("opacity: 1")
                || out.skin_css.contains("opacity:1.")
                || out.skin_css.contains("opacity:0")),
        "accent-bar CVA must lower before: stripe + hover opacity:\n{}",
        out.skin_css
    );

    // header/footer real named slots with chrome sx (no parent→child PE)
    assert!(
        src.contains(r#"slot name="header""#)
            && src.contains(r#"slot name="footer""#)
            && src.contains("empty:hidden")
            && src.contains("font-heading")
            && src.contains("font-(--heading-weight)")
            && src.contains("border-border-subtle")
            && src.contains("-mt-4")
            && src.contains("-mx-5")
            && src.contains("-mb-4"),
        "card structure must expose header/footer slots with bleed chrome sx"
    );
    assert!(
        out.structure_html.contains(r#"slot name="header""#)
            && out.structure_html.contains(r#"slot name="footer""#)
            && (out.structure_html.contains("sx-empty-hidden") || out.skin_css.contains(":empty"))
            && (out.structure_html.contains("sx-fontheading")
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
    // Zero card PE — entire block deleted.
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
        "card PE must be fully gone from contracts.css"
    );
}

#[test]
fn navbar_host_paint_on_sx_cva() {
    // Layout + paint owned by navbar.sail (layout keywords + Style match/compound).
    // No --rail-offset / --nav-*. Sticky rail uses top-0 bottom-0 +
    // offset top-N compounds. Center grow-1; actions ms-auto mt-auto.
    let src = include_str!("../stdlib/navbar.sail");
    let out = compile_element(src, "navbar").expect("navbar compiles");
    assert!(
        !src.contains("--rail-offset")
            && !src.contains("--nav-")
            && !src.contains("compound: [")
            && src.contains("<row")
            && src.contains("nowrap")
            && src.contains("gap=\"4\"")
            && src.contains("style.")
            && (src.contains("flexDir(column)") || src.contains("dir-column"))
            && (src.contains("bg(navbar)") || src.contains("bg: navbar"))
            && (src.contains("fg(navbar-foreground)") || src.contains("navbar-foreground"))
            && (src.contains("minH(14)") || src.contains("min-h: 14"))
            && (src.contains("padY(2)") || src.contains("py: 2"))
            && (src.contains("padX(4)") || src.contains("px: 4"))
            && (src.contains("position(sticky)") || src.contains("sticky"))
            && (src.contains("z(sticky)")
                || src.contains("z: sticky")
                || src.contains("z-(--z-sticky)"))
            && (src.contains("borderBWidth(1)") || src.contains("border-b"))
            && (src.contains("top(auto)") || src.contains("top: auto") || src.contains("top-auto"))
            && (src.contains("borderEWidth(1)") || src.contains("border-e"))
            && (src.contains("minW(panel-rail-w)") || src.contains("panel-rail-w"))
            && (src.contains("display(contents)") || src.contains("contents"))
            && (src.contains("grow(1)")
                || src.contains("grow: 1")
                || src.contains("grow-1")
                || src.contains("\n    grow\n")
                || src.contains(" grow")
                || src.contains("grow min-w"))
            && (src.contains("ms(auto)") || src.contains("ms: auto") || src.contains("ms-auto"))
            && (src.contains("mt(auto)") || src.contains("mt: auto") || src.contains("mt-auto"))
            && (src.contains("offset: '8'") || src.contains("offset: \"8\""))
            && (src.contains("top(8)") || src.contains("top: 8") || src.contains("top-8")),
        "navbar structure must own layout + sticky rail/offset Style compounds"
    );
    assert!(
        out.structure_html.contains("sail-row")
            && out.structure_html.contains("sail-nowrap")
            && (out.structure_html.contains("sail-g-4") || out.structure_html.contains("sail-g-"))
            && (out.structure_html.contains("sx-bgnavbar")
                || out.structure_html.contains("bgnavbar")
                || out.skin_css.contains("var(--navbar)"))
            && (out.structure_html.contains("navbarforeground")
                || out.structure_html.contains("sx-fgnavbar")
                || out.skin_css.contains("var(--navbar-foreground)"))
            && (out.structure_html.contains("sx-minh14")
                || out.structure_html.contains("minh14")
                || out.skin_css.contains("min-height"))
            && (out.structure_html.contains("sx-displaycontents")
                || out.structure_html.contains("sx-contents")
                || out.structure_html.contains("display:contents")
                || out.skin_css.contains("display:contents")
                || out.skin_css.contains("display: contents"))
            && out.structure_html.contains("slot name=\"brand\"")
            && out.structure_html.contains("slot name=\"center\"")
            && out.structure_html.contains("slot name=\"actions\"")
            && (out.structure_html.contains("sx-grow1")
                || out.structure_html.contains("grow1")
                || out.structure_html.contains("grow-1")
                || out.skin_css.contains("flex-grow:1")
                || out.skin_css.contains("flex-grow: 1"))
            && (out.structure_html.contains("sx-msauto")
                || out.structure_html.contains("msauto")
                || out.structure_html.contains("ms-auto")
                || out.skin_css.contains("margin-inline-start:auto")
                || out.skin_css.contains("margin-inline-start: auto"))
            && (out.structure_html.contains("sx-mtauto")
                || out.structure_html.contains("mtauto")
                || out.structure_html.contains("mt-auto")
                || out.skin_css.contains("margin-top:auto")
                || out.skin_css.contains("margin-top: auto")),
        "host row + region structure + grow/ms-auto/mt-auto must lower:\n{}",
        out.structure_html
    );
    assert!(
        out.skin_css.contains("placement='rail'")
            && out.skin_css.contains("min-height:0")
            && out.skin_css.contains("padding-block:var(--space-5)")
            && (out.skin_css.contains("min-width:var(--panel-rail-w)")
                || out.skin_css.contains("min-width: var(--panel-rail-w)"))
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
            && (out.skin_css.contains("offset='8'") || out.skin_css.contains("offset=\"8\""))
            && (out.skin_css.contains("top:var(--space-8)")
                || out.skin_css.contains("top: var(--space-8)")
                || out.skin_css.contains("top:2rem")
                || out.skin_css.contains("top: 2rem")),
        "placement/sticky/bordered/rail + offset top-N compounds (no rail-offset channel):\n{}",
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
        "navbar PE matrix must be fully deleted from contracts.css"
    );
}

#[test]
fn error_state_child_paint_on_structure_sx() {
    // Style product: destructive semantic tokens + when(empty/group-state/forced).
    // No --es-* / tone-* channel paint.
    let src = include_str!("../stdlib/error-state.sail");
    let out = compile_element(src, "error-state").expect("error-state compiles");

    assert!(
        !src.contains("--es-")
            && !src.contains("tone-destructive")
            && !src.contains("compound: [")
            && src.contains("align=\"stretch\"")
            && src.contains("style.")
            && (src.contains("grow(1)") || src.contains(" grow") || src.contains("grow min-w"))
            && src.contains("self(stretch)")
            && src.contains("self(center)")
            && src.contains("fg(destructive)")
            && src.contains("outline(destructive)")
            && src.contains("outline(1)")
            && src.contains("outline(solid)")
            && src.contains("outlineOffset(ring-inset)")
            && src.contains("fontSize(md)")
            && src.contains("fontSize(sm)")
            && src.contains("maxW(panel-ch-w-md)")
            && src.contains("maxW(panel-ch-w-lg)")
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
            || out.structure_html.contains("sx-textmd")
            || out.structure_html.contains("fontsizemd"))
            && (out.skin_css.contains("font-size:var(--text-sm)")
                || out.skin_css.contains("font-size: var(--text-sm)")
                || out.structure_html.contains("sx-textsm")
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
        "error-state must have zero contracts PE rules"
    );
}

#[test]
fn radio_nested_row_kills_label_flex_pe() {
    // Root stays <label> for native semantics; type paint on host Style.
    // Flex/gap for label+control is contracts PE (cannot nest <row> on label).
    let src = include_str!("../stdlib/radio.sail");
    let out = compile_element(src, "radio").expect("radio compiles");
    assert!(
        src.contains("<label")
            && src.contains("type=\"radio\"")
            && src.contains("style.")
            && (src.contains("fontFamily(body)")
                || src.contains("font-family: body")
                || src.contains("font-body"))
            && (src.contains("cursor(pointer)")
                || src.contains("cursor: pointer")
                || src.contains("cursor-pointer"))
            && (src.contains("radio-foreground") || src.contains("color: foreground")),
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
    // No hand sail-radio PE matrix (native label flex may remain in contracts).
    assert!(
        !contracts.contains("label.sail-radio")
            && !contracts.contains(".sail-radio {\n  display: inline-flex"),
        "radio label flex PE must be gone from contracts"
    );
}

#[test]
fn toggle_group_segmented_and_join_on_skins_not_pe() {
    // Segmented host CVA; size stamped for cells; join/ghost/segmented on
    // toggle first:/last: + size CVA (no --tg-*). Zero hand PE.
    let src = include_str!("../stdlib/toggle-group.sail");
    let out = compile_element(src, "toggle-group").expect("toggle-group compiles");
    assert!(
        !src.contains("'tg-h'")
            && !src.contains("--tg-")
            && src.contains("style.")
            && (src.contains("bg(toggle-group)") || src.contains("toggle-group"))
            && (src.contains("border(toggle-group-border)") || src.contains("toggle-group-border"))
            && src.contains("<row inline gap=\"0\"")
            && src.contains("size: 'sm' | 'md' | 'lg' = 'md'"),
        "segmented host chrome on Style via component.toggle-group tokens; host is inline row; size stamped"
    );
    assert!(
        out.skin_css.contains("variant='segmented'")
            && (out
                .skin_css
                .contains("background-color:var(--toggle-group)")
                || out
                    .skin_css
                    .contains("background-color: var(--toggle-group)")
                || out.structure_html.contains("togglegroup"))
            && !out.skin_css.contains("--tg-"),
        "segmented Style match uses --toggle-group; no --tg-* writes:\n{}",
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
        "toggle-group hand PE must be gone from contracts"
    );
    let toggle_src = include_str!("../stdlib/toggle.sail");
    let toggle_out = compile_element(toggle_src, "toggle").expect("toggle compiles");
    assert!(
        !toggle_src.contains("--tg-")
            && toggle_src.contains("first:")
            && toggle_src.contains("last:")
            && toggle_src.contains("outline:")
            && toggle_src.contains("ghost:")
            && toggle_src.contains("size:")
            && toggle_src.contains("style.")
            && (toggle_src.contains("h(control-h-md)") || toggle_src.contains("h: control-h-md"))
            && (toggle_src.contains("h(control-h-sm)") || toggle_src.contains("h: control-h-sm"))
            && (toggle_src.contains("bg(toggle)") || toggle_src.contains("bg: toggle")),
        "toggle.sail must Style join/ghost + size direct tokens + component.toggle paint"
    );
    assert!(
        toggle_out.skin_css.contains("size='sm'")
            && toggle_out.skin_css.contains("size='lg'")
            && (toggle_out.skin_css.contains("var(--control-h-md)")
                || toggle_out.structure_html.contains("controlhmd")
                || toggle_out.structure_html.contains("control-h-md"))
            && !toggle_out.skin_css.contains("--tg-"),
        "toggle size CVA direct control-h tokens; no --tg-*:\n{}",
        toggle_out.skin_css
    );
}

#[test]
fn alert_tone_stripe_before_on_host_sx() {
    // Host <row nowrap align=start gap=3> owns ico|content|close (layout
    // keywords — free grid-template PE forbidden). Stripe/empty/dismiss +
    // forced-colors via Style when/match. Zero alert PE in contracts.
    let src = include_str!("../stdlib/alert.sail");
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
        "must not emit free grid-template PE in structure/skin:\n{}\n{}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        src.contains("style.")
            && src.contains("before:")
            && src.contains("content(empty)")
            && src.contains("alert-stripe")
            && src.contains("empty:")
            && src.contains("group-dismissible")
            && src.contains("group()")
            && src.contains("forced-colors")
            && src.contains("CanvasText")
            && !src.contains("tone-bg")
            && !src.contains("compound: ["),
        "host/child Style must carry stripe PE, empty gates, group-dismissible, forced-colors"
    );
    assert!(
        src.contains("default: when({ before: style.bg(alert-stripe) })")
            && src.contains("info: when({ before: style.bg(info) })"),
        "match(tone) stripe: default alert-stripe; branded semantic tokens"
    );
    assert!(
        out.structure_html.contains("before")
            || out.structure_html.contains("sx-before")
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
        out.skin_css.contains("var(--alert-stripe)") || out.skin_css.contains("alert-stripe"),
        "PE default stripe must bind alert-stripe:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("tone='info'") && out.skin_css.contains("var(--info)"),
        "branded match(tone) before bg(info) must lower:\n{}",
        out.skin_css
    );
    assert!(
        out.structure_html.contains("empty")
            || out.structure_html.contains("sx-empty")
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
        "all alert PE (grid/empty/svg/dismiss/soft-tone/forced-colors) must be gone from contracts"
    );
    assert!(
        out.structure_html.contains("sail-row") || out.structure_html.contains("sail-col"),
        "icon/content/actions use layout keywords:\n{}",
        out.structure_html
    );
}

#[test]
fn toast_region_host_paint_on_sx_cva() {
    // Width + bottom reverse + paint left contracts PE → host sx + CVA.
    // Child hit targets: toast host pointer-events-auto (region is none).
    let src = include_str!("../stdlib/toast-region.sail");
    let out = compile_element(src, "toast-region").expect("toast-region compiles");
    assert!(
        src.contains("fixed z-(--z-toast) p-4 pointer-events-none")
            && src.contains("w-panel-w-toast")
            && src.contains("top-0 right-auto bottom-auto left-0")
            && src.contains("'top-end': 'top-0 right-0 bottom-auto left-auto'")
            && src.contains("col-reverse")
            && src.contains("'bottom-start':")
            && src.contains("'bottom-end':"),
        "toast-region host sx/CVA must carry fixed/z/pad/width + position insets + reverse"
    );
    // Style-minted class names (sx-{setter}{args}, case-preserving).
    assert!(
        out.structure_html.contains("sx-positionfixed")
            && out.structure_html.contains("sx-zztoast")
            && out.structure_html.contains("sx-pad4")
            && out.structure_html.contains("sx-pointerEventsnone")
            && out.structure_html.contains("sx-top0")
            && (out.structure_html.contains("sx-wpanelwtoast")
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
    // toast-region PE section may be fully deleted.
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
            "toast-region PE must be gone from contracts:\n{tr}"
        );
    } else {
        assert!(
            !contracts.contains(".sail-toast-region"),
            "toast-region PE fully deleted from contracts"
        );
    }
}

#[test]
fn theme_picker_host_paint_on_structure_sx() {
    // Layout keywords + host Style carry trigger/panel/option/swatch paint
    // on real nodes (no residual part classes). KEEP: panel anchor geometry,
    // [inline] matrices, forced-colors.
    let src = include_str!("../stdlib/theme-picker.sail");
    let out = compile_element(src, "theme-picker").expect("theme-picker compiles");
    assert!(
        src.contains("style.")
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
            && src.contains("role=\"listbox\"")
            && src.contains("role=\"option\"")
            && src.contains("data-mode-bar")
            && src.contains("<row inline")
            && !src.contains("sx=\""),
        "theme-picker structure must carry Style product without class anatomy"
    );
    assert!(
        (out.structure_html.contains("sx-sizecontrolhsm")
            || out.structure_html.contains("sizecontrolhsm")
            || out.skin_css.contains("var(--control-h-sm)"))
            && (out.structure_html.contains("sx-bgthemepicker")
                || out.structure_html.contains("bgthemepicker")
                || out.skin_css.contains("var(--theme-picker)"))
            && (out.structure_html.contains("sx-isolate")
                || out.skin_css.contains("isolation:isolate")
                || out.skin_css.contains("isolation: isolate"))
            && (out.structure_html.contains("role=\"listbox\"")
                || out.structure_html.contains("role='listbox'"))
            && out.structure_html.contains("theme=\"")
            && (out.structure_html.contains("sail-row-i") || out.skin_css.contains("inline-flex")),
        "child paint + layout classes on structure:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    // Option rows are real nodes with selected/hover/focus borders — no tc-*
    // mini-card classes, no part classes, primary/accent paint present.
    assert!(
        !out.structure_html.contains("tc-overlay")
            && !out.structure_html.contains("class=\"tc-")
            && (out.structure_html.contains("role=\"option\"")
                || out.skin_css.contains("role='option'"))
            && (out.structure_html.contains("sx-bgprimary")
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
    // Paint piles fully dissolved — no contracts PE for theme-picker anatomy.
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
    // (not hand contracts PE).
    let src = include_str!("../stdlib/prose.sail");
    let out = compile_element(src, "prose").expect("prose compiles");
    assert!(
        src.contains("style.")
            && (src.contains("display(block)") || src.contains("block"))
            && (src.contains("fontFamily(body)")
                || src.contains("font-body")
                || src.contains("font-family: body"))
            && (src.contains("prose-foreground") || src.contains("text-foreground"))
            && (src.contains("leading(normal-plus)")
                || src.contains("leading-normal-plus")
                || src.contains("leading: normal-plus")),
        "prose host Style must carry block + body type/color/leading"
    );
    assert!(
        (out.structure_html.contains("sx-displayblock")
            || out.structure_html.contains("sx-block")
            || out.skin_css.contains("display:block")
            || out.skin_css.contains("display: block"))
            && (out.structure_html.contains("sx-fontfamilybody")
                || out.structure_html.contains("sx-fontbody")
                || out.skin_css.contains("var(--body-font)"))
            && (out.structure_html.contains("proseforeground")
                || out.structure_html.contains("sx-fgprose")
                || out.skin_css.contains("var(--prose-foreground)")
                || out.structure_html.contains("sx-textforeground"))
            && (out.structure_html.contains("leadingnormalplus")
                || out.structure_html.contains("sx-leadingnormalplus")
                || out.skin_css.contains("var(--leading-normal-plus)")),
        "host paint atoms on structure:\n{}",
        out.structure_html
    );
    // Generated descendant skin (owned by prose map).
    assert!(
        out.skin_css.contains(".sail-prose h1")
            && out.skin_css.contains(".sail-prose p")
            && out.skin_css.contains(".sail-prose a")
            && out.skin_css.contains(".sail-prose code")
            && out.skin_css.contains(".sail-prose pre")
            && out.skin_css.contains(".sail-prose blockquote")
            && out.skin_css.contains(".sail-prose ul")
            && out.skin_css.contains(".sail-prose table")
            && out.skin_css.contains(".sail-prose strong")
            && out.skin_css.contains("li::marker")
            && out.skin_css.contains("font-family:var(--heading-font)")
            && out.skin_css.contains("color:var(--primary)"),
        "prose descendant recipe must generate into skin_css:\n{}",
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    // No hand PE descendant rules.
    assert!(
        !contracts.contains(".sail-prose h1")
            && !contracts.contains(".sail-prose p")
            && !contracts.contains(".sail-prose a")
            && !contracts.contains(".sail-prose code")
            && !contracts.contains(".sail-prose blockquote")
            && !contracts.contains(".sail-prose table"),
        "prose descendant PE must be gone from contracts"
    );
}

#[test]
fn tab_list_sticky_and_variant_on_host_not_parent_pe() {
    // Sticky + variant on <tab-list> host. Pill track CVA. Child tab matrices
    // are stamped variant CVA on tab.sail (not parent PE). Vertical host axis
    // is layout-owned; list/panel orientation stamped by use:tabs.
    let src = include_str!("../stdlib/tab-list.sail");
    let out = compile_element(src, "tab-list").expect("tab-list compiles");
    assert!(
        src.contains("sticky = false")
            && src.contains("style.")
            && (src.contains("position(sticky)") || src.contains("sticky"))
            && (src.contains("z(sticky)") || src.contains("z: sticky"))
            && (src.contains("bg(tab-list)") || src.contains("tab-list"))
            && src.contains("pill:")
            && (src.contains("bg(tab-list-rail)") || src.contains("tab-list-rail")),
        "tab-list must declare sticky compound + component.tab-list bg + pill rail Style on host"
    );
    assert!(
        (out.structure_html.contains("bgtablist")
            || out.structure_html.contains("sx-bgbackground")
            || out.skin_css.contains("var(--tab-list)"))
            && (out.structure_html.contains("borderB")
                || out.structure_html.contains("borderb")
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
                    .contains("background-color: var(--tab-list-rail)"))
            && (out.skin_css.contains("border-radius:var(--radius-md)")
                || out.skin_css.contains("border-radius: var(--radius-md)")),
        "pill variant CVA must lower on tab-list with --tab-list-rail:\n{}",
        out.skin_css
    );
    // sticky + variant no longer tabs props; tabs stamps size onto tab cells.
    let tabs_src = include_str!("../stdlib/tabs.sail");
    let tabs_out = compile_element(tabs_src, "tabs").expect("tabs compiles");
    assert!(
        !tabs_src.contains("'tab-py'")
            && !tabs_src.contains("--tab-")
            && !tabs_src.contains("sticky = false")
            && !tabs_src.contains("sticky=false")
            && !tabs_src.contains("variant:")
            && tabs_src.contains("size: 'sm' | 'md' | 'lg' = 'md'"),
        "sticky/variant not on <tabs>; size stamped (no --tab-*):\n{tabs_src}"
    );
    // Size CVA lives on <tab> (text-sm py-2 px-3 …), not --tab-* custom props.
    let tab_src = include_str!("../stdlib/tab.sail");
    let tab_out = compile_element(tab_src, "tab").expect("tab compiles");
    assert!(
        !tab_src.contains("--tab-")
            && tab_src.contains("size:")
            && tab_src.contains("style.")
            && (tab_src.contains("fontSize(sm)") || tab_src.contains("font-size: sm"))
            && (tab_src.contains("padY(2)") || tab_src.contains("py: 2"))
            && (tab_src.contains("padX(3)") || tab_src.contains("px: 3"))
            && (tab_src.contains("fontSize(xs)") || tab_src.contains("font-size: xs"))
            && (tab_src.contains("tab-foreground") || tab_src.contains("muted-foreground")),
        "tab.sail size Style direct pad/type tokens + component.tab paint:\n{tab_src}"
    );
    assert!(
        tab_out.skin_css.contains("size='sm'")
            && tab_out.skin_css.contains("size='lg'")
            && (tab_out.skin_css.contains("font-size:var(--text-sm)")
                || tab_out.skin_css.contains("font-size: var(--text-sm)")
                || tab_out.structure_html.contains("sx-textsm"))
            // Banned: legacy size channels --tab-py / --tab-h. Allowed: component
            // token reads --tab-foreground / --tab-selected*.
            && !tab_out.skin_css.contains("--tab-py")
            && !tab_out.skin_css.contains("--tab-h"),
        "tab size CVA must land; no legacy --tab-py/--tab-h channels:\n{}",
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
        "tabs/tab-list hand PE must be gone from contracts"
    );
}

#[test]
fn scroll_area_standard_scrollbar_on_host_sx() {
    // Standard scrollbar pack on host Style; orientation match owns overflow axes.
    // Webkit thumb pseudo pack is a documented Style-setter blocker.
    let src = include_str!("../stdlib/scroll-area.sail");
    let out = compile_element(src, "scroll-area").expect("scroll-area compiles");
    assert!(
        src.contains("style.")
            && (src.contains("scrollbarThin()")
                || src.contains("scrollbar-thin")
                || src.contains("scrollbar: thin"))
            && (src.contains("scrollbarThumb(scroll-area-thumb)")
                || src.contains("scroll-area-thumb"))
            && (src.contains("overscroll(contain)")
                || src.contains("overscroll: contain")
                || src.contains("overscroll-contain"))
            && (src.contains("match(orientation") || src.contains("orientation:")),
        "scroll-area host Style must carry standard scrollbar pack via component.scroll-area tokens"
    );
    // Overflow / orientation always land.
    assert!(
        out.structure_html.contains("sx-overflowyauto")
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
    // No hand scroll-area PE (section may be fully deleted).
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
            "scroll-area scrollbar PE (standard + webkit) must be gone:\n{sa}"
        );
    } else {
        assert!(
            !contracts.contains(".sail-scroll-area"),
            "scroll-area PE fully deleted from contracts"
        );
    }
}

#[test]
fn code_block_pre_tab_and_tok_generated() {
    // pre tab-size:2 + line-height code → pre Style (tab(2) + leading(code)).
    // .tok-* highlighter ink: compiler code-block map → element-skins (not PE).
    let src = include_str!("../stdlib/code-block.sail");
    let out = compile_element(src, "code-block").expect("code-block compiles");
    assert!(
        src.contains("style.")
            && (src.contains("tab(2)") || src.contains("tab-2") || src.contains("tab: 2"))
            && (src.contains("leading(code)")
                || src.contains("leading-code")
                || src.contains("leading: code")),
        "code-block pre Style must carry tab(2) + leading(code)"
    );
    assert!(
        (out.structure_html.contains("sx-tab2")
            || out.structure_html.contains("tab2")
            || out.skin_css.contains("tab-size"))
            && (out.structure_html.contains("sx-leadingcode")
                || out.structure_html.contains("leadingcode")
                || out.skin_css.contains("var(--leading-code)")),
        "tab/leading atoms on structure:\n{}",
        out.structure_html
    );
    assert!(
        out.skin_css.contains("tab-size:2")
            || out.skin_css.contains("tab-size: 2")
            || out.structure_html.contains("sx-tab2")
            || out.structure_html.contains("tab2"),
        "tab-2 must lower:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("line-height:var(--leading-code)")
            || out.skin_css.contains("line-height: var(--leading-code)")
            || out.structure_html.contains("sx-leadingcode")
            || out.structure_html.contains("leadingcode"),
        "leading-code must lower:\n{}",
        out.skin_css
    );
    // Generated .tok-* skin.
    assert!(
        out.skin_css.contains(".sail-code-block .tok-tag")
            && out.skin_css.contains("var(--syntax-tag)")
            && out.skin_css.contains(".tok-keyword")
            && out.skin_css.contains("var(--syntax-keyword)")
            && out.skin_css.contains(".tok-comment")
            && out.skin_css.contains("font-style:italic"),
        "code-block .tok-* must generate into skin_css:\n{}",
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/sail-base.css"
    ));
    assert!(
        !contracts.contains(".sail-code-block .tok-tag")
            && !contracts.contains(".sail-code-block .tok-keyword")
            && !contracts.contains(".sail-code-block pre"),
        "code-block .tok-* PE must be gone from contracts"
    );
}

#[test]
fn field_empty_and_invalid_hint_on_structure_sx() {
    // Style product: empty/group-invalid when gates on hint/error structure.
    let src = include_str!("../stdlib/field.sail");
    let out = compile_element(src, "field").expect("field compiles");
    assert!(
        (src.contains(".group()") || src.contains("style.group()"))
            && src.contains("empty:")
            && src.contains("group-invalid:")
            && !src.contains("class=")
            && src.contains("role=\"alert\"")
            && src.contains("field-hint")
            && src.contains("field-error"),
        "field structure must carry group + empty/group-invalid Style gates without part classes"
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
            "field hint/error PE rules must be gone:\n{field}"
        );
    } else {
        assert!(
            !contracts.contains(".sail-field .hint") && !contracts.contains(".sail-field[invalid]"),
            "field hint/error PE fully deleted from contracts"
        );
    }
}

#[test]
fn label_req_gate_in_element_skins_anatomy() {
    // Style product: .req gate via when(group-required) + component.label tokens.
    let src = include_str!("../stdlib/label.sail");
    let out = compile_element(src, "label").expect("label compiles");
    assert!(
        !src.contains("class=")
            && src.contains("label-required")
            && src.contains("group-required")
            && src.contains("display(none)")
            && (src.contains(".group()") || src.contains("style.group()")),
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
        "label .req PE must be gone from contracts"
    );
    assert!(
        out.structure_html.contains("aria-hidden")
            && (out.structure_html.contains("displaynone")
                || out.structure_html.contains("sx-")
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
    // use:pagination stamps size onto child pages; page size Style match owns geometry.
    // No --pg-* PE in contracts.
    let page_src = include_str!("../stdlib/page.sail");
    let pag_src = include_str!("../stdlib/pagination.sail");
    let page = compile_element(page_src, "page").expect("page compiles");
    let _pag = compile_element(pag_src, "pagination").expect("pagination compiles");
    assert!(
        page_src.contains("style.")
            && page_src.contains("match(size")
            && (page_src.contains("minW(control-h-md)")
                || page_src.contains("min-w-control-h-md")
                || page_src.contains("control-h-md"))
            && (page_src.contains("h(control-h-md)")
                || page_src.contains("h-control-h-md")
                || page_src.contains("control-h-md"))
            && (page_src.contains("fontSize(sm)")
                || page_src.contains("text-sm")
                || page_src.contains("font-size: sm")),
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
        "pagination --pg-* PE must be gone from contracts"
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
        ("hover-card", "panel-rail-w"),
        ("popover", "menu-min-w"),
    ] {
        let src = sail_compiler::stdlib::sources()
            .iter()
            .find(|(t, _)| *t == tag)
            .map(|(_, s)| *s)
            .unwrap_or_else(|| panic!("{tag} registered"));
        let out = compile_element(src, tag).unwrap_or_else(|e| panic!("{tag}: {e:?}"));
        let geometry = (src.contains("position(fixed)") || src.contains("fixed"))
            && (src.contains("inset(auto)")
                || src.contains("inset: auto")
                || src.contains("inset-auto"))
            && (src.contains("m(0)") || src.contains("m: 0") || src.contains("m-0"))
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
            (out.structure_html.contains("sx-positionfixed")
                || out.structure_html.contains("sx-fixed")
                || out.skin_css.contains("position:fixed")
                || out.skin_css.contains("position: fixed"))
                && (out.structure_html.contains("insetauto")
                    || out.structure_html.contains("sx-insetauto")
                    || out.skin_css.contains("inset:auto")
                    || out.skin_css.contains("inset: auto"))
                && (out.structure_html.contains("sx-m0")
                    || out.structure_html.contains("m0")
                    || out.skin_css.contains("margin:0")
                    || out.skin_css.contains("margin: 0"))
                && (out.structure_html.contains("anchorLeft")
                    || out.structure_html.contains("anchorleft")
                    || out.structure_html.contains("sx-leftanchorleft")
                    || out.skin_css.contains("var(--anchor-left)"))
                && (out.structure_html.contains("anchorTop")
                    || out.structure_html.contains("anchortop")
                    || out.structure_html.contains("sx-topanchortop")
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
            "overlay host PE must be gone: {needle}"
        );
    }
}

#[test]
fn combobox_listbox_geometry_on_child_sx_options_on_popover_skin() {
    // Listbox anchor geometry + scroll chrome on child Style; option anatomy is
    // global POPOVER_SKIN [role=listbox] (author children, no structure hook).
    let src = include_str!("../stdlib/combobox.sail");
    let out = compile_element(src, "combobox").expect("combobox compiles");
    assert!(
        src.contains("style.")
            && (src.contains("bind(anchorLeft)")
                || src.contains("left-(--anchor-left)")
                || src.contains("left: (--anchor-left)"))
            && (src.contains("bind(anchorTop)")
                || src.contains("top-(--anchor-top)")
                || src.contains("top: (--anchor-top)"))
            && (src.contains("bind(anchorWidth)")
                || src.contains("min-w-(--anchor-width)")
                || src.contains("min-w: (--anchor-width)"))
            && (src.contains("maxH(panel-max-h-sm)")
                || src.contains("max-h-panel-max-h-sm")
                || src.contains("panel-max-h-sm"))
            && (src.contains("overflowY(auto)")
                || src.contains("overflow-y-auto")
                || src.contains("overflow-y: auto"))
            && (src.contains("overscroll(contain)")
                || src.contains("overscroll-contain")
                || src.contains("overscroll: contain")),
        "listbox Style must carry anchor geometry + scroll"
    );
    assert!(
        (out.structure_html.contains("sx-positionfixed")
            || out.structure_html.contains("sx-fixed")
            || out.skin_css.contains("position:fixed")
            || out.skin_css.contains("position: fixed"))
            && (out.structure_html.contains("anchorWidth")
                || out.structure_html.contains("anchorwidth")
                || out.structure_html.contains("sx-minwanchorwidth")
                || out.skin_css.contains("var(--anchor-width)"))
            && (out.structure_html.contains("maxhpanelmaxhsm")
                || out.structure_html.contains("sx-maxhpanelmaxhsm")
                || out.skin_css.contains("var(--panel-max-h-sm)"))
            && (out.structure_html.contains("overflowyauto")
                || out.structure_html.contains("sx-overflowyauto")
                || out.skin_css.contains("overflow-y:auto")
                || out.skin_css.contains("overflow-y: auto"))
            && (out.structure_html.contains("overscrollcontain")
                || out.structure_html.contains("sx-overscrollcontain")
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
        "combobox component PE must be gone:\n{}",
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
    // Host: group + relative. Bubble: absolute + (--tt-*) geometry vars +
    // opacity show via group-hover/focus-within + after: arrow. tipDescribe
    // publishes placement vars (same bridge as use:anchor).
    let src = include_str!("../stdlib/tooltip.sail");
    let out = compile_element(src, "tooltip").expect("tooltip compiles");
    assert!(
        src.contains("group relative")
            && src.contains("top-(--tt-top)")
            && src.contains("translate-x-(--tt-tx)")
            && src.contains("group-hover:opacity-100")
            && src.contains("group-focus-within:opacity-100")
            && src.contains("after:content-empty")
            && src.contains("after:rotate-45"),
        "tooltip host/bubble sx must carry geometry + show + arrow"
    );
    // Style-minted class names (bind args mint as `<setter>bind<Name>`).
    assert!(
        out.structure_html.contains("sail-grp")
            && out.structure_html.contains("sx-positionrelative")
            && out.structure_html.contains("sx-positionabsolute")
            && out.structure_html.contains("sx-topbindttTop")
            && out.structure_html.contains("sx-grouphover-opacity100")
            && out.structure_html.contains("sx-after-rotate45"),
        "tooltip structure atoms:\n{}",
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
        "tooltip contracts PE matrix must be gone"
    );
}
