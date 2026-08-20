<!--
  <date> — temporal picker. Native <input>; type selects precision
  (date | time | datetime-local | month | week). Platform calendar/time UI.

  Look: this tag owns its Style. Slots, not :part.
  State: this file's <controller> (type + invalid/disabled/required/readonly
  as signals). No DOM. No date action.
  Listen: native <input>. No hotkeys.
-->

<scope>
  props: {
    type: 'date' | 'time' | 'datetime-local' | 'month' | 'week' = 'date'
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
  }

  style base = {display(block).size({ min-h: control-h-md }).pad({ y: 1_5, x: 3 }).fg(foreground).bg(inset).border({ color: border, width: 1 }).radius(md).elevation(xs).type({ size: base }), when: {hover: border({ color: border-strong }), disabled: cursor(not-allowed).filter({ opacity: 50 })}}
  style sizeXs = {size({ min-h: control-h-2xs }).pad({ x: 2 }).type({ size: xs })}
  style sizeSm = {size({ min-h: control-h-sm }).pad({ x: 2_5 }).type({ size: sm })}
  style sizeMd = {size({ min-h: control-h-md }).pad({ x: 3 }).type({ size: base })}
  style sizeLg = {size({ min-h: control-h-lg }).pad({ x: 4 }).type({ size: md })}
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
    const t = String(read(v) || 'date')
    if (t === 'time' || t === 'datetime-local' || t === 'month' || t === 'week') return t
    return 'date'
  }
  const type = derived(() => kindOf(init.type))
  const invalid = derived(() => isOn(init.invalid))
  const disabled = derived(() => isOn(init.disabled))
  const required = derived(() => isOn(init.required))
  const readOnly = derived(() => isOn(init.readonly))
  return { type, invalid, disabled, required, readOnly }
})
</controller>

<input
  anatomy
  type={type()}
  invalid={invalid()}
  disabled={disabled()}
  required={required()}
  readOnly={readOnly()}
  name={name()}
  value={value()}
  min={min()}
  max={max()}
  step={step()}
  placeholder={placeholder()}
  size={size()}
  style(base, {match: { size: { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg } }, compound: [{invalid: true, border({ color: danger })}]})
>
