<!--
  <context-menu for="…"> — L1 right-click menu.
  Look: this host owns Style. Slots, not :part.
  State: this file's <controller> (keys + select). Signals only.
  Listen: use:hotkeys={keys} thunks; native <button> on <menu-item>;
  native [popover] light-dismiss. (use:contextMenu diet)
-->

<scope>
  props: {
    for = ''
    onSelect: (detail: { value: string }) => void
  }

  style host = {position({ scheme: absolute, inset: auto, anchor: auto, area: bottom, z: popover }).pad(1).margin(0).fg(foreground).bg(overlay).border({ color: border, width: 1 }).radius(md).elevation(md)}
</scope>

<controller>
export default controller((init) => {
  function read(v) {
    return typeof v === 'function' ? v() : v
  }
  const nid = `sail-cm-${Math.random().toString(36).slice(2, 10)}`
  function hitFor(e) {
    const id = String(read(init.for) || '')
    if (!id) return false
    const named = hostFindById(id)
    if (!named) return false
    const t = e && e.target
    if (!t) return false
    if (t === named) return true
    return typeof named.contains === 'function' && named.contains(t)
  }
  function panel() {
    return hostFindById(nid)
  }
  function onContext(e) {
    if (e && typeof e.preventDefault === 'function') e.preventDefault()
    const el = (e && e.currentTarget) || panel()
    setTimeout(() => {
      try {
        el.hidePopover()
      } catch {
        /* closed */
      }
      try {
        el.showPopover()
      } catch {
        /* no popover */
      }
    }, 0)
  }
  function onDocContext(e) {
    if (!hitFor(e)) return
    if (e && typeof e.preventDefault === 'function') e.preventDefault()
    const el = panel()
    if (el) hostDispatch(el, 'contextmenu', { bubbles: true, cancelable: true })
  }
  function select(value) {
    const cb = init.onSelect
    if (typeof cb === 'function') cb({ value })
    try {
      panel().hidePopover()
    } catch {
      /* closed */
    }
  }
  const keys = {
    ArrowDown: () => {},
    ArrowUp: () => {},
    Home: () => {},
    End: () => {},
    Escape: () => {
      try {
        panel().hidePopover()
      } catch {
        /* closed */
      }
    },
  }
  onCleanup(hostListenDocument('contextmenu', onDocContext))
  return { keys, onContext, select, nid }
})
</controller>

<div
  popover
  id={nid}
  use:hotkeys={keys}
  on:contextmenu|preventDefault={onContext}
  style(host)
>
  <slot></slot>
</div>
