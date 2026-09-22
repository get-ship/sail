<!--
  <radio> — label + native radio. Root stays <label> for native semantics.
  Host label ink from foreground. Control face
  aligns with <checkbox> (foreground/background ring, primary when checked).
  value + disabled are host props. Parent `groupValue` / `select` / `name`
  are compiler-filled props (signal cell, not a snapshot).
  Listen: native <input type=radio> (shared name: exclusive check, arrows, form).
-->

<scope>
  props: {
    value = ''
    disabled = false
  }

  // Host when:disabled does not paint — :disabled/:aria-disabled only match form
  // controls. Input when:disabled paints from disabled={disabled()}.
  // textCase(normal) → CSS none: host is a <label>; type-label-case must not scream.
  style host = {fg(foreground).cursor(pointer).select(none).type({ family: body, size: sm, weight: normal, case: normal, tracking: normal })}
  // Checked disc via RADIO_INPUT_MAP (empty face + primary dot — native, no SVG).
  style input = {display(inline-block).shrink(0).pad(0).margin(0).size({ min-h: 0, w: icon-size-md, h: icon-size-md }).appearance(none).align(middle).border({ color: border, width: 1, style: solid }).radius(full).cursor(pointer).boxSizing(border), when: {disabled: cursor(not-allowed).filter({ opacity: 50 })}}
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
  const value = derived(() => String(read(init.value) || ''))
  const name = derived(() => String(read(init.name) || ''))
  const required = derived(() => isOn(init.required))
  const disabled = derived(() => isOn(init.disabled) || isOn(init.groupDisabled))
  const checked = derived(() => {
    const selected = init.groupValue
    const mine = value()
    return !!(typeof selected === 'function' && mine && selected() === mine)
  })
  function onPick() {
    if (disabled()) return
    const mine = value()
    if (typeof init.select === 'function') init.select(mine)
  }
  return { onPick, checked, disabled, name, required, value }
})
</controller>

<label style(host)>
  <input
    type="radio"
    name={name()}
    value={value()}
    checked={checked()}
    disabled={disabled()}
    required={required()}
    on:click={onPick}
    on:change={onPick}
    style(input, {shrink(0)})
  >
  <span><slot></slot></span>
</label>
