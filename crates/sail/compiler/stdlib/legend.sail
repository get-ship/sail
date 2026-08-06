<!--
  <legend> — fieldset caption. Style product only.
-->

<scope>
  props: {}

  const base = style.padX(2).fontSize(sm).fontWeight(medium).fg(foreground)
</scope>

<legend>
  style(base)

  <slot></slot>
</legend>
