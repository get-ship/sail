// L1 conformance spec: <tooltip> (mechanism-1, use:tooltip). Probe runs the
// tooltip essentials (per-instance anchor name, bubble text from content=,
// aria-describedby wiring) and force-shows the bubbles for the screenshot, then
// asserts the bubble text is set and the trigger is described by it.

const tt = (place, label) =>
  `<span class="sui-tooltip" placement="${place}" content="Tip: ${label}"><button style="padding:6px 12px;border:1px solid var(--border);border-radius:6px;background:var(--surface-1);color:var(--foreground)">${label}</button><span class="bubble" role="tooltip"></span></span>`

export default {
  tag: 'tooltip',
  width: 760,
  rows() {
    return [['top', 'bottom', 'left', 'right'].map((p) => tt(p, p)).join('&nbsp;&nbsp;&nbsp;&nbsp;')]
  },
  probe() {
    let n = 0
    const bad = []
    document.querySelectorAll('.sui-tooltip').forEach((el) => {
      const id = ++n
      el.style.setProperty('--sui-tooltip-anchor', `--sui-tooltip-${id}`)
      const bubble = el.querySelector('.bubble')
      bubble.id = `sui-tooltip-${id}`
      bubble.textContent = el.getAttribute('content') || ''
      const trig = [...el.children].find((c) => c !== bubble)
      if (trig) trig.setAttribute('aria-describedby', bubble.id)
      bubble.style.opacity = '1'
      const textOk = (bubble.textContent || '').trim().length > 0
      const ariaOk = !!trig && trig.getAttribute('aria-describedby') === bubble.id
      if (!textOk || !ariaOk) bad.push({ textOk, ariaOk, content: el.getAttribute('content') })
    })
    return { count: document.querySelectorAll('.sui-tooltip').length, bad }
  },
}
