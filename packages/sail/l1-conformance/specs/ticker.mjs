// L1 conformance spec: <ticker> (mechanism-1, compiles-away + use:marquee).
// Host is geometry only (overflow clip). duration · reverse are PE axes.
// No component.ticker.* — author paint. Probe: host renders, clips overflow,
// and duration/reverse attrs are present across the matrix.

const band =
  '<span style="display:inline-block;padding:0 1rem;white-space:nowrap;font-family:var(--font-mono);font-size:var(--text-sm);color:var(--muted-foreground)">tokens · signals · a11y · hot reload · SSR ·&nbsp;</span>'

// Lowered host classes from stdlib ticker.sail (overflow clip geometry).
const HOST = 'sail-ticker s-overflowhidden s-wfull s-minW0'

const ticker = (attrs = '') =>
  `<div class="${HOST}" ${attrs} style="width:280px;padding:12px 0;background:var(--muted);border-radius:var(--radius-md)">${band}</div>`

export default {
  tag: 'ticker',
  width: 640,
  rows() {
    return [
      // default / rest — duration=normal, reverse off + full matrix
      ticker('class="l1-mark" duration="normal"') +
        ticker('duration="fast"') +
        ticker('duration="slow"') +
        ticker('duration="normal" reverse') +
        ticker('duration="fast" reverse') +
        ticker('duration="slow" reverse'),
    ]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sail-ticker').forEach((el, i) => {
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) bad.push({ i, why: 'not-rendered' })
      const cs = getComputedStyle(el)
      if (!/(hidden|clip)/.test(cs.overflowX) && !/(hidden|clip)/.test(cs.overflow)) {
        bad.push({ i, why: 'not-clipping', overflowX: cs.overflowX, overflow: cs.overflow })
      }
      const d = el.getAttribute('duration')
      if (d && !['fast', 'normal', 'slow'].includes(d)) {
        bad.push({ i, why: 'bad-duration', d })
      }
    })
    const hasReverse = document.querySelector('.sail-ticker[reverse]')
    if (!hasReverse) bad.push({ why: 'missing-reverse-row' })
    const durations = new Set(
      [...document.querySelectorAll('.sail-ticker')].map((el) => el.getAttribute('duration')),
    )
    for (const d of ['fast', 'normal', 'slow']) {
      if (!durations.has(d)) bad.push({ why: 'missing-duration', d })
    }
    return { count: document.querySelectorAll('.sail-ticker').length, bad: bad.slice(0, 8) }
  },
}
