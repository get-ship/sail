<!--
  <page-item> — size match owns cell geometry (author size on each item).
  Paint from foreground / hover-bg / muted.

  State: `page` + `select` come from the parent <pagination> controller as
  props (signal cell, not a snapshot). This controller computes `current`
  (stamps aria-current="page", which also drives the when { current } paint)
  and the click → select wiring. Click is the platform <button>; native
  disabled from the prop stamp.

  variant=simple: denser cell (smaller pad, no border) for pagination simple.
-->

<scope>
  props: {
    value = ''
    disabled = false
    size: 'md' | 'sm' | 'lg' = 'md'
    variant: 'default' | 'simple' = 'default'
  }

  style base = {pad({ x: 2 }).fg(foreground).bg(transparent).radius(md).border({ color: transparent, width: 1 }).cursor(pointer).select(none).type({ family: body, weight: medium }), animate: { transition: play(colors, fast) }}
  style sizeMd = {size({ min-w: control-h-md, h: control-h-md }).type({ size: sm })}
  style sizeSm = {size({ min-w: control-h-sm, h: control-h-sm }).type({ size: xs })}
  style sizeLg = {size({ min-w: control-h-lg, h: control-h-lg }).type({ size: base })}
  style simple = {pad({ x: 1 }).border({ width: 0 }).radius(sm)}
</scope>

<controller>
export default controller((init) => {
  function isOn(v) {
    const x = typeof v === 'function' ? v() : v
    return x === true || x === '' || x === 'true'
  }
  const value = String(init.value ?? '')
  const disabled = derived(() => isOn(init.disabled))
  const current = derived(() => {
    const page = typeof init.page === 'function' ? init.page() : ''
    return page === value ? 'page' : ''
  })
  function onSelect() {
    if (disabled()) return
    if (typeof init.select === 'function') init.select(value)
  }
  return { current, onSelect, disabled }
})
</controller>

<button
  type="button"
  on:click={onSelect}
  current={current()}
  disabled={disabled()}
  style(base, {when: {hover: bg(hover-bg), current: bg(muted).fg(foreground).border({ color: muted }), disabled: filter({ opacity: 50 }).cursor(not-allowed), focus: border({ style: solid, kind: outline, width: 2, color: ring })}, match: { size: { md: sizeMd, sm: sizeSm, lg: sizeLg }, variant: { default: pad({ x: 2 }), simple: simple } }})
>
  <slot></slot>
</button>