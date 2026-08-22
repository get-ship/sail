<!--
  <drawer> — edge-anchored non-modal sheet. Style product only.
  Native <dialog> without showModal — invoker command="show" commandfor.
  side start|end|top|bottom (default end). size sm|md|lg. dismissible default true.
  Light-dom children: <header> / <section> / <footer> (same as dialog, not slots).

  Look: host Style owns paint — no DOM.
  State: this file's <controller> (open/close + dismissible).
  Listen: on:keydown (Escape) + on:click (light-dismiss) + swipe via on:pointer*.
-->

<scope>
  props: {
    size: 'sm' | 'md' | 'lg' = 'md'
    side: 'start' | 'end' | 'top' | 'bottom' = 'end'
    /** Escape, outside pointer, and swipe-toward-edge can close. */
    dismissible = true
    onClose: () => void
    onDismiss: () => void
  }

  // Shell only. Edge dock + header/section/footer live in platform/drawer.css
  // (same native-<dialog> adaptor as dialog). inset:0 + w/h full made a
  // full-viewport card — that is not a drawer.
  style host = {position({ scheme: fixed, z: dialog }).pad(0).margin(0).overflow({ y: auto, overscroll: contain }).border({ color: border, width: 1 }).radius(none).bg(overlay).fg(foreground).elevation(xl).type({ family: body }), animate: { transition: play(surface, fast), enter: play(fade), exit: play(fade) }}
  style shown = {display(flex).dir(column)}
  style sizeSm = {size({ max-w: overlay-max-w-sm, max-h: overlay-vh-sm })}
  style sizeMd = {size({ max-w: overlay-max-w-md, max-h: overlay-vh-md })}
  style sizeLg = {size({ max-w: overlay-max-w-lg, max-h: overlay-vh-lg })}
  style sideStart = {touch(pan-y)}
  style sideEnd = {touch(pan-y)}
  style sideTop = {touch(pan-x)}
  style sideBottom = {touch(pan-x)}
</scope>

<controller>
export default controller((init, host) => {
  function read(v) { return typeof v === 'function' ? v() : v }
  function isOn(v) { const x = read(v); return x === true || x === '' || x === 'true' }
  const dismissible = derived(() => isOn(init.dismissible) ? true : String(read(init.dismissible) ?? '') !== 'false')
  const side = derived(() => String(read(init.side) ?? 'end'))
  function close() {
    const el = (host as unknown as HTMLElement | null) ?? null
    if (!el) return
    if (el instanceof HTMLDialogElement) { if (el.open) el.close(); return }
    if (el.hasAttribute('open')) el.removeAttribute('open')
    const cb = init.onClose as unknown
    if (typeof cb === 'function') (cb as () => void)()
    else {
      const cb2 = init.onDismiss as unknown
      if (typeof cb2 === 'function') (cb2 as () => void)()
    }
  }
  function onKeydown(e) {
    if (!dismissible() || e.key !== 'Escape') return
    e.preventDefault()
    close()
  }
  function onClick(e) {
    if (!dismissible()) return
    const el = (host as unknown as HTMLElement | null) ?? null
    if (!el || e.target !== el) return
    close()
  }
  let startX = 0, startY = 0, active = false
  function onPointerdown(e) {
    if (!dismissible() || e.button !== 0) return
    active = true; startX = e.clientX; startY = e.clientY
  }
  function onPointerup(e) {
    if (!active) return; active = false
    const dx = e.clientX - startX, dy = e.clientY - startY
    const s = side()
    const isH = s === 'start' || s === 'end'
    const dist = isH ? dx : dy
    const threshold = 80
    const needNeg = s === 'start' || s === 'top'
    const ok = needNeg ? dist < -threshold : dist > threshold
    if (ok) close()
  }
  return { onKeydown, onClick, onPointerdown, onPointerup }
})
</controller>

<dialog
  on:keydown={onKeydown}
  on:click={onClick}
  on:pointerdown={onPointerdown}
  on:pointerup={onPointerup}
  style(host, {when: { open: shown }, match: {size: { sm: sizeSm, md: sizeMd, lg: sizeLg }, side: {start: sideStart, end: sideEnd, top: sideTop, bottom: sideBottom}}})
>
  <slot></slot>
</dialog>
