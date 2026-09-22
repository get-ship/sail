<!--
  <input> — text field product.
  type: text | email | password | search | tel | url | number | hidden.
  Other controls are own tags: checkbox, radio, slider, date, file, color, switch.
  Actions use <button> — type=submit is not an input type.
  Shared input / foreground / border with select/textarea.

  Look: this tag owns its Style. Slots, not :part.
  State: this file's <controller> (kind + shown + revealable). No DOM.
  Listen: native <button> Show/Hide; native <input>.
  reveal (default true): password peek chrome (padE + sibling Show/Hide).
-->

<scope>
  props: {
    // checkbox/radio: legal native types on RAW <input> inside <checkbox> /
    // <switch> / <radio> hosts. Not this tag's product surface — the registry
    // carries them so sail-prop-value does not reject sibling stdlib sources.
    // file/color/date/button live here too: native types on RAW <input> inside
    // their own hosts (<file>, <color>, <date>). Not this tag's product surface.
    type: 'text' | 'email' | 'password' | 'number' | 'search' | 'tel' | 'url' | 'hidden' | 'checkbox' | 'radio' | 'file' | 'color' | 'date' | 'button' | 'range' = 'text'
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
    id = ''
    describedBy = ''
  }

  style host = {display(contents).size({ min-w: 0 }), when: { group-fill: stretch }}
  style hostPassword = {display(inline-flex).position(relative).size({ min-w: 0 }), when: { group-fill: stretch }}
  style hostHidden = {display(none)}
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
  style peek = {display(none).position({ scheme: absolute, r: 2, t: 0, b: 0, z: 1 }).margin({ y: auto }).pad({ x: 2 }).bg(transparent).border({ width: 0 }).fg(muted-foreground).cursor(pointer).type({ size: sm, wrap: nowrap, leading: none, family: body }), animate: { transition: play(colors, fast) }}
</scope>

<controller>
export default controller((init) => {
  function read(v) {
    return typeof v === 'function' ? v() : v
  }
  function isOn(v) {
    const x = read(v)
    return x === true || x === '' || x === 'true'
  }
  function kindOf(v) {
    const t = String(read(v) || 'text')
    if (
      t === 'email' ||
      t === 'password' ||
      t === 'number' ||
      t === 'search' ||
      t === 'tel' ||
      t === 'url' ||
      t === 'hidden'
    ) {
      return t
    }
    return 'text'
  }
  const shown = signal(false)
  const kind = derived(() => kindOf(init.type))
  const isPassword = derived(() => kind() === 'password')
  const revealable = derived(() => isPassword() && isOn(init.reveal))
  const fieldType = derived(() => (revealable() && shown() ? 'text' : kind()))
  const showPeek = derived(() => revealable() && !shown())
  const hidePeek = derived(() => revealable() && shown())
  // CSS display:none needs the framework sheet; the a11y contract must hold
  // headless too — state the visibility on the element.
  const showPeekHidden = derived(() => !showPeek())
  const hidePeekHidden = derived(() => !hidePeek())
  const invalid = derived(() => isOn(init.invalid))
  // ARIA contract lives on the control: aria-invalid mirrors the design-system
  // `invalid` presence (explicit false, not absent, so the state is stated);
  // aria-describedby passes the field host's hint/error id through.
  const ariaInvalid = derived(() => (invalid() ? 'true' : 'false'))
  const describedBy = derived(() => String(read(init.describedBy) ?? ''))
  const ariaDescribedBy = derived(() => {
    const id = String(read(init.describedBy) ?? '')
    return id === '' ? null : id
  })
  const disabled = derived(() => isOn(init.disabled))
  const required = derived(() => isOn(init.required))
  const readOnly = derived(() => isOn(init.readonly))
  function toggle() {
    if (!revealable() || isOn(init.disabled)) return
    shown.set(!shown())
  }
  return {
    kind,
    isPassword,
    fieldType,
    showPeek,
    hidePeek,
    showPeekHidden,
    hidePeekHidden,
    toggle,
    invalid,
    ariaInvalid,
    ariaDescribedBy,
    disabled,
    required,
    readOnly,
  }
})
</controller>

<span
  type={kind()}
  password={isPassword()}
  style(host, {match: { type: { hidden: hostHidden }, password: { true: hostPassword } }})
>
  <input
    anatomy
    type={fieldType()}
    password={isPassword()}
    aria-invalid={ariaInvalid()}
    aria-describedby={ariaDescribedBy()}
    invalid={invalid()}
    disabled={disabled()}
    required={required()}
    readOnly={readOnly()}
    name={name()}
    value={value()}
    min={min()}
    max={max()}
    step={step()}
    minlength={minlength()}
    maxlength={maxlength()}
    pattern={pattern()}
    autocomplete={autocomplete()}
    inputmode={inputmode()}
    placeholder={placeholder()}
    size={size()}
    id={id()}
    style({match: {type: {text: fieldSized, email: fieldSized, password: passwordSized, number: fieldSized, search: fieldSized, tel: fieldSized, url: fieldSized, hidden: style(hidden, {size({ min-h: 0 })})}, password: { true: passwordSized }, surface: { default: surfaceDefault, filled: surfaceFilled, flushed: surfaceFlushed }}, compound: [{invalid: true, border({ color: danger })}]})
  >
  <button
    type="button"
    anatomy
    tabindex="-1"
    label="Show"
    peek={showPeek()}
    hidden={showPeekHidden()}
    on:click={toggle}
    style(peek, {match: {peek: {true: display(inline-flex), false: display(none)}}, when: {hover: fg(foreground), focus: border({ style: solid, kind: outline, width: 2, color: ring, offset: 2 })}})
  >Show</button>
  <button
    type="button"
    anatomy
    tabindex="-1"
    label="Hide"
    peek={hidePeek()}
    hidden={hidePeekHidden()}
    on:click={toggle}
    style(peek, {match: {peek: {true: display(inline-flex), false: display(none)}}, when: {hover: fg(foreground), focus: border({ style: solid, kind: outline, width: 2, color: ring, offset: 2 })}})
  >Hide</button>
</span>
