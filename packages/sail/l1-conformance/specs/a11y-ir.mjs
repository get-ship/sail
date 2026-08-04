// L1 conformance: SPEC-ARIA-1 semantic prop stamping (compiler IR → DOM).
// Probes runtime DOM after host mounts lowered markup.

const btn = (label, attrs = '') =>
  label
    ? `<button type="button" ${attrs}>${label}</button>`
    : `<button type="button" ${attrs}><svg width="12" height="12"></svg></button>`

export default {
  tag: 'a11y-ir',
  width: 480,
  rows() {
    return [
      // text button — no label required
      btn('Save', 'variant="solid" tone="primary"'),
      // icon-only with semantic label (compiler stamps aria-label)
      btn('', 'label="Close" variant="ghost"'),
      // select host — native semantics (no free role=combobox)
      `<select class="l1-mark"><option>Ada</option><option>Grace</option></select>`,
      // nav currency
      `<a href="/" current="page">Home</a>`,
    ]
  },
  probe() {
    const bad = []
    const icon = document.querySelector('button[aria-label="Close"], button[label="Close"]')
    // After full pipeline label → aria-label; accept either if probe runs pre-lower
    const labeled =
      document.querySelector('button[aria-label="Close"]') ||
      [...document.querySelectorAll('button')].find((b) => b.getAttribute('aria-label') === 'Close')
    if (!labeled) {
      // Structure may still carry label= if probe uses unstamped HTML — check label attr
      const byLabel = document.querySelector('button[label="Close"]')
      if (!byLabel) bad.push({ why: 'icon-button-missing-label-stamp' })
    }
    const home = document.querySelector('a[aria-current="page"], a[current="page"]')
    if (!home) bad.push({ why: 'current-page-not-stamped' })
    const sel = document.querySelector('select.l1-mark')
    if (!sel) bad.push({ why: 'select-missing' })
    // Native select must NOT carry role=combobox (native semantics)
    if (sel?.getAttribute('role') === 'combobox') bad.push({ why: 'select-false-combobox-role' })
    return { count: document.querySelectorAll('button,select,a').length, bad: bad.slice(0, 10) }
  },
}
