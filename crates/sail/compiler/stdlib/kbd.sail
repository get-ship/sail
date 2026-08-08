<!--
  <kbd> — mono keycap chip. Style product only.
  Paint: muted chip, mono type, doubled bottom edge (physical key affordance).
-->

<scope>
  props: {}

  // Keycap paint from component.kbd tokens (global fallbacks in the pack).
  const base = style({ box: display(inline-block).padY(px).padX(1_5), paint: bg(kbd).borderWidth(1).border(kbd.border).borderBWidth(2).radius(sm).fg(kbd.foreground).align(middle), text: fontFamily(mono).fontSize(xs).leading(none).whitespace(nowrap) })
</scope>

<kbd style(base)>
  <slot></slot>
</kbd>
