<!--
  <select> — native picker. Field surface + size ladder (Style).
  Chevron + option popup ink: SELECT_MAP (data-URI glyph / option colors).
  padS on the size ladder + padE(8) on base leave chevron room; size must
  not use padX (that shorthand overwrites end pad and collides with the glyph).
-->

<scope>
  props: {
    size: 'xs' | 'sm' | 'md' | 'lg' = 'md'
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    invalid = false
    disabled = false
    multiple = false
  }

  const base = style.display(block).minH(control-h-md).padY(1_5).padS(3).padE(8).fontSize(base).fg(foreground).bg(background).borderWidth(1).border(border).radius(md).shadow(xs).w(full).cursor(pointer).when({ hover: style.border(border-strong), disabled: style.cursor(not-allowed).opacity(50) })
  const sizeXs = style.minH(control-h-2xs).padS(2).fontSize(xs)
  const sizeSm = style.minH(control-h-sm).padS(2_5).fontSize(sm)
  const sizeMd = style.minH(control-h-md).padS(3).fontSize(base)
  const sizeLg = style.minH(control-h-lg).padS(4).fontSize(md)
</scope>

<select>
  style([
  base,
  match(size, {
    xs: sizeXs,
    sm: sizeSm,
    md: sizeMd,
    lg: sizeLg,
  }),
  compound({ invalid: true }, style.border(destructive)),
])

  <slot></slot>
</select>
