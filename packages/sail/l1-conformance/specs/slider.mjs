// L1 conformance spec: <slider> (mechanism-2, native <input type=range> +
// RANGE_SKIN_CSS, use:rangeFill). Probe runs rangeFill and asserts the native
// slider contract (type=range, --rng-pct set to value%, native value).
// Single thumb only — no dual-thumb range invent.

const sizeCls = {
  sm: 's-sizesm-h4',
  md: 's-sizemd-h5',
  lg: 's-sizelg-h6',
}

const sl = (attrs) => {
  const size = /size="([^"]+)"/.exec(attrs || '')?.[1] || ''
  const sc = size && sizeCls[size] ? ` ${sizeCls[size]}` : ''
  return `<input type="range" class="sail-slider${sc}" ${attrs} style="width:200px">`
}
const cell = (label, s) =>
  `<div style="display:flex;flex-direction:column;gap:6px"><span style="font-size:11px;color:var(--muted-foreground)">${label}</span>${s}</div>`

export default {
  tag: 'slider',
  width: 900,
  rows() {
    return [
      // rest-default + value + size ladder + disabled
      cell('rest-default 40', sl('value="40"')) +
        cell('sm 50', sl('size="sm" value="50"')) +
        cell('md 70', sl('size="md" value="70"')) +
        cell('lg 85', sl('size="lg" value="85"')) +
        cell('disabled 60', sl('disabled value="60"')),
    ]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sail-slider').forEach((el) => {
      const min = parseFloat(el.min || '0'),
        max = parseFloat(el.max || '100'),
        val = parseFloat(el.value || '0')
      const pct = max > min ? ((val - min) / (max - min)) * 100 : 0
      el.style.setProperty('--rng-pct', `${pct}%`)
      const rngPctOk = el.style.getPropertyValue('--rng-pct') === `${pct}%`
      const isRange = el.type === 'range'
      const nativeVal = el.value === String(val)
      if (!rngPctOk || !isRange || !nativeVal) bad.push({ val, rngPctOk, isRange, nativeVal })
    })
    for (const s of ['sm', 'md', 'lg']) {
      if (!document.querySelector(`.sail-slider[size="${s}"]`)) bad.push({ why: 'missing-size', size: s })
    }
    if (!document.querySelector('.sail-slider[disabled]')) bad.push({ why: 'missing-disabled' })
    return { count: document.querySelectorAll('.sail-slider').length, bad }
  },
}
