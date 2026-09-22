<!--
  <drawer> — edge-anchored non-modal sheet. Style product only.
  Native <dialog> sheet. Openers wire invoker command="show-modal"
  commandfor=<panel-id> — browsers ship no native default for "show",
  so non-modal open needs author-supplied JS (see site docs polyfill).
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
    /** Authored panel id — invoker `commandfor` targets the inner dialog. */
    panelId = ''
    onClose: () => void
    onDismiss: () => void
  }

  // Shell only. Edge dock + header/section/footer live in platform/drawer.css
  // (same native-<dialog> adaptor as dialog). inset:0 + w/h full made a
  // full-viewport card — that is not a drawer.
  style host = {position({ scheme: fixed, z: dialog }).pad(0).margin(0).overflow({ y: auto, overscroll: contain }).border({ color: border, width: 1 }).radius(none).bg(overlay).fg(foreground).elevation(xl).type({ family: body }), animate: { transition: play(surface, fast), enter: play(fade), exit: play(fade) }}
</scope>

<controller>
export default controller((init) => {
  let lastEl: unknown = null
  function read(v) { return typeof v === 'function' ? v() : v }
  function isOn(v) { const x = read(v); return x === true || x === '' || x === 'true' }
  const dismissible = derived(() => isOn(init.dismissible) ? true : String(read(init.dismissible) ?? '') !== 'false')
  const side = derived(() => String(read(init.side) ?? 'end'))
  // Authored `id` must reach the inner <dialog>: invoker buttons wire
  // `commandfor` to it, and a wrapper-path component does not forward
  // host attrs to its template root on its own.
  const genId = `sail-drawer-${Math.random().toString(36).slice(2, 10)}`
  const dialogId = derived(() => String(read(init.panelId) ?? '').trim() || genId)
  function closeFrom(el: unknown) {
    if (!el) return
    const box = el as { open?: boolean; close?: () => void; removeAttribute?: (n: string) => void }
    if (box.open && typeof box.close === 'function') box.close()
    else if (typeof box.removeAttribute === 'function') box.removeAttribute('open')
    const cb = init.onClose as unknown
    if (typeof cb === 'function') (cb as () => void)()
    else {
      const cb2 = init.onDismiss as unknown
      if (typeof cb2 === 'function') (cb2 as () => void)()
    }
  }
  function panel() {
    return hostFindById(String(dialogId() ?? ''))
  }
  // Close the panel by id — lastEl is only set after an inside click, so a
  // fresh-open Escape must not depend on it.
  function onKeydown(e) {
    if (!dismissible() || e.key !== 'Escape') return
    e.preventDefault()
    const box = panel() as { open?: boolean; close?: () => void } | null
    if (box?.open && typeof box.close === 'function') box.close()
  }
  function onClick(e) {
    if (!dismissible()) return
    const el = (e as { currentTarget?: unknown }).currentTarget
    if (!el || e.target !== el) return
    lastEl = el
    closeFrom(el)
  }
  let startX = 0, startY = 0, active = false
  function onPointerdown(e) {
    if (!dismissible() || e.button !== 0) return
    active = true
    startX = e.clientX
    startY = e.clientY
    lastEl = (e as { currentTarget?: unknown }).currentTarget
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
    if (ok) closeFrom(lastEl)
  }
  return { onKeydown, onClick, onPointerdown, onPointerup, dialogId }
})
</controller>

<dialog
  id={dialogId()}
  on:keydown={onKeydown}
  on:click={onClick}
  on:pointerdown={onPointerdown}
  on:pointerup={onPointerup}
  style(host, {match: {size: { sm: {size({ max-w: panel-max-w-sm })}, md: {size({ max-w: panel-max-w-md })}, lg: {size({ max-w: panel-max-w-lg })} }, side: {start: {touch(pan-y)}, end: {touch(pan-y)}, top: {touch(pan-x)}, bottom: {touch(pan-x)}}}})
>
  <slot></slot>
</dialog>
