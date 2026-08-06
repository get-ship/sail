<!--
  <legend> — fieldset caption. Style product only.
-->

<scope>
  props: {}

  const base = style({ box: padX(2), paint: fg(foreground), text: fontSize(sm).fontWeight(medium) })
</scope>

<legend>
  style(base)

  <slot></slot>
</legend>
