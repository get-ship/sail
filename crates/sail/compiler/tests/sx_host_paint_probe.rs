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
