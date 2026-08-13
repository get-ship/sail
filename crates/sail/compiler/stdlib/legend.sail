<!--
  <legend> — fieldset caption. Style product only.
-->

<scope>
  props: {}

  // Caption rides foreground.
  style base = {box: space({ pad-x: 2 }), paint: ink({ color: foreground }), text: type({ size: sm, weight: medium })}
</scope>

<legend style(base)>
  <slot></slot>
</legend>
