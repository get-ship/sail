<!--
  <fieldset> — form group. Style product only.
  Native groove border replaced with token surface.
-->

<scope>
  props: {
    disabled = false
  }

  style base = {pad(4).margin(0).border({ color: border, width: 1 }).radius(md).bg(transparent), when: {disabled: filter({ opacity: 50 }).cursor(not-allowed)}}
</scope>

<fieldset style(base)>
  <slot></slot>
</fieldset>
