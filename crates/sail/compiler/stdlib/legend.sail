<!--
  <legend> — fieldset caption. Style product only.
-->

<scope>
  props: {}

  // Caption rides component.legend tokens (global fallbacks in the pack).
  style base = {box: space({ pad-x: 2 }), paint: ink({ color: legend.foreground }), text: type({ size: sm, weight: medium })}
</scope>

<legend style(base)>
  <slot></slot>
</legend>
