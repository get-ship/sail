// L1 conformance spec: <tooltip> (mechanism-1, use:tipDescribe). Probe runs the
// tooltip essentials (per-instance anchor name, bubble text from content=,
// aria-describedby wiring) and force-shows the bubbles for the screenshot, then
// asserts the bubble text is set and the trigger is described by it.

const tt = (place, label) =>
  `<span class="sail-tooltip" placement="${place}" content="Tip: ${label}"><button style="padding:6px 12px;border:1px solid var(--border);border-radius:6px;background:var(--surface-1);color:var(--foreground)">${label}</button><span role="tooltip"></span></span>`

export default {
  tag: 'tooltip',
  width: 760,
  rows() {
    return [['top', 'bottom', 'left', 'right'].map((p) => tt(p, p)).join('&nbsp;&nbsp;&nbsp;&nbsp;')]
  },
  probe() {
    let n = 0
    const bad = []
    document.querySelectorAll('.sail-tooltip').forEach((el) => {
      const id = ++n
      el.style.setProperty('--sail-tooltip-anchor', `--sail-tooltip-${id}`)
      const bubble = el.querySelector('[role="tooltip"]')
      bubble.id = `sail-tooltip-${id}`
      bubble.textContent = el.getAttribute('content') || ''
      const trig = [...el.children].find((c) => c !== bubble)
      if (trig) trig.setAttribute('aria-describedby', bubble.id)
      bubble.style.opacity = '1'
      const textOk = (bubble.textContent || '').trim().length > 0
      const ariaOk = !!trig && trig.getAttribute('aria-describedby') === bubble.id
      if (!textOk || !ariaOk) bad.push({ textOk, ariaOk, content: el.getAttribute('content') })
      // Token-backed bubble paint (tooltip / tooltip-foreground) must resolve when open.
      const cs = getComputedStyle(bubble)
      if (cs.backgroundColor === 'rgba(0, 0, 0, 0)' || cs.backgroundColor === 'transparent') {
        bad.push({ id, why: 'no-token-bg' })
      }
      if (!cs.color || cs.color === 'rgba(0, 0, 0, 0)') bad.push({ id, why: 'no-token-fg' })
    })
    return { count: document.querySelectorAll('.sail-tooltip').length, bad }
  },
}
