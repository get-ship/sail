<!--
  <tooltip> — hover/focus text bubble. Style value language only.
  Arrow is a border-colored caret (not a rotated square) so one edge is not a
  double/broken seam against the bubble fill + shadow.

  Look: host + bubble own Style — no DOM.
  State: this file's <controller> (bubbleId + content + hidden).
  Listen: use:hotkeys={keys} (Escape) + pointer/focus on host.
-->

<scope>
  props: {
    placement: 'top' | 'bottom' | 'left' | 'right' = 'top'
    content = ''
  }

  style host = {position({ scheme: relative, name: tooltip }).display(inline-flex).self(start).group()}
  style hide = {filter({ opacity: 0 })}
  style show = {filter({ opacity: 100 }), match: { 'data-hidden': { true: hide } }}
  style bubble = {position({ scheme: absolute, anchor: tooltip, area: top, z: popover }).pad({ y: 1, x: 2 }).size({ w: max, min-w: max }).overflow({ all: visible }).filter({ opacity: 0 }).bg(overlay).fg(foreground).radius(md).elevation(md).border({ width: 0 }).pointerEvents(none).type({ family: body, size: xs, weight: medium, wrap: nowrap }), match: { placement: { top: position({ area: top }), bottom: position({ area: bottom }), left: position({ area: left }), right: position({ area: right }) } }, animate: { enter: play(fade), exit: play(fade) }}
  style arrowBox = {position({ scheme: absolute, b: auto }).size({ w: 0, h: 0 }).content(empty).bg(transparent).border({ width: 0, style: solid })}
  style arrowFromTop = {border({ width: 4, edge: l, color: transparent }).border({ width: 4, edge: r, color: transparent }).border({ width: 4, edge: t, color: overlay }).border({ width: 0, edge: b, color: transparent })}
  style arrowFromBottom = {border({ width: 4, edge: l, color: transparent }).border({ width: 4, edge: r, color: transparent }).border({ width: 4, edge: b, color: overlay }).border({ width: 0, edge: t, color: transparent })}
  style arrowFromLeft = {border({ width: 4, edge: t, color: transparent }).border({ width: 4, edge: b, color: transparent }).border({ width: 4, edge: l, color: overlay }).border({ width: 0, edge: r, color: transparent })}
  style arrowFromRight = {border({ width: 4, edge: t, color: transparent }).border({ width: 4, edge: b, color: transparent }).border({ width: 4, edge: r, color: overlay }).border({ width: 0, edge: l, color: transparent })}
</scope>

<controller>
export default controller((init) => {
  function read(v) { return typeof v === 'function' ? v() : v }
  const genId = `sail-tooltip-${Math.random().toString(36).slice(2, 10)}`
  const bubbleId = derived(() => String(read(init.id) ?? '').trim() || genId)
  const content = derived(() => String(read(init.content) ?? ''))
  const hidden = signal(false)
  function clearHidden() { hidden.set(false) }
  function forceHidden() { hidden.set(true) }
  const keys = { Escape: () => { forceHidden() } }
  function onMouseenter() { clearHidden() }
  function onMouseleave() { forceHidden() }
  function onFocusin() { clearHidden() }
  function onFocusout(e) {
    const next = e?.relatedTarget as Node | null
    const host = e?.currentTarget as HTMLElement | null
    if (next && host?.contains(next)) return
    forceHidden()
  }
  return { bubbleId, content, hidden, keys, onMouseenter, onMouseleave, onFocusin, onFocusout }
})
</controller>

<span use:hotkeys={keys} on:mouseenter={onMouseenter} on:mouseleave={onMouseleave} on:focusin={onFocusin} on:focusout={onFocusout} style(host)>
  <slot></slot>
  <span id={bubbleId()} role="tooltip" data-tooltip-bubble data-hidden={hidden()} style(bubble)>{content()}</span>
</span>
