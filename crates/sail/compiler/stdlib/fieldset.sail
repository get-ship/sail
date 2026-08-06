<!--
  <fieldset> — form group. Style product only.
  Native groove border replaced with token surface.
-->

<scope>
  props: {
    disabled = false
  }

  const base = style.m(0).pad(4).borderWidth(1).border(border).radius(md).bg(transparent).when({
    disabled: style.opacity(50).cursor(not-allowed),
  })
</scope>

<fieldset>
  style(base)

  <slot></slot>
</fieldset>
