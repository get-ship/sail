<!--
  <input> — native field / toggle / range surface. Style product only.
  type axes paint: text-like fields, checkbox, radio, range, color, file.
  Freeform <input type=…> stamps host classes; CSS attribute gates activate.
-->

<scope>
  props: {
    type: 'text' | 'email' | 'password' | 'number' | 'search' | 'tel' | 'url' | 'date' | 'time' | 'datetime-local' | 'month' | 'week' | 'file' | 'color' | 'checkbox' | 'radio' | 'range' | 'submit' | 'hidden' = 'text'
    size: 'xs' | 'sm' | 'md' | 'lg' = 'md'
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    invalid = false
    disabled = false
    placeholder = ''
  }

  // Text-like fields share component.field surface tokens with select/textarea.
  style field = { box: display(block).minH(control-h-md).padY(1_5).padX(3).w(full), paint: fg(field.foreground).bg(field).borderWidth(1).border(field.border).radius(md).shadow(xs), text: fontSize(base), when: { hover: { paint: border(border-strong) }, disabled: { paint: cursor(not-allowed).opacity(50) } } }
  style sizeXs = { box: minH(control-h-2xs).padX(2), text: fontSize(xs) }
  style sizeSm = { box: minH(control-h-sm).padX(2_5), text: fontSize(sm) }
  style sizeMd = { box: minH(control-h-md).padX(3), text: fontSize(base) }
  style sizeLg = { box: minH(control-h-lg).padX(4), text: fontSize(md) }
  style fieldSized = style(field, { match: { size: { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg } } })

  // Border uses foreground (not border-strong) so checks stay visible on muted/soft surfaces.
  // Explicit border(solid): appearance:none leaves border-style:none in some UAs,
  // so borderWidth alone paints nothing (catalog "barely visible" checkboxes).
  style check = { box: display(inline-block).m(0).pad(0).minH(0).w(icon-size-sm).h(icon-size-sm), paint: appearance(none).align(middle).borderWidth(2).border(solid).border(field.foreground).bg(field).shadow(xs).radius(sm).cursor(pointer), text: fontSize(base), when: { checked: { paint: bg(primary).border(primary) }, disabled: { paint: cursor(not-allowed).opacity(50) } } }
  style radio = { box: display(inline-block).m(0).pad(0).minH(0).w(icon-size-sm).h(icon-size-sm), paint: appearance(none).align(middle).borderWidth(2).border(solid).border(field.foreground).bg(field).shadow(xs).radius(full).cursor(pointer), text: fontSize(base), when: { checked: { paint: bg(primary).border(primary) }, disabled: { paint: cursor(not-allowed).opacity(50) } } }
  style checkSm = { box: w(icon-size-xs).h(icon-size-xs) }
  style checkMd = { box: w(icon-size-sm).h(icon-size-sm) }
  style checkLg = { box: w(icon-size-md).h(icon-size-md) }
  style checkSized = style(check, { match: { size: { xs: checkSm, sm: checkSm, md: checkMd, lg: checkLg } } })
  style radioSized = style(radio, { match: { size: { xs: checkSm, sm: checkSm, md: checkMd, lg: checkLg } } })
  style range = { box: w(full).h(5), paint: appearance(none).bg(transparent).cursor(pointer), when: { disabled: { paint: cursor(not-allowed).opacity(50) } } }
  style color = { box: w(9).h(control-h-md).pad(1), paint: appearance(none).borderWidth(1).border(field.border).radius(md).bg(field).shadow(xs).cursor(pointer) }
  style file = { box: display(flex).minH(control-h-md).padY(1).padX(3), paint: cursor(pointer) }
  style fileSized = style(file, { match: { size: { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg } } })
  style hidden = { box: display(none) }
</scope>

<input style({
  match: {
    type: {
      text: fieldSized,
      email: fieldSized,
      password: fieldSized,
      number: fieldSized,
      search: fieldSized,
      tel: fieldSized,
      url: fieldSized,
      date: fieldSized,
      time: fieldSized,
      datetime-local: fieldSized,
      month: fieldSized,
      week: fieldSized,
      submit: fieldSized,
      file: fileSized,
      color: style(color, { box: minH(0) }),
      checkbox: checkSized,
      radio: radioSized,
      range: style(range, { box: minH(0) }),
      hidden: style(hidden, { box: minH(0) }),
    },
  },
  compound: [{ invalid: true, paint: border(destructive) }],
})>
