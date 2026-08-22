<!--
  <slider> — native range. Single thumb by default.
  range=true: two thumbs on one track (`<stack>` — overlay is the product).
  ticks: named labels in a flow row under the track.
  Empty tick row uses HTML `hidden` so `.sail-row { display:flex }` cannot
  win the cascade. Track/thumb vendor pseudos: platform/slider.css.

  Look: host/track Style owns paint — no DOM.
  State: this file's <controller> (value/valueEnd/min/max/step/disabled/range, clamping).
  Listen: on:input on each thumb writes value/valueEnd.
-->

<scope>
  props: {
    value = 0
    'value-end' = 100
    min = 0
    max = 100
    step = 1
    disabled = false
    range = false
    size: 'sm' | 'md' | 'lg' = 'md'
    name = ''
    ticks = ''
  }

  style host = {size({ w: 100%, min-w: 0 })}
  style track = {position(relative).size({ w: 100%, h: 5 })}
  style sizeSm = {size({ h: 4 })}
  style sizeMd = {size({ h: 5 })}
  style sizeLg = {size({ h: 6 })}
  style base = {size({ w: 100%, h: 100% }).appearance(none).bg(transparent).cursor(pointer), when: {disabled: cursor(not-allowed).filter({ opacity: 50 })}}
  style thumbOn = {display(block)}
  style thumbOff = {display(none)}
  style tickRow = {size({ w: 100%, min-w: 0 }).pad({ t: 2 }).fg(muted-foreground).type({ size: xs, family: mono, wrap: nowrap })}
</scope>

<controller>
export default controller((init) => {
  function read(v) {
    return typeof v === 'function' ? v() : v
  }
  function num(name, fb) {
    const raw = read(init[name])
    const n = parseFloat(raw ?? '')
    return Number.isFinite(n) ? n : fb
  }
  function isOn(v) {
    const x = read(v)
    return x === true || x === '' || x === 'true'
  }
  // Snapshot props at mount — host attrs re-sync via page store if needed.
  const min = String(read(init.min) ?? '0')
  const max = String(read(init.max) ?? '100')
  const step = String(read(init.step) ?? '1')
  const disabled = isOn(init.disabled)
  const ranged = isOn(init.range)
  const start = signal(num('value', Number(min)))
  const end = signal(num('value-end', Number(max)))
  const tickLabels = String(read(init.ticks) ?? '').split(',').map(s => s.trim()).filter(Boolean)
  function onInputA(e) {
    const v = e?.target?.value ?? String(start())
    start.set(Number(v))
  }
  function onInputB(e) {
    const v = e?.target?.value ?? String(end())
    end.set(Number(v))
  }
  return { min, max, step, disabled, ranged, start, end, tickLabels, onInputA, onInputB }
})
</controller>

<column gap="0" style(host)>
  <stack style(track, { match: { size: { sm: sizeSm, md: sizeMd, lg: sizeLg } } })>
    <input type="range" min={min} max={max} step={step} disabled={disabled} value={start()} on:input={onInputA} style(base)>
    <input type="range" min={min} max={max} step={step} disabled={disabled} value={end()} on:input={onInputB} style(base, { match: { range: { true: thumbOn, false: thumbOff } } })>
  </stack>
  <row nowrap justify="between" gap="1" hidden="hidden" data-slider-ticks style(tickRow)><for {tickLabels as label}><span>{label}</span></for></row>
</column>
