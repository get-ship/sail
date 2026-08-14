<!--
  <legend> — fieldset caption. Style product only.
-->

<scope>
  props: {}

  // Caption rides foreground.
  style base = {pad({ x: 2 }).fg(foreground).type({ size: sm, weight: medium })}
</scope>

<legend style(base)>
  <slot></slot>
</legend>
