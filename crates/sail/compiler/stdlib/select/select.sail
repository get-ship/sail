<!--
  <select> — native closed picker only (host Style). Not a combobox/listbox.
  Rest surface shares input / foreground / border with input/textarea
  (fg · bg · border → --field-foreground / --field / --field-border).
  Hover spends free border-strong (SPEC-D1 rest-only; field-border-hover is
  storage residual, not a host Style rest leaf). invalid → destructive border.
  Chevron + option popup ink: SELECT_MAP (data-URI glyph / option field colors).
  padS on the size ladder + padE(8) on base leave chevron room; size must
  not use padX (that shorthand overwrites end pad and collides with the glyph).
  multiple = native multi-select attr (no host chrome change). Styled listbox
  with arbitrary item content is <combobox> — a different L1.
-->

<scope>
  props: {
    size: 'xs' | 'sm' | 'md' | 'lg' = 'md'
    invalid = false
    disabled = false
    multiple = false
  }

  // Rest: input / foreground / border (shared with input/textarea).
  style base = {display(block).size({ min-h: control-h-md }).pad({ y: 1_5, s: 3, e: 8 }).fg(foreground).bg(input).border({ color: border, width: 1 }).radius(md).elevation(xs).cursor(pointer).type({ size: base }), when: {hover: border({ color: border-strong }), disabled: cursor(not-allowed).filter({ opacity: 50 })}}
  style sizeXs = {size({ min-h: control-h-2xs }).pad({ s: 2 }).type({ size: xs })}
  style sizeSm = {size({ min-h: control-h-sm }).pad({ s: 2_5 }).type({ size: sm })}
  style sizeMd = {size({ min-h: control-h-md }).pad({ s: 3 }).type({ size: base })}
  style sizeLg = {size({ min-h: control-h-lg }).pad({ s: 4 }).type({ size: md })}
</scope>

<select
  style(base, {match: { size: { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg } }, compound: [{invalid: true, border({ color: destructive })}]})
>
  <slot></slot>
</select>
