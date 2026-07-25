<!--
  <page> — size match owns cell geometry (stamped from pagination).

  Paint (Style product): fg / hover / current from component.page tokens
  (cssNames page-foreground, page-hover-bg, page-current-bg, page-current-foreground).
-->
<scope>
  props: {
    value = ''
    disabled = false
    size: 'md' | 'sm' | 'lg' = 'md'
  }

  const base = style.fontFamily(body).fontWeight(medium).fontSize(sm).fg(page-foreground).bg(transparent).radius(md).cursor(pointer).select(none).minW(control-h-md).h(control-h-md).padX(2).borderWidth(1).border(transparent).transition(colors).duration(fast).ease(out)
  const sizeMd = style.fontSize(sm).minW(control-h-md).h(control-h-md)
  const sizeSm = style.fontSize(xs).minW(control-h-sm).h(control-h-sm)
  const sizeLg = style.fontSize(base).minW(control-h-lg).h(control-h-lg)
</scope>
<button type="button" sx={[
  base,
  match(size, {
    md: sizeMd,
    sm: sizeSm,
    lg: sizeLg,
  }),
  when({
    hover: style.bg(page-hover-bg),
    current: style.bg(page-current-bg).fg(page-current-foreground).border(page-current-bg),
    disabled: style.opacity(50).cursor(not-allowed),
    focus: style.outline(solid).outline(2).outline(ring),
  }),
]}>
  <slot></slot>
</button>
