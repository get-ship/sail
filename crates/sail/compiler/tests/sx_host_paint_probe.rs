//! Host-paint delete path — layout + sx + variants replace author `<style>`.
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
fn variants_axes_lower_with_commas() {
    // Legacy variants: field still lowers (same pipeline).
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
    let out = compile_element(src, "probe").expect("compile");
    assert!(out.skin_css.contains("variant='soft'"), "{}", out.skin_css);
    assert!(out.skin_css.contains("variant='solid'"), "{}", out.skin_css);
    assert!(out.skin_css.contains("variant='outline'"), "{}", out.skin_css);
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
