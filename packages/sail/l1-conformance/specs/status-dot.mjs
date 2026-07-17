// L1 conformance spec: <span dot> — status-dot (mechanism-2, native span +
// STATUS_DOT_SKIN_CSS). tones + pulse + size. Probe asserts each is a small
// round tone-filled dot.

const dot = (a = '') => `<span dot ${a}></span>`
const TONES = ['default', 'primary', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']

export default {
  tag: 'status-dot',
  width: 640,
  rows() {
    return [
      TONES.map((t) => `<span style="display:inline-flex;align-items:center;gap:6px;margin-right:14px">${dot(`tone="${t}"`)} ${t}</span>`).join(''),
      `<span style="display:inline-flex;align-items:center;gap:6px;margin-right:18px">${dot('tone="success" pulse')} pulse</span><span style="display:inline-flex;align-items:center;gap:6px">${dot('tone="primary" size="sm"')} sm</span>`,
    ]
  },
  probe() {
    const bad = []
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    document.querySelectorAll('span[dot]').forEach((el) => {
      const cs = getComputedStyle(el)
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.width > 24) bad.push({ why: 'bad-size', w: Math.round(r.width) })
      const round = parseFloat(cs.borderRadius) >= r.width / 2 - 1 || cs.borderRadius.includes('%')
      if (!round) bad.push({ why: 'not-round', radius: cs.borderRadius })
      if (clear(cs.backgroundColor)) bad.push({ why: 'no-fill', tone: el.getAttribute('tone') })
    })
    return { count: document.querySelectorAll('span[dot]').length, bad: bad.slice(0, 8) }
  },
}
