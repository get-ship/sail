<!--
  <select> — native closed picker only (host Style). Not a combobox/listbox.
  Rest surface shares component.field.* with input/textarea
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

  // Rest: component.field surface (shared with input/textarea).
  style base = {box: flow({ display: block }).size({ min-h: control-h-md, w: full }).space({ pad-y: 1_5, pad-s: 3, pad-e: 8 }), paint: ink({ color: field.foreground }).fill({ color: field }).border({ color: field.border, width: 1 }).radius(md).elevation(xs).cursor(pointer), text: type({ size: base }), when: {hover: {paint: border({ color: border-strong })}, disabled: {paint: cursor(not-allowed).filter({ opacity: 50 })}}}
  style sizeXs = {box: size({ min-h: control-h-2xs }).space({ pad-s: 2 }), text: type({ size: xs })}
  style sizeSm = {box: size({ min-h: control-h-sm }).space({ pad-s: 2_5 }), text: type({ size: sm })}
  style sizeMd = {box: size({ min-h: control-h-md }).space({ pad-s: 3 }), text: type({ size: base })}
  style sizeLg = {box: size({ min-h: control-h-lg }).space({ pad-s: 4 }), text: type({ size: md })}
</scope>

<select
  style(base, {match: { size: { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg } }, compound: [{invalid: true, paint: border({ color: destructive })}]})
>
  <slot></slot>
</select>
