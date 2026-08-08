<!--
  <select> — native picker. Field surface + size ladder (Style).
  Chevron + option popup ink: SELECT_MAP (data-URI glyph / option colors).
  padS on the size ladder + padE(8) on base leave chevron room; size must
  not use padX (that shorthand overwrites end pad and collides with the glyph).
-->

<scope>
  props: {
    size: 'xs' | 'sm' | 'md' | 'lg' = 'md'
    invalid = false
    disabled = false
    multiple = false
  }

  // Shares component.field surface with input/textarea (global fallbacks in the pack).
  style base = {
    box: display(block).minH(control-h-md).padY(1_5).padS(3).padE(8).w(full),
    paint: fg(field.foreground).bg(field).borderWidth(1).border(field.border).radius(md).shadow(xs).cursor(pointer),
    text: fontSize(base),
    when: {
      hover: { paint: border(border-strong) },
      disabled: { paint: cursor(not-allowed).opacity(50) },
    },
  }
  style sizeXs = { box: minH(control-h-2xs).padS(2), text: fontSize(xs) }
  style sizeSm = { box: minH(control-h-sm).padS(2_5), text: fontSize(sm) }
  style sizeMd = { box: minH(control-h-md).padS(3), text: fontSize(base) }
  style sizeLg = { box: minH(control-h-lg).padS(4), text: fontSize(md) }
</scope>

<select
  style(base, {
    match: { size: { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg } },
    compound: [{ invalid: true, paint: border(destructive) }],
  })
>
  <slot></slot>
</select>
