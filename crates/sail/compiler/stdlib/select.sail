<!--
  <select> — native picker. Field surface + size ladder (Style).
  Chevron + option popup ink: SELECT_MAP (data-URI glyph / option colors).
  padS on the size ladder + padE(8) on base leave chevron room; size must
  not use padX (that shorthand overwrites end pad and collides with the glyph).
-->

<scope>
  props: {
    size: 'xs' | 'sm' | 'md' | 'lg' = 'md'
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    invalid = false
    disabled = false
    multiple = false
  }

  // Shares component.field surface with input/textarea (global fallbacks in the pack).
  const base = style({ box: display(block).minH(control-h-md).padY(1_5).padS(3).padE(8).w(full), paint: fg(field-foreground).bg(field).borderWidth(1).border(field-border).radius(md).shadow(xs).cursor(pointer), text: fontSize(base) }).when(hover, { paint: border(field-border-hover) }).when(disabled, { paint: cursor(not-allowed).opacity(50) })
  const sizeXs = style({ box: minH(control-h-2xs).padS(2), text: fontSize(xs) })
  const sizeSm = style({ box: minH(control-h-sm).padS(2_5), text: fontSize(sm) })
  const sizeMd = style({ box: minH(control-h-md).padS(3), text: fontSize(base) })
  const sizeLg = style({ box: minH(control-h-lg).padS(4), text: fontSize(md) })
</scope>

<select style([
  base,
  match(size, {
    xs: sizeXs,
    sm: sizeSm,
    md: sizeMd,
    lg: sizeLg,
  }),
  compound({ invalid: true }, style({ paint: border(destructive) })),
])>
  <slot></slot>
</select>
