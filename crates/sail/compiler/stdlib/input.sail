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

  const field = style.display(block).minH(control-h-md).padY(1_5).padX(3).fontSize(base).fg(foreground).bg(background).borderWidth(1).border(border).radius(md).shadow(xs).w(full).when({ hover: style.border(border-strong), disabled: style.cursor(not-allowed).opacity(50) })
  const sizeXs = style.minH(control-h-2xs).padX(2).fontSize(xs)
  const sizeSm = style.minH(control-h-sm).padX(2_5).fontSize(sm)
  const sizeMd = style.minH(control-h-md).padX(3).fontSize(base)
  const sizeLg = style.minH(control-h-lg).padX(4).fontSize(md)

  // Border uses foreground (not border-strong) so checks stay visible on muted/soft surfaces.
  // Explicit border(solid): appearance:none leaves border-style:none in some UAs,
  // so borderWidth alone paints nothing (catalog "barely visible" checkboxes).
  const check = style.appearance(none).display(inline-block).align(middle).m(0).pad(0).minH(0).w(icon-size-sm).h(icon-size-sm).fontSize(base).borderWidth(2).border(solid).border(foreground).bg(background).shadow(xs).radius(sm).cursor(pointer).when({
    checked: style.bg(primary).border(primary),
    disabled: style.cursor(not-allowed).opacity(50),
  })
  const radio = style.appearance(none).display(inline-block).align(middle).m(0).pad(0).minH(0).w(icon-size-sm).h(icon-size-sm).fontSize(base).borderWidth(2).border(solid).border(foreground).bg(background).shadow(xs).radius(full).cursor(pointer).when({
    checked: style.bg(primary).border(primary),
    disabled: style.cursor(not-allowed).opacity(50),
  })
  const checkSm = style.w(icon-size-xs).h(icon-size-xs)
  const checkMd = style.w(icon-size-sm).h(icon-size-sm)
  const checkLg = style.w(icon-size-md).h(icon-size-md)
  const range = style.appearance(none).w(full).h(5).bg(transparent).cursor(pointer).when({
    disabled: style.cursor(not-allowed).opacity(50),
  })
  const color = style.appearance(none).w(9).h(control-h-md).pad(1).borderWidth(1).border(border).radius(md).bg(background).shadow(xs).cursor(pointer)
  const file = style.display(flex).minH(control-h-md).padY(1).padX(3).cursor(pointer)
  const hidden = style.display(none)
</scope>

<input sx={[
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
    color: style.minH(0),
    checkbox: match(size, { xs: checkSm, sm: checkSm, md: checkMd, lg: checkLg }),
    radio: match(size, { xs: checkSm, sm: checkSm, md: checkMd, lg: checkLg }),
    range: style.minH(0),
    hidden: style.minH(0),
  }),
  compound({ invalid: true }, style.border(destructive)),
]}>
