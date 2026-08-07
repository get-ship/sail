<!--
  <fieldset> — form group. Style product only.
  Native groove border replaced with token surface.
-->

<scope>
  props: {
    disabled = false
  }

  // Group chrome rides component.fieldset tokens (global fallbacks in the pack).
  const base = style({ box: m(0).pad(4), paint: borderWidth(1).border(fieldset-border).radius(md).bg(transparent) }).when(disabled, { paint: opacity(50).cursor(not-allowed) })
</scope>

<fieldset style(base)>
  <slot></slot>
</fieldset>
