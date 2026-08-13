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
  style field = {box: flow({ display: block }).size({ min-h: control-h-md, w: full }).space({ pad-y: 1_5, pad-x: 3 }), paint: ink({ color: field.foreground }).fill({ color: field }).border({ color: field.border, width: 1 }).radius(md).elevation(xs), text: type({ size: base }), when: {hover: {paint: border({ color: border-strong })}, disabled: {paint: cursor(not-allowed).filter({ opacity: 50 })}}}
  // Password: reserve end chrome for reveal (select chevron model — padE, not padX).
  style fieldPassword = {box: flow({ display: block }).size({ min-h: control-h-md, w: full }).space({ pad-y: 1_5, pad-s: 3, pad-e: 8 }), paint: ink({ color: field.foreground }).fill({ color: field }).border({ color: field.border, width: 1 }).radius(md).elevation(xs), text: type({ size: base }), when: {hover: {paint: border({ color: border-strong })}, disabled: {paint: cursor(not-allowed).filter({ opacity: 50 })}}}
  style sizeXs = {box: size({ min-h: control-h-2xs }).space({ pad-x: 2 }), text: type({ size: xs })}
  style sizeSm = {box: size({ min-h: control-h-sm }).space({ pad-x: 2_5 }), text: type({ size: sm })}
  style sizeMd = {box: size({ min-h: control-h-md }).space({ pad-x: 3 }), text: type({ size: base })}
  style sizeLg = {box: size({ min-h: control-h-lg }).space({ pad-x: 4 }), text: type({ size: md })}
  // Password size ladder: padS only so padE(8) room is not clobbered.
  style sizeXsPw = {box: size({ min-h: control-h-2xs }).space({ pad-s: 2 }), text: type({ size: xs })}
  style sizeSmPw = {box: size({ min-h: control-h-sm }).space({ pad-s: 2_5 }), text: type({ size: sm })}
  style sizeMdPw = {box: size({ min-h: control-h-md }).space({ pad-s: 3 }), text: type({ size: base })}
  style sizeLgPw = {box: size({ min-h: control-h-lg }).space({ pad-s: 4 }), text: type({ size: md })}
  style fieldSized = style(field, { match: { size: { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg } } })
  style passwordSized = style(fieldPassword, { match: { size: { xs: sizeXsPw, sm: sizeSmPw, md: sizeMdPw, lg: sizeLgPw } } })
  style hidden = {box: flow({ display: none })}
</scope>

<input style({match: {type: {text: fieldSized, email: fieldSized, password: passwordSized, number: fieldSized, search: fieldSized, tel: fieldSized, url: fieldSized, hidden: style(hidden, {box: size({ min-h: 0 })})}}, compound: [{invalid: true, paint: border({ color: destructive })}]})>
