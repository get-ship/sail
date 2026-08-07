<!--
  <input> — native field / toggle / range surface. Style product only.
  type axes paint: text-like fields, checkbox, radio, range, color, file.
  Freeform <input type=…> stamps host classes; CSS attribute gates activate.
-->

<scope>
  props: {
    type: 'text' | 'email' | 'password' | 'number' | 'search' | 'tel' | 'url' | 'date' | 'time' | 'datetime-local' | 'month' | 'week' | 'file' | 'color' | 'checkbox' | 'radio' | 'range' | 'submit' | 'hidden' = 'text'
    size: 'xs' | 'sm' | 'md' | 'lg' = 'md'
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    invalid = false
    disabled = false
    placeholder = ''
  }

  // Text-like fields share component.field surface tokens with select/textarea.
  const field = style({ box: display(block).minH(control-h-md).padY(1_5).padX(3).w(full), paint: fg(field-foreground).bg(field).borderWidth(1).border(field-border).radius(md).shadow(xs), text: fontSize(base) }).when(hover, { paint: border(field-border-hover) }).when(disabled, { paint: cursor(not-allowed).opacity(50) })
  const sizeXs = style({ box: minH(control-h-2xs).padX(2), text: fontSize(xs) })
  const sizeSm = style({ box: minH(control-h-sm).padX(2_5), text: fontSize(sm) })
  const sizeMd = style({ box: minH(control-h-md).padX(3), text: fontSize(base) })
  const sizeLg = style({ box: minH(control-h-lg).padX(4), text: fontSize(md) })

  // Border uses foreground (not border-strong) so checks stay visible on muted/soft surfaces.
  // Explicit border(solid): appearance:none leaves border-style:none in some UAs,
  // so borderWidth alone paints nothing (catalog "barely visible" checkboxes).
  const check = style({ box: display(inline-block).m(0).pad(0).minH(0).w(icon-size-sm).h(icon-size-sm), paint: appearance(none).align(middle).borderWidth(2).border(solid).border(field-foreground).bg(field).shadow(xs).radius(sm).cursor(pointer), text: fontSize(base) }).when(checked, { paint: bg(primary).border(primary) }).when(disabled, { paint: cursor(not-allowed).opacity(50) })
  const radio = style({ box: display(inline-block).m(0).pad(0).minH(0).w(icon-size-sm).h(icon-size-sm), paint: appearance(none).align(middle).borderWidth(2).border(solid).border(field-foreground).bg(field).shadow(xs).radius(full).cursor(pointer), text: fontSize(base) }).when(checked, { paint: bg(primary).border(primary) }).when(disabled, { paint: cursor(not-allowed).opacity(50) })
  const checkSm = style({ box: w(icon-size-xs).h(icon-size-xs) })
  const checkMd = style({ box: w(icon-size-sm).h(icon-size-sm) })
  const checkLg = style({ box: w(icon-size-md).h(icon-size-md) })
  const range = style({ box: w(full).h(5), paint: appearance(none).bg(transparent).cursor(pointer) }).when(disabled, { paint: cursor(not-allowed).opacity(50) })
  const color = style({ box: w(9).h(control-h-md).pad(1), paint: appearance(none).borderWidth(1).border(field-border).radius(md).bg(field).shadow(xs).cursor(pointer) })
  const file = style({ box: display(flex).minH(control-h-md).padY(1).padX(3), paint: cursor(pointer) })
  const hidden = style({ box: display(none) })
</scope>

<input style([
  match(type, {
    text: field,
    email: field,
    password: field,
    number: field,
    search: field,
    tel: field,
    url: field,
    date: field,
    time: field,
    datetime-local: field,
    month: field,
    week: field,
    submit: field,
    file: file,
    color: color,
    checkbox: check,
    radio: radio,
    range: range,
    hidden: hidden,
  }),
  match(type, {
    text: match(size, { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg }),
    email: match(size, { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg }),
    password: match(size, { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg }),
    number: match(size, { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg }),
    search: match(size, { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg }),
    tel: match(size, { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg }),
    url: match(size, { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg }),
    date: match(size, { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg }),
    time: match(size, { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg }),
    datetime-local: match(size, { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg }),
    month: match(size, { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg }),
    week: match(size, { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg }),
    submit: match(size, { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg }),
    file: match(size, { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg }),
    color: style({ box: minH(0) }),
    checkbox: match(size, { xs: checkSm, sm: checkSm, md: checkMd, lg: checkLg }),
    radio: match(size, { xs: checkSm, sm: checkSm, md: checkMd, lg: checkLg }),
    range: style({ box: minH(0) }),
    hidden: style({ box: minH(0) }),
  }),
  compound({ invalid: true }, style({ paint: border(destructive) })),
])>
