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
    surface: 'default' | 'filled' | 'flushed' = 'default'
    invalid = false
    disabled = false
    multiple = false
  }

  style stretch = {size({ w: 100% })}
  // Rest: input / foreground / border (shared with input/textarea).
  style base = {display(block).size({ min-h: control-h-md }).pad({ y: 1_5, s: 3, e: 8 }).fg(foreground).bg(control).border({ color: border, width: 1 }).radius(md).elevation(xs).cursor(pointer).type({ size: base }), when: {hover: border({ color: border-strong }), disabled: cursor(not-allowed).filter({ opacity: 50 }), group-fill: stretch}}
  style sizeXs = {size({ min-h: control-h-2xs }).pad({ s: 2 }).type({ size: xs })}
  style sizeSm = {size({ min-h: control-h-sm }).pad({ s: 2_5 }).type({ size: sm })}
  style sizeMd = {size({ min-h: control-h-md }).pad({ s: 3 }).type({ size: base })}
  style sizeLg = {size({ min-h: control-h-lg }).pad({ s: 4 }).type({ size: md })}
  // default = current field (bg input, border). filled = muted well, no outer border.
  // flushed = no fill, underline only (border edge b).
  style surfaceDefault = {bg(control).border({ color: border, width: 1 })}
  style surfaceFilled = {bg(muted).border({ color: transparent }).elevation(none), when: {hover: border({ color: transparent })}}
  style surfaceFlushed = {bg(transparent).elevation(none).radius(none).border({ width: 0, edge: l }).border({ width: 0, edge: r }).border({ width: 0, edge: t }).border({ width: 1, edge: b, color: border }), when: {hover: border({ color: border-strong, edge: b })}}
</scope>

<select
  style(base, {match: { size: { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg }, surface: { default: surfaceDefault, filled: surfaceFilled, flushed: surfaceFlushed } }, compound: [{invalid: true, border({ color: destructive })}]})
>
  <slot></slot>
</select>
