<!--
  <summary> — disclosure trigger inside <details>. Style product only.
  ::after chevron mask + open rotate: SUMMARY_MAP / DETAILS_MAP.
-->

<scope>
  props: {}

  const base = style({ box: display(flex).padY(3).padX(4), paint: fg(foreground).cursor(pointer).list(none).select(none).radius(lg).transition(colors).duration(fast).ease(out), text: fontSize(sm).fontWeight(semibold) }).when(hover, { paint: bg(hover-bg) }).when(after, { box: ms(auto).size(4), paint: content(empty).bg(muted-foreground) })
</scope>

<summary>
  style(base)

  <slot></slot>
</summary>
