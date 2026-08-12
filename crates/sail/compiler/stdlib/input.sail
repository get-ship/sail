<!--
  <input> — text field product. Style product only.
  type: text | email | password | search | tel | url | number | hidden.
  Other controls are own tags: checkbox, radio, slider, date, file, color, switch.
  Actions use <button> — type=submit is not an input type.
  Shared component.field surface with select/textarea.
  reveal (default true): password peek chrome (padE + use:passwordReveal).
-->

<scope>
  props: {
    type: 'text' | 'email' | 'password' | 'number' | 'search' | 'tel' | 'url' | 'hidden' = 'text'
    size: 'xs' | 'sm' | 'md' | 'lg' = 'md'
    invalid = false
    disabled = false
    placeholder = ''
    name = ''
    required = false
    readonly = false
    value = ''
    min = ''
    max = ''
    step = ''
    minlength = ''
    maxlength = ''
    pattern = ''
    autocomplete = ''
    inputmode = ''
    reveal = true
  }

  // Text-like fields share component.field surface with select/textarea.
  style field = {
    box: display(block).minH(control-h-md).padY(1_5).padX(3).w(full),
    paint: fg(field.foreground).bg(field).borderWidth(1).border(field.border).radius(md).shadow(xs),
    text: fontSize(base),
    when: {
      hover: { paint: border(border-strong) },
      disabled: { paint: cursor(not-allowed).opacity(50) },
    },
  }
  // Password: reserve end chrome for reveal (select chevron model — padE, not padX).
  style fieldPassword = {
    box: display(block).minH(control-h-md).padY(1_5).padS(3).padE(8).w(full),
    paint: fg(field.foreground).bg(field).borderWidth(1).border(field.border).radius(md).shadow(xs),
    text: fontSize(base),
    when: {
      hover: { paint: border(border-strong) },
      disabled: { paint: cursor(not-allowed).opacity(50) },
    },
  }
  style sizeXs = { box: minH(control-h-2xs).padX(2), text: fontSize(xs) }
  style sizeSm = { box: minH(control-h-sm).padX(2_5), text: fontSize(sm) }
  style sizeMd = { box: minH(control-h-md).padX(3), text: fontSize(base) }
  style sizeLg = { box: minH(control-h-lg).padX(4), text: fontSize(md) }
  // Password size ladder: padS only so padE(8) room is not clobbered.
  style sizeXsPw = { box: minH(control-h-2xs).padS(2), text: fontSize(xs) }
  style sizeSmPw = { box: minH(control-h-sm).padS(2_5), text: fontSize(sm) }
  style sizeMdPw = { box: minH(control-h-md).padS(3), text: fontSize(base) }
  style sizeLgPw = { box: minH(control-h-lg).padS(4), text: fontSize(md) }
  style fieldSized = style(field, { match: { size: { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg } } })
  style passwordSized = style(fieldPassword, { match: { size: { xs: sizeXsPw, sm: sizeSmPw, md: sizeMdPw, lg: sizeLgPw } } })
  style hidden = { box: display(none) }
</scope>

<input style({
  match: {
    type: {
      text: fieldSized,
      email: fieldSized,
      password: passwordSized,
      number: fieldSized,
      search: fieldSized,
      tel: fieldSized,
      url: fieldSized,
      hidden: style(hidden, { box: minH(0) }),
    },
  },
  compound: [{ invalid: true, paint: border(destructive) }],
})>
