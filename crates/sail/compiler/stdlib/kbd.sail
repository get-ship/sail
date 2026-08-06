<!--
  <kbd> — mono keycap chip. Style product only.
  Paint: muted chip, mono type, doubled bottom edge (physical key affordance).
-->

<scope>
  props: {}

  const base = style({ box: display(inline-block).padY(px).padX(1_5), paint: bg(muted).borderWidth(1).border(border).borderBWidth(2).radius(sm).fg(foreground).align(middle), text: fontFamily(mono).fontSize(xs).leading(none).whitespace(nowrap) })
</scope>

<kbd>
  style(base)

  <slot></slot>
</kbd>
