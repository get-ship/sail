<!--
  <summary> — disclosure trigger inside <details>. Style product only.
  ::after chevron mask + open rotate: SUMMARY_MAP / DETAILS_MAP.
-->

<scope>
  props: {}

  const base = style.display(flex).padY(3).padX(4).fontSize(sm).fontWeight(semibold).fg(foreground).cursor(pointer).list(none).select(none).radius(lg).transition(colors).duration(fast).ease(out).when({
    hover: style.bg(hover-bg),
    after: style.content(empty).ms(auto).size(4).bg(muted-foreground),
  })
</scope>

<summary>
  style(base)

  <slot></slot>
</summary>
