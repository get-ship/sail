<!--
  <page> — size match owns cell geometry (stamped from pagination).

  Paint (Style product): fg / hover / current from component.page tokens
  (cssNames page.foreground, hover-bg, page.current-bg, page.current-fg).
-->
<scope>
  props: {
    value = ''
    disabled = false
    size: 'md' | 'sm' | 'lg' = 'md'
  }

  const base = style({ box: minW(control-h-md).h(control-h-md).padX(2), paint: fg(page.foreground).bg(transparent).radius(md).cursor(pointer).select(none).borderWidth(1).border(transparent).transition(colors).duration(fast).ease(out), text: fontFamily(body).fontWeight(medium).fontSize(sm) })
  const sizeMd = style({ box: minW(control-h-md).h(control-h-md), text: fontSize(sm) })
  const sizeSm = style({ box: minW(control-h-sm).h(control-h-sm), text: fontSize(xs) })
  const sizeLg = style({ box: minW(control-h-lg).h(control-h-lg), text: fontSize(base) })
</scope>
<button type="button" style(base, { when: { hover: { paint: bg(hover-bg) }, current: { paint: bg(page.current-bg).fg(page.current-fg).border(page.current-bg) }, disabled: { paint: opacity(50).cursor(not-allowed) }, focus: { paint: outline(solid).outline(2).outline(ring) } }, match: { size: { md: sizeMd, sm: sizeSm, lg: sizeLg } } })>
  <slot></slot>
</button>
