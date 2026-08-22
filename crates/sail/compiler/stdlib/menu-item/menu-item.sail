<!--
  <menu-item> — item inside dropdown/context-menu. Style product host paint.
  Host stays <button>. Icon+label pack is the inner <row cluster>, not Style gap().

  Look: this tag owns its Style. Slots, not :part.
  State: this file's <controller> (disabled as signal, onSelect). No DOM.
  Listen: native <button> on:click={onSelect}. No item hotkeys
  (arrows stay on the menu host).
-->

<scope>
  props: {
    value = ''
    disabled = false
    onSelect: (detail: { value: string }) => void
  }

  // display(flex) not inline-flex: inline items sit on one line under a
  // max-content menu and the panel width becomes the SUM of labels.
  // padY(2)+padX(3) ≈ 32px hit; icons align via the inner cluster.
  style base = {display(flex).pad({ y: 2, x: 3 }).size({ w: 100% }).border({ width: 0 }).fg(foreground).bg(transparent).radius(sm).cursor(pointer).type({ align: start, family: body, size: sm, leading: tight, wrap: nowrap })}
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
  function onSelect() {
    if (isOn(init.disabled)) return
    const cb = init.onSelect
    if (typeof cb === 'function') cb({ value: String(read(init.value) || '') })
  }
  const disabled = derived(() => isOn(init.disabled))
  return { onSelect, disabled }
})
</controller>

<button
  type="button"
  disabled={disabled()}
  on:click={onSelect}
  style(base, {when: {hover: bg(hover-bg), focus: bg(hover-bg).border({ style: solid, kind: outline, width: 2, color: ring }), disabled: filter({ opacity: 50 }).cursor(not-allowed)}})
>
  <row cluster gap="2_5">
    <slot></slot>
  </row>
</button>
