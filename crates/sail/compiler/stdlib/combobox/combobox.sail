<!--
  <combobox placeholder="…"> — L1 filtering picker. Text input filters a listbox
  and commits one option. Listbox is CSS-anchor absolute under the field.

  Look: this host owns layout. Listbox owns its Style. Slots, not :part.
  State: this file's <controller> (open + text + active + pick). No DOM.
  Child options take value/label from the use site; the compiler fills `names`
  / `labels` (signal cell, not a snapshot). Listen: use:hotkeys={keys()} with
  allowInInputs, use:clickOutside={onOutside}, native <input> on:input/on:focus
  /pointerdown, native <option> click.
-->

<scope>
  props: {
    placeholder = ''
    value = ''
    onSelect: (detail: { value: string; label: string }) => void
  }

  style host = {display(block).position(relative).size({ w: 100% }).type({ family: body })}
  // Same `--input` surface as input/select/textarea — never bg(background).
  style input = {display(block).size({ min-h: control-h-md }).pad({ y: 1_5, x: 3 }).fg(foreground).bg(inset).border({ color: border, width: 1 }).radius(md).elevation(xs).type({ size: base }), when: {hover: border({ color: border-strong }), focus: border({ style: solid, kind: outline, width: 2, color: ring, offset: 2 }), disabled: cursor(not-allowed).filter({ opacity: 50 })}}
  // w + minW both 100% so the list matches the field (min alone hugs content when options are short).
  style listbox = {display(none).position({ scheme: absolute, inset: auto, anchor: auto, area: bottom, z: popover }).pad(1).margin(0).size({ w: 100%, min-w: 100%, max-h: overlay-max-h-sm }).overflow({ y: auto, overscroll: contain }).fg(foreground).bg(overlay).border({ color: border, width: 1 }).radius(md).elevation(lg)}
  style listShow = {display(block)}
</scope>

<controller>
export default controller((init) => {
  function read(v) {
    return typeof v === 'function' ? v() : v
  }
  const names = Array.isArray(init.names) ? init.names : []
  const labels = Array.isArray(init.labels) ? init.labels : []
  const uid = `sail-cb-${Date.now().toString(36)}`
  const listId = `${uid}-list`
  const items = names.map((value, i) => ({
    value: String(value),
    label: String(labels[i] ?? value),
    id: `${uid}-opt-${i}`,
  }))
  function labelOf(value) {
    const hit = items.find((o) => o.value === value)
    return hit ? hit.label : ''
  }
  const seed = String(read(init.value) ?? '')
  const text = signal(labelOf(seed) || '')
  const committed = signal(seed)
  const open = signal(false)
  const active = signal(seed)
  function shown() {
    const q = text().trim().toLowerCase()
    const sel = committed()
    const selLabel = labelOf(sel)
    if (sel && (q === '' || q === selLabel)) return items
    if (q === '') return items
    return items.filter(
      (o) => o.label.toLowerCase().includes(q) || o.value.toLowerCase().includes(q),
    )
  }
  const visible = derived(() => shown())
  const activeId = derived(() => {
    if (!open()) return ''
    const cur = active()
    const hit = shown().find((o) => o.value === cur)
    return hit ? hit.id : ''
  })
  function fireSelect(value, label) {
    const cb = init.onSelect
    if (typeof cb === 'function') cb({ value, label })
  }
  function pick(value) {
    const hit = items.find((o) => o.value === value) || items.find((o) => o.label === value)
    if (!hit) {
      const label = String(value || '')
      if (!label) return
      text.set(label)
      committed.set(label)
      active.set(label)
      open.set(false)
      fireSelect(label, label)
      return
    }
    text.set(hit.label)
    committed.set(hit.value)
    active.set(hit.value)
    open.set(false)
    fireSelect(hit.value, hit.label)
  }
  function onOption(e) {
    const t = e && e.target
    if (!t) return
    const value = t.value != null && String(t.value) !== '' ? String(t.value) : ''
    const label = String(t.textContent || '').trim()
    pick(value || label)
  }
  function openPicker() {
    const vis = shown()
    if (!vis.length) {
      open.set(false)
      return
    }
    open.set(true)
    const sel = committed()
    const cur = vis.find((o) => o.value === sel)
    active.set(cur ? cur.value : vis[0].value)
  }
  function close() {
    open.set(false)
  }
  function onInput(e) {
    const v = e && e.target && e.target.value != null ? String(e.target.value) : ''
    text.set(v)
    committed.set('')
    const vis = shown()
    if (!vis.length) {
      open.set(false)
      active.set('')
      return
    }
    open.set(true)
    if (!vis.some((o) => o.value === active())) active.set(vis[0].value)
  }
  function onFocus() {
    openPicker()
  }
  function onFieldPointerDown() {
    openPicker()
  }
  function onOutside() {
    if (open()) close()
  }
  function move(delta) {
    const vis = shown()
    if (!vis.length) return
    if (!open()) {
      openPicker()
      if (delta < 0) active.set(vis[vis.length - 1].value)
      else active.set(vis[0].value)
      return
    }
    const i = vis.findIndex((o) => o.value === active())
    const from = i < 0 ? (delta > 0 ? -1 : 0) : i
    const next = (from + delta + vis.length) % vis.length
    active.set(vis[next].value)
  }
  function goEdge(which) {
    const vis = shown()
    if (!open() || !vis.length) return
    active.set(which < 0 ? vis[vis.length - 1].value : vis[0].value)
  }
  function commitActive() {
    if (!open() || !active()) return
    pick(active())
  }
  const keys = derived(() => {
    const allow = (run) => ({ run, allowInInputs: true })
    // Shape is constant; open() only gates the handlers. The hotkeys type
    // gate checks the declared shape — mutating keys in/out confuses it.
    const noop = () => {}
    const map = {
      ArrowDown: allow(() => (open() ? move(1) : noop())),
      ArrowUp: allow(() => (open() ? move(-1) : noop())),
      Escape: allow(() => close()),
      Home: allow(() => (open() ? goEdge(0) : noop())),
      End: allow(() => (open() ? goEdge(-1) : noop())),
      Enter: allow(() => (open() ? commitActive() : noop())),
    }
    return map
  })
  return {
    text,
    open,
    active,
    visible,
    keys,
    listId,
    activeId,
    onInput,
    onFocus,
    onFieldPointerDown,
    onOutside,
    onOption,
    pick,
  }
})
</controller>

<div use:hotkeys={keys()} use:clickOutside={onOutside} style(host)>
  <input
    anatomy
    type="text"
    autocomplete="off"
    placeholder={placeholder()}
    value={text()}
    expanded={open()}
    controls={listId}
    activedescendant={activeId()}
    on:input={onInput}
    on:focus={onFocus}
    on:pointerdown={onFieldPointerDown}
    on:click={onFocus}
    style(input)
  >
  <div id={listId} open={open()} on:click={onOption} style(listbox, {when: {open: listShow}})>
    <slot></slot>
  </div>
</div>
