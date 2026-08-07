<!--
  <legend> — fieldset caption. Style product only.
-->

<scope>
  props: {}

  // Caption rides component.legend tokens (global fallbacks in the pack).
  const base = style({ box: padX(2), paint: fg(legend-foreground), text: fontSize(sm).fontWeight(medium) })
</scope>

<legend style(base)>
  <slot></slot>
</legend>
