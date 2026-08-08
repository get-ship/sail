<!--
  <textarea> — native multi-line field. Style product only.
  Shares the field surface with <input>; residual no longer carries field paint.
-->

<scope>
  props: {
    size: 'xs' | 'sm' | 'md' | 'lg' = 'md'
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    invalid = false
    disabled = false
    placeholder = ''
    rows = 3
  }

  // Shares component.field surface with input/select (global fallbacks in the pack).
  style base = { box: display(block).minH(control-h-md).padY(1_5).padX(3).w(full), paint: fg(field.foreground).bg(field).borderWidth(1).border(field.border).radius(md).shadow(xs), text: fontSize(base), when: { hover: { paint: border(border-strong) }, disabled: { paint: cursor(not-allowed).opacity(50) } } }
  style sizeXs = { box: minH(control-h-2xs).padX(2), text: fontSize(xs) }
  style sizeSm = { box: minH(control-h-sm).padX(2_5), text: fontSize(sm) }
  style sizeMd = { box: minH(control-h-md).padX(3), text: fontSize(base) }
  style sizeLg = { box: minH(control-h-lg).padX(4), text: fontSize(md) }
</scope>

<textarea style(base, { match: { size: { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg } }, compound: [ { invalid: true, paint: border(destructive) } ] })></textarea>
