<!--
  <radio-group> — vertical group of <radio> options.

  Look: this host owns layout. Each <radio> owns its Style. Slots, not :part.
  State: this file's <controller> (value + name + select). No DOM.
  Child radios take `groupValue` / `select` / `name` / `groupDisabled` as
  props from this instance (signal cell, not a snapshot).
  Listen: native <input type=radio> with shared name (exclusive check,
  arrow roving, focus, form). No use:hotkeys.
-->

<scope>
  props: {
    value = ''
    disabled = false
    name = ''
    label = ''
    required = false

    onChange: (detail: { value: string }) => void
  }

  style host = {type({ family: body })}
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
  const fallback = `sail-radio-${Date.now().toString(36)}`
  const value = signal(String(read(init.value) || ''))
  const name = derived(() => String(read(init.name) || fallback))
  const groupDisabled = derived(() => isOn(init.disabled))
  const required = derived(() => isOn(init.required))
  function select(next) {
    const n = String(next ?? '')
    if (n === value()) return
    value.set(n)
    const cb = init.onChange
    if (typeof cb === 'function') cb({ value: n })
  }
  return { value, groupValue: value, select, name, groupDisabled, required }
})
</controller>

<column gap="2" label={label()} style(host)>
  <slot></slot>
</column>
