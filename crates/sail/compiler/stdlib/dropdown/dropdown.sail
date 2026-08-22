<!--
  <dropdown id="…" placement="bottom-start"> — L1 anchored menu (element mode).
  Native [popover] + CSS anchor geometry. Compose <menu-item>s.
  Paint from popover / foreground / border globals + anchor geometry.
  Backdrop transparent is global :where([popover])::backdrop.

  Look: this host owns Style (placement match). Slots, not :part.
  State: this file's <controller> (select + close + keys). Signals only.
  Listen: use:hotkeys={keys} (Escape closes); native <button> items.
  Slotted <menu-item>s inherit this scope's onSelect: activating an item
  fires { value } and closes. Open/close + light-dismiss stay the native
  popover's (trigger wires popovertarget). Roving arrow focus died with
  the old menu action — items are native buttons (Tab reaches them);
  context-menu set the same precedent.
-->

<scope>
  props: {
    placement = ''
    onSelect: (detail: { value: string }) => void
  }

  // w(max)=max-content hugs the longest item (no minW floor). pad(1_5) insets items.
  style host = {position({ scheme: absolute, inset: auto, anchor: auto, area: bottom, z: popover }).pad(1_5).margin(0).size({ w: max }).fg(foreground).bg(background).border({ color: border, width: 1 }).radius(md).elevation(md), match: { placement: { top: position({ area: top }), bottom: position({ area: bottom }), left: position({ area: left }), right: position({ area: right }), 'top-end': position({ area: top-end }), 'bottom-end': position({ area: bottom-end }), 'left-end': position({ area: left-end }), 'right-end': position({ area: right-end }) } }, animate: { enter: play(fade), exit: play(fade) }}
</scope>

<controller>
export default controller((init) => {
  function read(v) {
    return typeof v === 'function' ? v() : v
  }
  const nid = `sail-dd-${Math.random().toString(36).slice(2, 10)}`
  function panel() {
    return hostFindById(nid)
  }
  function close() {
    const el = panel()
    if (!el || typeof el.hidePopover !== 'function') return
    try {
      el.hidePopover()
    } catch {
      /* already closed */
    }
  }
  function onSelect(detail) {
    const cb = init.onSelect
    if (typeof cb === 'function') cb(detail)
    close()
  }
  const keys = {
    Escape: close,
  }
  return { onSelect, keys, nid }
})
</controller>

<div
  popover
  id={nid}
  use:hotkeys={keys}
  style(host)
>
  <slot></slot>
</div>
