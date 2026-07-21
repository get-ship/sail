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
fn variants_field_is_parse_error() {
    // Dual path killed: scope-level `variants:` must not lower.
    let src = r#"
<scope>
  props: {
    tone: 'default' | 'primary' = 'default'
  }
  variants: {
    variant: {
      soft: 'bg-tone-bg text-tone-fg border-tone-border',
      solid: 'bg-tone text-tone-contrast border-tone',
      outline: 'bg-transparent text-tone-fg border-tone-border',
    }
  }
</scope>
<row inline nowrap align="center" gap="1" sx="rounded-sm border font-semibold">
  <slot></slot>
</row>
"#;
    let err = compile_element(src, "probe").expect_err("variants: must be rejected");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("variants:") && (msg.contains("CVA-in-props") || msg.contains("props:")),
        "must reject variants: and name CVA-in-props: {msg}"
    );
}

#[test]
fn cva_in_props_lowers_same_sx_pipeline_element_mode() {
    // Preferred matrix form: prop map key→sx; reflect + types; same lower as host sx=.
    let src = r#"
<scope>
  props: {
    tone: 'default' | 'primary' = 'default'
    variant: {
      soft: 'bg-tone-bg text-tone-fg border-tone-border',
      solid: 'bg-tone text-tone-contrast border-tone hover:opacity-90',
      outline: 'bg-transparent text-tone-fg border-tone-border',
    }
    size: {
      sm: 'text-xs px-2',
      md: 'text-sm px-2_5',
    }
  }
</scope>
<span sx="rounded-full border font-medium">
  <slot></slot>
</span>
"#;
    let out = compile_element(src, "probe").expect("cva-in-props compiles");
    let attrs = out.manifest["attributes"].as_array().expect("attrs");
    let variant = attrs.iter().find(|a| a["name"] == "variant").expect("variant attr");
    assert_eq!(variant["default"], "soft", "first map key is default: {variant}");
    let values = variant["values"].as_array().expect("values");
    assert!(
        values.iter().any(|v| v == "soft") && values.iter().any(|v| v == "solid"),
        "union of keys: {variant}"
    );
    // Attribute-keyed rules from the same sx lower (not hand property soup).
    assert!(
        out.skin_css.contains("variant='soft'")
            && out.skin_css.contains("background-color:var(--tone-bg)")
            && out.skin_css.contains("color:var(--tone-fg)"),
        "soft axis via sx atoms: {}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("variant='solid'")
            && out.skin_css.contains("background-color:var(--tone)")
            && out.skin_css.contains("color:var(--tone-contrast)"),
        "solid axis via sx atoms: {}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("size='sm'") && out.skin_css.contains("font-size:var(--text-xs)"),
        "size axis via sx atoms: {}",
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
        out.structure_html.contains("sx-bgtonebg")
            && out.structure_html.contains("sx-texttonefg")
            && out.structure_html.contains("sx-bordertoneborder"),
        "soft default paint must be host sx (attr-absent base):\n{}",
        out.structure_html
    );
    assert!(
        out.structure_html.contains("sx-fontbody"),
        "typeface on host sx:\n{}",
        out.structure_html
    );
    assert!(
        !out.skin_css.contains("background: var(--tone-bg)"),
        "soft base author paint deleted:\n{}",
        out.skin_css
    );
    // solid/outline/size via CVA-in-props (same sx pipeline)
    assert!(
        out.skin_css.contains("variant='solid'")
            && out.skin_css.contains("background-color:var(--tone)")
            && out.skin_css.contains("variant='outline'")
            && out.skin_css.contains("size='sm'"),
        "solid/outline/size via CVA-in-props:\n{}",
        out.skin_css
    );
    assert!(
        !out.skin_css.contains("--badge-h") && !out.skin_css.contains("--badge-fs"),
        "badge residual geometry channels must be gone:\n{}",
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
            && src.contains("avatar-media"),
        "avatar must layout-center + avatar-media child pack"
    );
    assert!(
        out.structure_html.contains("sx-trackingwide")
            && (out.structure_html.contains("sx-shadowinsetring")
                || out.skin_css.contains("box-shadow:var(--shadow-inset-ring)")
                || out.skin_css.contains("box-shadow: var(--shadow-inset-ring)")),
        "tracking-wide + shadow-inset-ring must land on host:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.structure_html.contains("sx-size7")
            && out.structure_html.contains("sx-roundedfull")
            && out.structure_html.contains("sx-fontbody"),
        "attr-absent PE size=md + shape round + type on host sx:\n{}",
        out.structure_html
    );
    // PE gradient via TW pack on host + default tone CVA.
    assert!(
        src.contains("bg-gradient-to-br")
            && src.contains("from-primary")
            && src.contains("to-accent")
            && (out.structure_html.contains("sx-bggradienttobr")
                || out.structure_html.contains("bggradienttobr")
                || out.skin_css.contains("linear-gradient(to bottom right")),
        "bg-gradient-to-br from-primary to-accent must land on host:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css.contains("linear-gradient(to bottom right")
            && (out.skin_css.contains("--sx-gradient-from:var(--primary)")
                || out.skin_css.contains("--sx-gradient-from: var(--primary)"))
            && (out.skin_css.contains("--sx-gradient-to:var(--accent)")
                || out.skin_css.contains("--sx-gradient-to: var(--accent)")),
        "gradient direction + from/to color stops must lower:\n{}",
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
    // Branded tones clear the host-base background-image then solid-fill.
    assert!(
        out.skin_css.contains("tone='primary'")
            && (out.skin_css.contains("background-image:none")
                || out.skin_css.contains("background-image: none"))
            && out.skin_css.contains("background-color:var(--tone)")
            && out.skin_css.contains("size='xs'")
            && out.skin_css.contains("shape='square'"),
        "branded tone bg-none + solid + size/shape via CVA-in-props:\n{}",
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
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
    // Structure: stretch host + grow copy + channel-read icon/title.
    // Channel WRITES + size gap + inline flex + .icon svg anatomy via
    // scope `channels:` → element-skins. empty:hidden + forced-colors on sx.
    // Zero contracts PE.
    let src = include_str!("../stdlib/empty-state.sail");
    let out = compile_element(src, "empty-state").expect("empty-state compiles");
    assert!(
        src.contains("align=\"stretch\"")
            && src.contains("grow-1")
            && src.contains("self-stretch")
            && src.contains("self-center")
            && src.contains("bg-(--es-icon-bg)")
            && src.contains("text-(--es-icon-fg)")
            && src.contains("shadow-(--es-icon-ring)")
            && src.contains("font-(--es-title-weight)")
            && src.contains("text-(--es-title-color)")
            && src.contains("text-(--es-title-size)")
            && src.contains("text-(--es-desc-size)")
            && src.contains("max-w-(--es-desc-max)")
            && src.contains("size-(--es-tile)")
            && src.contains("channels:")
            && src.contains("empty:hidden")
            && src.contains("forced-colors:"),
        "empty-state structure must stretch/grow + read --es-* + channels: + empty/forced sx"
    );
    assert!(
        out.skin_css.contains("font-size:var(--es-title-size)")
            || out.skin_css.contains("font-size: var(--es-title-size)"),
        "title text-(--es-title-size) must bind font-size (not color):\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("font-size:var(--es-desc-size)")
            || out.skin_css.contains("font-size: var(--es-desc-size)"),
        "desc text-(--es-desc-size) must bind font-size (not color):\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("max-width:var(--es-desc-max)")
            || out.skin_css.contains("max-width: var(--es-desc-max)"),
        "desc max-w channel must land:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("box-shadow:var(--es-icon-ring)")
            || out.skin_css.contains("box-shadow: var(--es-icon-ring)"),
        "icon shadow-(--es-icon-ring) must land:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("background-color:var(--es-icon-bg)")
            || out.skin_css.contains("background-color: var(--es-icon-bg)"),
        "icon bg channel must land:\n{}",
        out.skin_css
    );
    // Channel WRITES + layout axes + svg anatomy lower into skin CSS.
    assert!(
        out.skin_css.contains("--es-icon:var(--control-h-2xs)")
            && out.skin_css.contains("--es-tile:var(--space-12)")
            && out.skin_css.contains("--es-icon-bg:var(--muted)")
            && out.skin_css.contains("--es-icon-ring:none")
            && out.skin_css.contains("--es-title-weight:var(--font-weight-semibold)")
            && out.skin_css.contains("size='sm'")
            && out.skin_css.contains("--es-icon:var(--icon-size-md)")
            && out.skin_css.contains("gap:var(--space-2)")
            && out.skin_css.contains("gap:var(--space-4)")
            && out.skin_css.contains("tone='primary'")
            && out.skin_css.contains("--es-icon-bg:var(--tone-bg)")
            && out.skin_css.contains("variant='inline'")
            && out.skin_css.contains("--es-tile:var(--es-icon)")
            && out.skin_css.contains("flex-direction:row")
            && out.skin_css.contains("align-items:center")
            && out.skin_css.contains("variant='connecting'")
            && out.skin_css.contains("--es-title-color:var(--muted-foreground)")
            && (out.skin_css.contains(".icon svg") || out.skin_css.contains(" .icon svg")),
        "channels: must lower attr-keyed --es-* writes + layout axes + svg anatomy:\n{}",
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
    ));
    // Zero empty-state PE in contracts (comment may remain).
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
    // Was contracts PE: selected border-bottom tone + inset focus ring.
    // Now host sx: selected:border-b-tone + focus outline ring-width/tone-ring/ring-inset.
    let src = include_str!("../stdlib/tab.sail");
    let out = compile_element(src, "tab").expect("tab compiles");
    assert!(
        src.contains("selected:border-b-tone")
            && src.contains("focus:outline-solid")
            && src.contains("focus:outline-ring-width")
            && src.contains("focus:outline-tone-ring")
            && src.contains("focus:outline-offset-ring-inset"),
        "tab host sx must carry selected underline + inset focus ring"
    );
    assert!(
        out.skin_css.contains("border-bottom-color:var(--tone)")
            || out.skin_css.contains("border-bottom-color: var(--tone)"),
        "selected underline tone must lower into skin:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("outline-width:var(--ring-width)")
            || out.skin_css.contains("outline-width: var(--ring-width)"),
        "focus outline width must use --ring-width:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("outline-color:var(--tone-ring)")
            || out.skin_css.contains("outline-color: var(--tone-ring)"),
        "focus outline color must use --tone-ring:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("calc(-1 * var(--ring-width))"),
        "focus outline-offset must be inset by ring-width:\n{}",
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
    ));
    assert!(
        !contracts.contains(".sail-tab[aria-selected='true']")
            && !contracts.contains(".sail-tab-list[variant='pill'] .sail-tab")
            && !contracts.contains(".sail-tab-list[variant='enclosed'] .sail-tab"),
        "tab selected / pill/enclosed parent→child PE must be gone from contracts"
    );
    // Bare focus PE gone.
    assert!(
        !contracts.contains(".sail-tab:focus-visible"),
        "tab focus-visible PE must be gone from contracts"
    );
    // Pill/enclosed chrome lives on tab variant CVA (stamped by use:tabs).
    let tab_src = include_str!("../stdlib/tab.sail");
    assert!(
        tab_src.contains("pill:")
            && tab_src.contains("enclosed:")
            && tab_src.contains("selected:bg-background"),
        "tab.sail must CVA pill/enclosed selected chrome"
    );
}

#[test]
fn toggle_pressed_focus_stack_on_host_sx() {
    // Joined-outline stack (was contracts PE) lives on host sx:
    // relative + pressed:z-1 + focus:z-2 + focus outline utilities.
    let out = compile_element(include_str!("../stdlib/toggle.sail"), "toggle").expect("toggle");
    assert!(
        out.structure_html.contains("sx-relative")
            && out.structure_html.contains("sx-pressed-z1")
            && out.structure_html.contains("sx-focus-z2")
            && out.structure_html.contains("sx-focus-outlinesolid")
            && out.structure_html.contains("sx-focus-outline2")
            && out.structure_html.contains("sx-focus-outlinering"),
        "stack + focus outline atoms on host structure:\n{}",
        out.structure_html
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
    ));
    assert!(
        !contracts.contains(".sail-toggle[aria-pressed='true']")
            && !contracts.contains(".sail-toggle:focus-visible")
            && !contracts.contains(".sail-toggle-group > .sail-toggle"),
        "toggle pressed/focus + toggle-group join PE must be gone from contracts"
    );
    // Join radii on first:/last: host sx + variant CVA.
    assert!(
        out.structure_html.contains("sx-first-")
            || out.skin_css.contains(":first-child")
            || include_str!("../stdlib/toggle.sail").contains("first:"),
        "toggle must express join edges via first:/last:"
    );
}

#[test]
fn switch_host_track_paint_on_host_sx() {
    // Host track + before: thumb + channels: --sw-* + forced-colors/motion-reduce
    // on host sx / generated skin. PE checked/focus is primary on host; branded
    // tones CVA rebind to --tone + thumb on-fg. checked: → :checked.
    let src = include_str!("../stdlib/switch.sail");
    let out = compile_element(src, "switch").expect("switch compiles");
    assert!(
        src.contains("appearance-none")
            && src.contains("bg-none")
            && src.contains("bg-muted")
            && src.contains("border-border-strong")
            && src.contains("rounded-full")
            && src.contains("w-(--sw-w)")
            && src.contains("h-(--sw-h)")
            && src.contains("before:content-empty")
            && src.contains("before:size-(--sw-thumb)")
            && src.contains("checked:before:left-auto")
            && src.contains("checked:before:right-px")
            && src.contains("hover:bg-hover-bg")
            && src.contains("checked:bg-primary")
            && src.contains("checked:border-selected-border")
            && src.contains("checked:shadow-xs")
            && src.contains("focus:outline-primary")
            && src.contains("focus:outline-offset-ring-offset")
            && src.contains("disabled:saturate-60")
            && src.contains("disabled:cursor-not-allowed")
            && src.contains("checked:bg-tone")
            && src.contains("focus:outline-tone")
            && src.contains("channels:")
            && src.contains("'sw-w'"),
        "switch host sx PE primary + before: thumb + channels + CVA branded tone atoms"
    );
    assert!(
        out.structure_html.contains("sx-appearancenone")
            || out.skin_css.contains("appearance:none")
            || out.skin_css.contains("appearance: none"),
        "appearance-none must land:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css.contains("background-image:none")
            || out.skin_css.contains("background-image: none")
            || out.structure_html.contains("sx-bgnone"),
        "bg-none must clear checkbox-contract mark SVG:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css.contains("background-color:var(--muted)")
            || out.skin_css.contains("background-color: var(--muted)")
            || out.structure_html.contains("sx-bgmuted"),
        "bg-muted off-track paint:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css.contains("width:var(--sw-w)")
            || out.skin_css.contains("width: var(--sw-w)")
            || out.structure_html.contains("sx-wsww")
            || out.structure_html.contains("sx-w--sw-w")
            || out.structure_html.contains("sw-w"),
        "size channel reads w-(--sw-w):\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    // checked: → :is(:checked, [aria-checked='true']) for native host
    assert!(
        out.skin_css.contains(":checked")
            && (out.skin_css.contains("background-color:var(--primary)")
                || out.skin_css.contains("background-color: var(--primary)")),
        "host PE checked:bg-primary must lower against :checked:\n{}",
        out.skin_css
    );
    // Branded CVA rebinds checked fill to global --tone (does not clobber via bare PE).
    assert!(
        out.skin_css.contains("tone='primary'")
            && (out.skin_css.contains("background-color:var(--tone)")
                || out.skin_css.contains("background-color: var(--tone)")),
        "CVA branded tone checked:bg-tone must land:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("selected-border")
            || out.skin_css.contains("var(--selected-border)"),
        "checked:border-selected-border:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("saturate(0.6)")
            || out.structure_html.contains("saturate60")
            || out.structure_html.contains("saturate-60"),
        "disabled:saturate-60:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    // channels: --sw-* land in generated skin (not contracts).
    assert!(
        out.skin_css.contains("--sw-w:var(--control-h-sm)")
            || out.skin_css.contains("--sw-w: var(--control-h-sm)"),
        "channels default --sw-w must land in skin:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("--sw-thumb:var(--space-3)")
            || out.skin_css.contains("--sw-thumb: var(--space-3)")
            || out.skin_css.contains("size='sm'"),
        "channels size rekeys must land:\n{}",
        out.skin_css
    );
    // before: thumb anatomy in skin.
    assert!(
        out.skin_css.contains("::before")
            && (out.skin_css.contains("content:''") || out.skin_css.contains("content: ''")),
        "before: thumb must lower:\n{}",
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
    ));
    // Extract switch PE section (between its banner and the next element banner).
    let sw_start = contracts
        .find("/* switch")
        .expect("switch PE banner present");
    let sw_rest = &contracts[sw_start..];
    let sw_end = sw_rest
        .find("/* prose")
        .or_else(|| sw_rest.find("\n/* "))
        .unwrap_or(sw_rest.len());
    let sw_pe = &sw_rest[..sw_end];
    // All switch PE gone (channels + thumb + forced-colors → generated skin).
    assert!(
        !sw_pe.contains("--sw-w:")
            && !sw_pe.contains("::before")
            && !sw_pe.contains("--tone-on-fg:")
            && !sw_pe.contains("forced-colors")
            && !sw_pe.contains("appearance:")
            && !sw_pe.contains("background: var(--muted)"),
        "switch PE must be fully gone (channels/thumb/forced-colors on skin):\n{sw_pe}"
    );
}

#[test]
fn toast_start_stripe_on_host_sx() {
    // PE was border-inline-start: 3px solid var(--tone-fg). Now border-s-3 border-s-tone-fg.
    let out = compile_element(include_str!("../stdlib/toast.sail"), "toast").expect("toast");
    assert!(
        out.structure_html.contains("sx-borders3") || out.structure_html.contains("border-s-3")
            || out.skin_css.contains("border-inline-start-width:3px")
            || out.skin_css.contains("border-inline-start-width: 3px"),
        "border-s-3 must lower to 3px start width:\nstructure={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css.contains("border-inline-start-color:var(--tone-fg)")
            || out.skin_css.contains("border-inline-start-color: var(--tone-fg)"),
        "border-s-tone-fg must bind start color to --tone-fg:\n{}",
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
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
            && src.contains("ms-auto")
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
            || out.skin_css.contains("backdrop-filter:var(--card-backdrop-filter)")
            || out.skin_css.contains("backdrop-filter: var(--card-backdrop-filter)"),
        "backdrop token must land:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css.contains("-webkit-backdrop-filter:var(--card-backdrop-filter)")
            || out.skin_css.contains("-webkit-backdrop-filter: var(--card-backdrop-filter)")
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
            && (out.skin_css.contains("border-inline-start-width:3px")
                || out.skin_css.contains("border-inline-start-width: 3px"))
            && (out.skin_css.contains("border-inline-start-color:var(--border-strong)")
                || out.skin_css.contains("border-inline-start-color: var(--border-strong)"))
            && (out.skin_css.contains("border-inline-start-color:var(--tone-fg)")
                || out.skin_css.contains("border-inline-start-color: var(--tone-fg)"))
            && (out.skin_css.contains("padding-inline-start:var(--space-4)")
                || out.skin_css.contains("padding-inline-start: var(--space-4)")),
        "status-rail CVA must lower border-s paint + ps pad-adjust:\n{}",
        out.skin_css
    );

    // interactive lift + forced-colors focus
    assert!(
        src.contains("hover:-translate-y-px")
            && src.contains("active:translate-y-0")
            && src.contains("motion-reduce:translate-y-0")
            && src.contains("active:duration-instant")
            && src.contains("forced-colors:outline-CanvasText"),
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

    // branded tone border via compound (not PE)
    assert!(
        out.skin_css.contains("tone='primary'")
            && (out.skin_css.contains("border-color:var(--tone-border)")
                || out.skin_css.contains("border-color: var(--tone-border)")),
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
            && out.skin_css.contains("linear-gradient(to right")
            && (out.skin_css.contains("--sx-gradient-from:var(--primary)")
                || out.skin_css.contains("--sx-gradient-from: var(--primary)"))
            && (out.skin_css.contains("--sx-gradient-to:var(--accent)")
                || out.skin_css.contains("--sx-gradient-to: var(--accent)"))
            && (out.skin_css.contains("height:var(--space-half)")
                || out.skin_css.contains("height: var(--space-half)"))
            && out.skin_css.contains(":hover")
            && (out.skin_css.contains("opacity:1") || out.skin_css.contains("opacity: 1")
                || out.skin_css.contains("opacity:1.") || out.skin_css.contains("opacity:0")),
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
            && (out.structure_html.contains("sx-empty-hidden")
                || out.skin_css.contains(":empty"))
            && (out.structure_html.contains("sx-fontheading")
                || out.skin_css.contains("font-family:var(--heading-font)")
                || out.skin_css.contains("font-family: var(--heading-font)")),
        "header/footer chrome must lower into structure/skin:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );

    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
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
    // Layout + paint owned by navbar.sail (layout keywords + sx/CVA/compound/
    // channels). No contracts PE matrix for navbar / nav-* regions / rail chrome.
    let src = include_str!("../stdlib/navbar.sail");
    let out = compile_element(src, "navbar").expect("navbar compiles");
    assert!(
        src.contains("<row")
            && src.contains("nowrap")
            && src.contains("gap=\"4\"")
            && src.contains("dir-column")
            && src.contains("h-rail-fill")
            && src.contains("top-(--rail-offset)")
            && src.contains("channels:")
            && src.contains("rail-offset")
            && src.contains("bg-background")
            && src.contains("text-foreground")
            && src.contains("min-h-14")
            && src.contains("py-2")
            && src.contains("px-4")
            && src.contains("sticky top-0 z-(--z-sticky)")
            && src.contains("border-b border-border")
            && src.contains("top-auto bottom-0")
            && src.contains("border-b-0 border-t border-border")
            && src.contains("border-b-0 border-e border-border")
            && src.contains("min-h-0 py-5 px-4")
            && src.contains("min-w-(--panel-rail-w)")
            && src.contains("sx=\"contents\"")
            && src.contains("grow-(--nav-center-grow)")
            && src.contains("ms-(--nav-actions-ms)")
            && src.contains("mt-(--nav-actions-mt)"),
        "navbar structure must own layout keywords + rail sticky atoms + channels"
    );
    assert!(
        out.structure_html.contains("sail-row")
            && out.structure_html.contains("sail-nowrap")
            && (out.structure_html.contains("sail-g-4") || out.structure_html.contains("sail-g-"))
            && out.structure_html.contains("sx-bgbackground")
            && out.structure_html.contains("sx-textforeground")
            && out.structure_html.contains("sx-minh14")
            && out.structure_html.contains("sx-contents")
            && out.structure_html.contains("sail-nav-brand")
            && out.structure_html.contains("sail-nav-links")
            && out.structure_html.contains("sail-nav-center")
            && out.structure_html.contains("sail-nav-actions"),
        "host row + region structure must lower:\n{}",
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
            && out.skin_css.contains("placement='bottom'][sticky]")
            && out.skin_css.contains("bottom:0")
            && out.skin_css.contains("placement='bottom'][bordered]")
            && out.skin_css.contains("border-top-width:1px")
            && out.skin_css.contains("placement='rail'][bordered]")
            && out.skin_css.contains("border-inline-end-width:1px")
            && out.skin_css.contains("--rail-offset")
            && out.skin_css.contains("calc(100dvh - var(--rail-offset, 0px))")
            && out.skin_css.contains("placement='rail'][sticky]")
            && out.skin_css.contains("overflow-y:auto"),
        "placement/sticky/bordered/rail geometry via CVA+compound+channels:\n{}",
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
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
    // Zero contracts PE. Structure + channels: + empty:hidden + group-state-*
    // details gates + forced-colors: cover the former hand PE.
    let src = include_str!("../stdlib/error-state.sail");
    let out = compile_element(src, "error-state").expect("error-state compiles");

    assert!(
        src.contains("align=\"stretch\"")
            && src.contains("grow-1")
            && src.contains("self-stretch")
            && src.contains("self-center")
            && src.contains("bg-(--es-icon-bg)")
            && src.contains("outline-(--es-icon-outline)")
            && src.contains("outline-1")
            && src.contains("outline-solid")
            && src.contains("outline-offset-ring-inset")
            && src.contains("text-md")
            && src.contains("text-sm")
            && src.contains("max-w-panel-ch-w-md")
            && src.contains("max-w-panel-ch-w-lg")
            && src.contains("size-(--es-tile)")
            && src.contains("size-(--es-icon)")
            && src.contains("channels:")
            && src.contains("empty:hidden")
            && src.contains("group-state-has-details:inline-block")
            && src.contains("group-state-open:block")
            && src.contains("forced-colors:border-CanvasText"),
        "error-state structure must stretch/grow + channels + empty/group-state + forced-colors"
    );

    assert!(
        out.skin_css.contains("font-size:var(--text-md)")
            && out.skin_css.contains("font-size:var(--text-sm)")
            && out.skin_css.contains("max-width:var(--panel-ch-w-md)")
            && out.skin_css.contains("max-width:var(--panel-ch-w-lg)")
            && (out.skin_css.contains("outline-color:var(--es-icon-outline)")
                || out.skin_css.contains("outline-color: var(--es-icon-outline)"))
            && out.skin_css.contains("outline-offset:calc(-1 * var(--ring-width))")
            && (out.skin_css.contains("background-color:var(--es-icon-bg)")
                || out.skin_css.contains("background-color: var(--es-icon-bg)")),
        "child paint + icon channels must lower into element-skins:\n{}",
        out.skin_css
    );
    assert!(
        out.structure_html.contains("use:recoverySurface")
            && out.structure_html.contains(" icon ")
            && out.structure_html.contains("class=\"title ")
            && out.structure_html.contains("class=\"desc ")
            && out.structure_html.contains("class=\"details ")
            && out.structure_html.contains("class=\"details-toggle ")
            && (out.structure_html.contains("sail-grp") || src.contains(" group")),
        "structure anatomy classes + recoverySurface + group preserved:\n{}",
        out.structure_html
    );
    // Channel WRITES + layout axes + [hidden] anatomy.
    assert!(
        out.skin_css.contains("--es-icon:var(--control-h-2xs)")
            && out.skin_css.contains("--es-tile:var(--control-h-xl)")
            && out.skin_css.contains("--es-icon-bg:var(--tone-destructive-bg)")
            && out.skin_css.contains("--es-icon-outline:var(--tone-destructive-ring)")
            && out.skin_css.contains("size='sm'")
            && out.skin_css.contains("gap:var(--space-2)")
            && out.skin_css.contains("size='lg'")
            && out.skin_css.contains("gap:var(--space-4)")
            && out.skin_css.contains("variant='inline'")
            && out.skin_css.contains("flex-direction:row")
            && out.skin_css.contains("--es-tile:var(--es-icon)")
            && (out.skin_css.contains("[hidden]{display:none}")
                || out.skin_css.contains("[hidden]{display:none;}"))
            && (out.skin_css.contains("data-state~='has-details'")
                || out.skin_css.contains("data-state~=\"has-details\"")
                || out.skin_css.contains("groupstatehasdetails")
                || out.skin_css.contains("group-state")),
        "channels/layout/hidden/group-state must lower into skin_css:\n{}",
        out.skin_css
    );

    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
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
    // Root stays <label> for native semantics; nested <row> owns flex/gap.
    // Disabled is compound. Zero contracts PE for radio.
    let src = include_str!("../stdlib/radio.sail");
    let out = compile_element(src, "radio").expect("radio compiles");
    assert!(
        src.contains("<label")
            && src.contains("<row inline nowrap align=\"center\" gap=\"2\">")
            && src.contains("cursor-not-allowed opacity-50"),
        "radio must nest row flex + compound disabled"
    );
    assert!(
        out.structure_html.contains("sail-row")
            || out.structure_html.contains("sail-row-i")
            || out.skin_css.contains("inline-flex")
            || out.skin_css.contains("sail-row"),
        "nested row must lower layout:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css.contains("[disabled]")
            && (out.skin_css.contains("opacity:0.5") || out.skin_css.contains("opacity: 0.5")),
        "disabled compound on host:\n{}",
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
    ));
    assert!(
        !contracts.contains("label.sail-radio")
            && !contracts.contains(".sail-radio {\n  display: inline-flex"),
        "radio label flex PE must be gone from contracts"
    );
}

#[test]
fn toggle_group_channels_and_join_on_skins_not_pe() {
    // Size --tg-* via channels:; segmented host CVA; join/ghost/segmented cells
    // on toggle first:/last: + stamped variant CVA. Zero hand PE.
    let src = include_str!("../stdlib/toggle-group.sail");
    let out = compile_element(src, "toggle-group").expect("toggle-group compiles");
    assert!(
        src.contains("segmented: 'p-1 bg-muted border border-border rounded-md'")
            && src.contains("<row inline gap=\"0\"")
            && src.contains("channels:")
            && src.contains("'tg-h'"),
        "segmented host chrome on CVA; host is inline row; channels for --tg-*"
    );
    assert!(
        out.skin_css.contains("variant='segmented'")
            && (out.skin_css.contains("background-color:var(--muted)")
                || out.skin_css.contains("background-color: var(--muted)"))
            && out.skin_css.contains("--tg-h:var(--control-h-md)"),
        "segmented CVA + --tg-* channels on skins:\n{}",
        out.skin_css
    );
    // Layout owns vertical axis.
    assert!(
        out.skin_css.contains("orientation='vertical'")
            || out.skin_css.contains("[orientation='vertical']"),
        "layout orientation flip should emit for row host:\n{}",
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
    ));
    assert!(
        !contracts.contains("--tg-h:")
            && !contracts.contains(".sail-toggle-group > .sail-toggle")
            && !contracts.contains(".sail-toggle-group[orientation")
            && !contracts.contains(".sail-toggle-group[variant='ghost']"),
        "toggle-group hand PE must be gone from contracts"
    );
    let toggle_src = include_str!("../stdlib/toggle.sail");
    assert!(
        toggle_src.contains("first:")
            && toggle_src.contains("last:")
            && toggle_src.contains("outline:")
            && toggle_src.contains("ghost:"),
        "toggle.sail must CVA join/ghost with first:/last:"
    );
}

#[test]
fn alert_tone_stripe_before_on_host_sx() {
    // Host <row nowrap align=start gap=3> owns ico|content|close (layout
    // keywords — free grid-template PE forbidden). Stripe/empty/dismiss +
    // forced-colors live on host/child sx + CVA. Zero alert PE in contracts.
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
            && (out.structure_html.contains("sail-nowrap") || out.structure_html.contains("nowrap"))
            && out.structure_html.contains("sail-g-3"),
        "host row/nowrap/gap must lower into structure:\n{}",
        out.structure_html
    );
    assert!(
        !out.structure_html.contains("grid-template-columns: auto 1fr auto")
            && !out.skin_css.contains("grid-template-columns: auto 1fr auto")
            && !out.skin_css.contains("grid-template-columns:auto 1fr auto"),
        "must not emit free grid-template PE in structure/skin:\n{}\n{}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        src.contains("before:content-empty")
            && src.contains("before:bg-muted-foreground")
            && src.contains("empty:hidden")
            && src.contains("group-dismissible:")
            && src.contains("sx=\"group ")
            && src.contains("forced-colors:border-CanvasText")
            && src.contains("forced-colors:before:bg-CanvasText"),
        "host/child sx must carry stripe PE default, empty gates, group-dismissible, forced-colors"
    );
    assert!(
        src.contains("default: 'before:bg-muted-foreground'")
            && src.contains("info: 'before:bg-tone'"),
        "CVA tone first-key muted stripe; branded before:bg-tone"
    );
    assert!(
        out.structure_html.contains("before:")
            || out.structure_html.contains("sx-before-")
            || out.skin_css.contains("::before"),
        "before: stripe atoms must lower into structure/skin:\n{}\n{}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css.contains("content:''") || out.skin_css.contains("content: ''"),
        "content-empty must emit content:'':\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("var(--muted-foreground)"),
        "PE default before:bg-muted-foreground must bind muted:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("tone='info'")
            && out.skin_css.contains("var(--tone)"),
        "branded CVA before:bg-tone must lower:\n{}",
        out.skin_css
    );
    assert!(
        out.structure_html.contains("empty:")
            || out.structure_html.contains("sx-empty-")
            || out.skin_css.contains(":empty"),
        "empty:hidden must lower:\n{}\n{}",
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
        out.skin_css.contains("forced-colors")
            && out.skin_css.contains("CanvasText"),
        "forced-colors CanvasText must lower into skin:\n{}",
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
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
    assert!(
        out.structure_html.contains("sx-fixed")
            && out.structure_html.contains("sx-zztoast")
            && out.structure_html.contains("sx-p4")
            && out.structure_html.contains("sx-pointereventsnone")
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
            && out.skin_css.contains("bottom:0")
            && out.skin_css.contains("flex-direction:column-reverse"),
        "position axis + column-reverse via CVA-in-props:\n{}",
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
    ));
    let tr = contracts
        .split("/* toast-region")
        .nth(1)
        .and_then(|s| s.split("/* ").next())
        .expect("toast-region contracts section");
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
}

#[test]
fn theme_picker_host_paint_on_structure_sx() {
    // Layout keywords + structure sx carry trigger/panel/option/swatch paint
    // (including .tc-* mini-card pieces on real nodes → element-skins).
    // KEEP: panel anchor geometry + raw width, [inline] matrices, forced-colors.
    let src = include_str!("../stdlib/theme-picker.sail");
    let out = compile_element(src, "theme-picker").expect("theme-picker compiles");
    assert!(
        src.contains("size-control-h-sm")
            && src.contains("bg-popover")
            && src.contains("selected:border-primary")
            && src.contains("h-24")
            && src.contains("isolate")
            && src.contains("class=\"tc-overlay\"")
            && src.contains("class=\"tc-card\"")
            && src.contains("class=\"tc-cta\"")
            && src.contains("cols=\"2\"")
            && src.contains("<row inline")
            && src.contains("shadow-inset-ring")
            && src.contains("text-(--icon-size-3xs)"),
        "theme-picker structure must carry layout + child paint atoms + tc-* anatomy"
    );
    assert!(
        out.structure_html.contains("sx-sizecontrolhsm")
            && out.structure_html.contains("sx-bgpopover")
            && out.structure_html.contains("sx-selected-borderprimary")
            && out.structure_html.contains("sx-h24")
            && out.structure_html.contains("sx-isolate")
            && out.structure_html.contains("sx-shadowinsetring")
            && (out.structure_html.contains("sail-grid") || out.structure_html.contains("sail-gc-2"))
            && (out.structure_html.contains("sail-row-i") || out.skin_css.contains("inline-flex")),
        "child paint + layout classes on structure:\n{}",
        out.structure_html
    );
    // .tc-* paint is structure sx classes, not parent→child PE.
    assert!(
        out.structure_html.contains("tc-overlay")
            && out.structure_html.contains("tc-card")
            && out.structure_html.contains("tc-cta")
            && out.structure_html.contains("sx-bgprimary")
            && out.structure_html.contains("sx-textprimaryforeground"),
        "swatch mini-card paint must be on real nodes:\n{}",
        out.structure_html
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
    ));
    let tp_start = contracts
        .find("/* ── theme-picker")
        .or_else(|| contracts.find("/* theme-picker"))
        .expect("theme-picker PE banner present");
    // Theme-picker is the last PE section — take through EOF (inner `/* … */`
    // comments must not truncate the slice).
    let tp_pe = &contracts[tp_start..];
    // Deleted paint piles (root flex, trigger flex/svg, option flex, swatch
    // color-mix ring, entire .tc-* anatomy, popover-open grid).
    // Note: [inline] matrices still mention .sail-theme-option / .sail-theme-trigger
    // as parent→child show/hide — those are KEEP, not the deleted paint piles.
    assert!(
        !tp_pe.contains(".sail-theme-picker { display: inline-flex")
            && !tp_pe.contains(".sail-theme-trigger svg")
            && !tp_pe.contains(".sail-theme-swatch")
            && !tp_pe.contains(".tc-overlay")
            && !tp_pe.contains(".tc-card")
            && !tp_pe.contains(".tc-cta")
            && !tp_pe.contains(".tc-chip")
            && !tp_pe.contains(":popover-open")
            && !tp_pe.contains("color-mix(in oklch, currentColor 14%")
            && !tp_pe.contains("display: inline-flex")
            && !tp_pe.contains("flex-direction: column"),
        "moved theme-picker paint piles must be gone from contracts:\n{tp_pe}"
    );
    // KEEP unexpressibles: anchor geometry, [inline] matrices, forced-colors.
    assert!(
        tp_pe.contains("left: var(--anchor-left)")
            && tp_pe.contains("top: var(--anchor-top)")
            && tp_pe.contains("width: 380px")
            && tp_pe.contains("[inline] .sail-theme-panel")
            && tp_pe.contains("grid-template-columns: repeat(4, 1fr)")
            && tp_pe.contains("[inline] .sail-theme-option")
            && tp_pe.contains("forced-colors: active"),
        "KEEP anchor geometry, [inline] matrices, forced-colors:\n{tp_pe}"
    );
}

#[test]
fn prose_host_paint_on_sx_descendants_generated() {
    // Host block + body typeface/color/leading → host sx.
    // Descendant recipe: freeform HTML → compiler prose map → element-skins
    // (not hand contracts PE).
    let src = include_str!("../stdlib/prose.sail");
    let out = compile_element(src, "prose").expect("prose compiles");
    assert!(
        src.contains("block")
            && src.contains("font-body")
            && src.contains("text-foreground")
            && src.contains("leading-normal-plus"),
        "prose host sx must carry block + body type/color/leading"
    );
    assert!(
        out.structure_html.contains("sx-block")
            && out.structure_html.contains("sx-fontbody")
            && out.structure_html.contains("sx-textforeground")
            && out.structure_html.contains("sx-leadingnormalplus"),
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
        "/../../../packages/sail/tokens/src/contracts.css"
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
            && src.contains("sticky top-0 z-(--z-sticky)")
            && src.contains("bg-background")
            && src.contains("pill:")
            && src.contains("bg-muted"),
        "tab-list must declare sticky compound + opaque bg + pill CVA on host"
    );
    assert!(
        out.structure_html.contains("sx-bgbackground")
            && out.structure_html.contains("sx-borderb"),
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
            && (out.skin_css.contains("background-color:var(--muted)")
                || out.skin_css.contains("background-color: var(--muted)"))
            && (out.skin_css.contains("border-radius:var(--radius-md)")
                || out.skin_css.contains("border-radius: var(--radius-md)")),
        "pill variant CVA must lower on tab-list:\n{}",
        out.skin_css
    );
    // sticky + variant no longer tabs props; size channels live on tabs.
    let tabs_src = include_str!("../stdlib/tabs.sail");
    let tabs_out = compile_element(tabs_src, "tabs").expect("tabs compiles");
    assert!(
        !tabs_src.contains("sticky = false")
            && !tabs_src.contains("sticky=false")
            && !tabs_src.contains("variant:")
            && tabs_src.contains("channels:")
            && tabs_src.contains("'tab-py'"),
        "sticky/variant not on <tabs>; channels: for --tab-*:\n{tabs_src}"
    );
    assert!(
        tabs_out.skin_css.contains("--tab-py:var(--space-2)")
            || tabs_out.skin_css.contains("--tab-py: var(--space-2)"),
        "tabs size channels must land in skins:\n{}",
        tabs_out.skin_css
    );
    // Layout owns vertical host axis.
    assert!(
        tabs_out.skin_css.contains("[orientation='vertical']")
            && tabs_out.skin_css.contains("flex-direction:row"),
        "layout orientation flip on column host:\n{}",
        tabs_out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
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
    // Standard + webkit scrollbar packs on host sx → generated skin. No PE.
    let src = include_str!("../stdlib/scroll-area.sail");
    let out = compile_element(src, "scroll-area").expect("scroll-area compiles");
    assert!(
        src.contains("scrollbar-thin")
            && src.contains("scrollbar-thumb-border")
            && src.contains("scrollbar:w-2_5")
            && src.contains("scrollbar-thumb:bg-border")
            && src.contains("scrollbar-thumb:bg-clip-padding"),
        "scroll-area host sx must carry standard + webkit scrollbar packs"
    );
    assert!(
        out.structure_html.contains("sx-scrollbarthin")
            && out.structure_html.contains("sx-scrollbarthumbborder"),
        "scrollbar pack atoms on structure:\n{}",
        out.structure_html
    );
    assert!(
        out.skin_css.contains("scrollbar-width:thin")
            || out.skin_css.contains("scrollbar-width: thin")
            || out.structure_html.contains("sx-scrollbarthin"),
        "scrollbar-thin must land:\nstruct={}\nskin={}",
        out.structure_html,
        out.skin_css
    );
    assert!(
        out.skin_css.contains("--sx-scrollbar-thumb:var(--border)")
            || out.skin_css.contains("--sx-scrollbar-thumb: var(--border)")
            || out.skin_css.contains("scrollbar-color:"),
        "thumb border + color composition must lower:\n{}",
        out.skin_css
    );
    // Webkit anatomy via pseudo prefixes in generated skin.
    assert!(
        out.skin_css.contains("::-webkit-scrollbar")
            && (out.skin_css.contains("::-webkit-scrollbar-thumb")
                || out.structure_html.contains("scrollbarthumb")),
        "webkit scrollbar pseudo prefixes must lower into skin:\n{}",
        out.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
    ));
    let sa = contracts
        .split("/* scroll-area")
        .nth(1)
        .and_then(|s| s.split("/* ").next())
        .expect("scroll-area contracts section");
    // All scrollbar PE gone.
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
}

#[test]
fn code_block_pre_tab_and_tok_generated() {
    // pre tab-size:2 + line-height 1.6 → pre sx (tab-2 + leading-code).
    // .tok-* highlighter ink: compiler code-block map → element-skins (not PE).
    let src = include_str!("../stdlib/code-block.sail");
    let out = compile_element(src, "code-block").expect("code-block compiles");
    assert!(
        src.contains("tab-2") && src.contains("leading-code"),
        "code-block pre sx must carry tab-2 + leading-code"
    );
    assert!(
        out.structure_html.contains("sx-tab2")
            && out.structure_html.contains("sx-leadingcode"),
        "tab/leading atoms on structure:\n{}",
        out.structure_html
    );
    assert!(
        out.skin_css.contains("tab-size:2")
            || out.skin_css.contains("tab-size: 2")
            || out.structure_html.contains("sx-tab2"),
        "tab-2 must lower:\n{}",
        out.skin_css
    );
    assert!(
        out.skin_css.contains("line-height:var(--leading-code)")
            || out.skin_css.contains("line-height: var(--leading-code)")
            || out.structure_html.contains("sx-leadingcode"),
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
        "/../../../packages/sail/tokens/src/contracts.css"
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
    // :empty + [invalid]→hint hide + error show left contracts → structure sx
    // (empty: / group-invalid: + error-gate hidden/group-invalid:contents).
    let src = include_str!("../stdlib/field.sail");
    let out = compile_element(src, "field").expect("field compiles");
    assert!(
        src.contains("group")
            && src.contains("empty:hidden")
            && src.contains("group-invalid:hidden")
            && src.contains("group-invalid:contents")
            && src.contains("class=\"hint")
            && src.contains("class=\"error"),
        "field structure must carry group + empty/group-invalid hint/error gates"
    );
    assert!(
        out.structure_html.contains("sail-grp")
            && (out.structure_html.contains("sx-empty-hidden")
                || out.skin_css.contains(":empty{display:none}")
                || out.skin_css.contains(":empty{display:none;}"))
            && (out.structure_html.contains("sx-groupinvalid-hidden")
                || out.skin_css.contains(".sail-grp[invalid]")
                || out.skin_css.contains("[invalid]"))
            && (out.structure_html.contains("sx-groupinvalid-contents")
                || out.skin_css.contains("display:contents")),
        "empty: + group-invalid: must lower into structure/skin:\n{}\n{}",
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
        "/../../../packages/sail/tokens/src/contracts.css"
    ));
    let field = contracts
        .split("/* field")
        .nth(1)
        .and_then(|s| s.split("/* ").next())
        .expect("field contracts section");
    assert!(
        !field.contains(".sail-field .hint:empty")
            && !field.contains(".sail-field[invalid] .hint")
            && !field.contains(".sail-field:not([invalid]) .error"),
        "field hint/error PE rules must be gone:\n{field}"
    );
}

#[test]
fn label_req_gate_in_element_skins_anatomy() {
    // Native label never splices; authors place bare .req. Gate is element-skins
    // author anatomy (not contracts PE). Structure still documents group-required.
    let src = include_str!("../stdlib/label.sail");
    let out = compile_element(src, "label").expect("label compiles");
    assert!(
        src.contains("class=\"req\"")
            && src.contains("text-destructive")
            && src.contains("group-required:inline"),
        "structure documents .req anatomy + group-required paint"
    );
    assert!(out.native, "label is native-kind");
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
    ));
    assert!(
        !contracts.contains("label .req { display: none; }")
            && !contracts.contains("label[required] .req { display: inline; }"),
        "label .req PE must be gone from contracts"
    );
    let skins = sail_compiler::stdlib::skins_css();
    assert!(
        skins.contains("label .req{display:none}")
            && skins.contains("label[required] .req{display:inline}"),
        "label .req gate must live in element-skins anatomy:\n{}",
        skins.lines().filter(|l| l.contains(".req")).collect::<Vec<_>>().join("\n")
    );
    // .req paint still lowers (child anatomy classes for authors who use them).
    assert!(
        out.skin_css.contains("color:var(--destructive)")
            || out.skin_css.contains("color: var(--destructive)"),
        "req paint in skin:\n{}",
        out.skin_css
    );
}

#[test]
fn pagination_size_cva_on_page_not_parent_channels() {
    // use:pagination stamps size onto child pages; page size CVA owns geometry.
    // No --pg-* channel PE in contracts.
    let page_src = include_str!("../stdlib/page.sail");
    let pag_src = include_str!("../stdlib/pagination.sail");
    let page = compile_element(page_src, "page").expect("page compiles");
    let _pag = compile_element(pag_src, "pagination").expect("pagination compiles");
    assert!(
        page_src.contains("size: {")
            && page_src.contains("min-w-control-h-md")
            && page_src.contains("h-control-h-md")
            && page_src.contains("text-sm"),
        "page must own size CVA with control-h / text steps"
    );
    assert!(
        page.skin_css.contains("size='sm'")
            && page.skin_css.contains("size='lg'")
            && (page.skin_css.contains("min-width:var(--control-h-sm)")
                || page.skin_css.contains("min-width: var(--control-h-sm)"))
            && (page.skin_css.contains("height:var(--control-h-lg)")
                || page.skin_css.contains("height: var(--control-h-lg)")),
        "page size CVA must lower:\n{}",
        page.skin_css
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
    ));
    assert!(
        !contracts.contains("--pg-h:")
            && !contracts.contains("--pg-size:")
            && !contracts.contains(".sail-pagination { --pg-"),
        "pagination --pg-* channel PE must be gone from contracts"
    );
    assert!(
        pag_src.contains("use:pagination"),
        "pagination stamps size via use:pagination"
    );
}

#[test]
fn overlay_family_anchor_geometry_on_host_sx() {
    // dropdown / context-menu / hover-card / popover: fixed + inset-auto + m-0 +
    // left/top-(--anchor-*) left contracts PE → host sx. Backdrop is global
    // :where([popover])::backdrop. role=menu min-width is POPOVER_SKIN.
    for (tag, extra) in [
        ("dropdown", "use:menu"),
        ("context-menu", "use:contextMenu"),
        ("hover-card", "min-w-(--panel-rail-w)"),
        ("popover", "min-w-(--menu-min-w)"),
    ] {
        let src = sail_compiler::stdlib::sources()
            .iter()
            .find(|(t, _)| *t == tag)
            .map(|(_, s)| *s)
            .unwrap_or_else(|| panic!("{tag} registered"));
        let out = compile_element(src, tag).unwrap_or_else(|e| panic!("{tag}: {e:?}"));
        assert!(
            src.contains("fixed inset-auto m-0 left-(--anchor-left) top-(--anchor-top)")
                && src.contains(extra),
            "{tag} host sx must carry anchor geometry + extras"
        );
        assert!(
            out.structure_html.contains("sx-fixed")
                && out.structure_html.contains("sx-insetauto")
                && out.structure_html.contains("sx-m0")
                && out.structure_html.contains("sx-leftanchorleft")
                && out.structure_html.contains("sx-topanchortop"),
            "{tag} geometry atoms on structure:\n{}",
            out.structure_html
        );
    }
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
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
    // Listbox anchor geometry + scroll chrome on child sx; option anatomy is
    // global POPOVER_SKIN [role=listbox] (author children, no structure hook).
    let src = include_str!("../stdlib/combobox.sail");
    let out = compile_element(src, "combobox").expect("combobox compiles");
    assert!(
        src.contains("left-(--anchor-left) top-(--anchor-top) min-w-(--anchor-width)")
            && src.contains("max-h-panel-max-h-sm overflow-y-auto overscroll-contain"),
        "listbox sx must carry anchor geometry + scroll"
    );
    assert!(
        out.structure_html.contains("sx-fixed")
            && out.structure_html.contains("sx-minwanchorwidth")
            && out.structure_html.contains("sx-maxhpanelmaxhsm")
            && out.structure_html.contains("sx-overflowyauto")
            && out.structure_html.contains("sx-overscrollcontain"),
        "listbox geometry on structure:\n{}",
        out.structure_html
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
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
    assert!(
        out.structure_html.contains("sail-grp")
            && out.structure_html.contains("sx-relative")
            && out.structure_html.contains("sx-absolute")
            && out.structure_html.contains("sx-toptttop")
            && out.structure_html.contains("sx-grouphover-opacity100")
            && out.structure_html.contains("sx-after-rotate45"),
        "tooltip structure atoms:\n{}",
        out.structure_html
    );
    let contracts = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../packages/sail/tokens/src/contracts.css"
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
