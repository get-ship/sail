<!--
  <input> — text field product. Style product only.
  type: text | email | password | search | tel | url | number | hidden.
  Other controls are own tags: checkbox, radio, slider, date, file, color, switch.
  Actions use <button> — type=submit is not an input type.
  Shared input / foreground / border with select/textarea.
  reveal (default true): password peek chrome (padE).
-->

<scope>
  props: {
    type: 'text' | 'email' | 'password' | 'number' | 'search' | 'tel' | 'url' | 'hidden' = 'text'
    size: 'xs' | 'sm' | 'md' | 'lg' = 'md'
    surface: 'default' | 'filled' | 'flushed' = 'default'
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

  // Replaced element: width:auto is the UA size=20 (~174px). Inside a
  // field (group-fill) the control fills the column. Naked input hugs.
  style stretch = {size({ w: 100% })}
  // Text-like fields share input / foreground / border with select/textarea.
  style field = {display(block).size({ min-h: control-h-md }).pad({ y: 1_5, x: 3 }).fg(foreground).bg(inset).border({ color: border, width: 1 }).radius(md).elevation(xs).type({ size: base }), when: {hover: border({ color: border-strong }), disabled: cursor(not-allowed).filter({ opacity: 50 }), group-fill: stretch}}
  // Password: reserve end chrome for reveal (select chevron model — padE, not padX).
  style fieldPassword = {display(block).size({ min-h: control-h-md }).pad({ y: 1_5, s: 3, e: 8 }).fg(foreground).bg(inset).border({ color: border, width: 1 }).radius(md).elevation(xs).type({ size: base }), when: {hover: border({ color: border-strong }), disabled: cursor(not-allowed).filter({ opacity: 50 }), group-fill: stretch}}
  style sizeXs = {size({ min-h: control-h-2xs }).pad({ x: 2 }).type({ size: xs })}
  style sizeSm = {size({ min-h: control-h-sm }).pad({ x: 2_5 }).type({ size: sm })}
  style sizeMd = {size({ min-h: control-h-md }).pad({ x: 3 }).type({ size: base })}
  style sizeLg = {size({ min-h: control-h-lg }).pad({ x: 4 }).type({ size: md })}
  // Password size ladder: padS only so padE(8) room is not clobbered.
  style sizeXsPw = {size({ min-h: control-h-2xs }).pad({ s: 2 }).type({ size: xs })}
  style sizeSmPw = {size({ min-h: control-h-sm }).pad({ s: 2_5 }).type({ size: sm })}
  style sizeMdPw = {size({ min-h: control-h-md }).pad({ s: 3 }).type({ size: base })}
  style sizeLgPw = {size({ min-h: control-h-lg }).pad({ s: 4 }).type({ size: md })}
  style fieldSized = style(field, { match: { size: { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg } } })
  style passwordSized = style(fieldPassword, { match: { size: { xs: sizeXsPw, sm: sizeSmPw, md: sizeMdPw, lg: sizeLgPw } } })
  style hidden = {display(none)}
  // default = current field (bg input, border). filled = muted well, no outer border.
  // flushed = no fill, underline only (border edge b).
  style surfaceDefault = {bg(inset).border({ color: border, width: 1 })}
  style surfaceFilled = {bg(muted).border({ color: transparent }).elevation(none), when: {hover: border({ color: transparent })}}
  style surfaceFlushed = {bg(transparent).elevation(none).radius(none).border({ width: 0, edge: l }).border({ width: 0, edge: r }).border({ width: 0, edge: t }).border({ width: 1, edge: b, color: border }), when: {hover: border({ color: border-strong, edge: b })}}
</scope>

<input style({match: {type: {text: fieldSized, email: fieldSized, password: passwordSized, number: fieldSized, search: fieldSized, tel: fieldSized, url: fieldSized, hidden: style(hidden, {size({ min-h: 0 })})}, surface: { default: surfaceDefault, filled: surfaceFilled, flushed: surfaceFlushed }}, compound: [{invalid: true, border({ color: danger })}]})>
