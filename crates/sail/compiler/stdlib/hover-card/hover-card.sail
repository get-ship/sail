<!--
  <hover-card> — L1 hover-preview overlay. Style value language only.
  Paint from popover / foreground / border; w(max) hugs content (no rail minW).

  Look: panel Style owns paint — no DOM.
  State: this file's <controller> (open/close dwell + placement).
  Listen: use:hotkeys={keys} (Escape) + on:toggle + pointer dwell on trigger/panel.
-->

<scope>
  props: {
    placement = ''
    id = ''
  }

  style panel = {position({ scheme: absolute, inset: auto, anchor: auto, area: bottom, z: popover }).pad(3).margin(0).size({ w: max, max-w: overlay-max-w-sm }).fg(foreground).bg(overlay).border({ color: border, width: 1 }).radius(md).elevation(md).type({ family: body, size: sm }), match: { placement: { top: position({ area: top }), bottom: position({ area: bottom }), left: position({ area: left }), right: position({ area: right }) } }, animate: { enter: play(fade), exit: play(fade) }}
</scope>

<controller>
export default controller((init) => {
  function read(v) {
    return typeof v === 'function' ? v() : v
  }
  const placement = derived(() => String(read(init.placement) ?? ''))
  const genId = `sail-hc-${Math.random().toString(36).slice(2, 10)}`
  const hoverId = derived(() => String(read(init.id) ?? '').trim() || genId)
  function panel() {
    return hostFindById(String(hoverId() ?? ''))
  }
  let openT = 0
  let closeT = 0
  function openMs() {
    const el = panel()
    const v = el?.getAttribute?.('open-delay') ?? ''
    const n = Number(v ?? '')
    return Number.isFinite(n) && n > 0 ? n : v === '0' ? 0 : 200
  }
  function closeMs() {
    const el = panel()
    const v = el?.getAttribute?.('close-delay') ?? ''
    const n = Number(v ?? '')
    return Number.isFinite(n) && n > 0 ? n : v === '0' ? 0 : 150
  }
  const clear = () => {
    if (openT) clearTimeout(openT)
    if (closeT) clearTimeout(closeT)
    openT = closeT = 0
  }
  const show = () => {
    clear()
    const ms = openMs()
    if (ms === 0) { try { panel()?.showPopover?.() } catch {} return }
    openT = Number(setTimeout(() => { try { panel()?.showPopover?.() } catch {} }, ms))
  }
  const hide = () => {
    clear()
    const ms = closeMs()
    if (ms === 0) { try { panel()?.hidePopover?.() } catch {} return }
    closeT = Number(setTimeout(() => { try { panel()?.hidePopover?.() } catch {} }, ms))
  }
  const keys = { Escape: () => { clear(); try { panel()?.hidePopover?.() } catch {} } }
  function onHostEnter() { show() }
  function onHostLeave() { hide() }
  function onPanelEnter() { clear() }
  function onPanelLeave() { hide() }
  function onToggle(e) { if (e && e.newState === 'open') clear() }
  return { placement, hoverId, keys, onToggle, onHostEnter, onHostLeave, onPanelEnter, onPanelLeave }
})
</controller>

<span use:hotkeys={keys} on:mouseenter={onHostEnter} on:mouseleave={onHostLeave} on:focusin={onHostEnter} on:focusout={onHostLeave} style({display(inline-flex).position({ scheme: relative }).self(start).group()})>
  <slot name="trigger"></slot>
  <div popover role="dialog" id={hoverId()} use:hotkeys={keys} on:toggle={onToggle} on:pointerenter={onPanelEnter} on:pointerleave={onPanelLeave} style(panel)><slot></slot></div>
</span>
