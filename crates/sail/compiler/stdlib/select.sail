<!--
  <select> — native picker. Field surface + size ladder (Style).
  Chevron + option popup ink: SELECT_MAP (data-URI glyph / option colors).
-->

<scope>
  props: {
    size: 'xs' | 'sm' | 'md' | 'lg' = 'md'
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    invalid = false
    disabled = false
    multiple = false
  }

  const base = style.display(block).minH(control-h-md).padY(1_5).padX(3).fontSize(base).fg(foreground).bg(background).borderWidth(1).border(border).radius(md).shadow(xs).w(full).cursor(pointer).when({ hover: style.border(border-strong), disabled: style.cursor(not-allowed).opacity(50) })
  const sizeXs = style.minH(control-h-2xs).padX(2).fontSize(xs)
  const sizeSm = style.minH(control-h-sm).padX(2_5).fontSize(sm)
  const sizeMd = style.minH(control-h-md).padX(3).fontSize(base)
  const sizeLg = style.minH(control-h-lg).padX(4).fontSize(md)
</scope>

<select sx={[
  base,
  match(size, {
    xs: sizeXs,
    sm: sizeSm,
    md: sizeMd,
    lg: sizeLg,
  }),
  compound({ invalid: true }, style.border(destructive)),
]}>
  <slot></slot>
</select>
