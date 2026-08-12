// L1 conformance spec: <tooltip> (mechanism-1, use:tipDescribe).
// visual.cross: placement · content.
// Bubble needs Style stamps for token paint when force-shown.

const HOST = 'sail-tooltip s-positionrelative'
const BUBBLE =
  's-positionabsolute s-padY1 s-padX2 s-zpopover s-bgtooltip s-fgtooltipforeground s-radiusmd s-shadowmd s-pointerEventsnone s-borderWidth0 s-fontFamilybody s-fontSizexs s-fontWeightmedium s-whitespacenowrap'

const tt = (place, label) =>
  `<span class="${HOST}" data-sail-host="tooltip" placement="${place}" content="Tip: ${label}"><button style="padding:6px 12px;border:1px solid var(--border);border-radius:6px;background:var(--surface-1);color:var(--foreground)">${label}</button><span role="tooltip" class="${BUBBLE}"></span></span>`

export default {
  tag: 'tooltip',
  width: 760,
  rows() {
    return [
      ['top', 'bottom', 'left', 'right'].map((p) => tt(p, p)).join('&nbsp;&nbsp;&nbsp;&nbsp;'),
      tt('top', 'Copy URL'),
    ]
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
      bubble.style.position = 'static' // in-flow for paint assert in grid
      const textOk = (bubble.textContent || '').trim().length > 0
      const ariaOk = !!trig && trig.getAttribute('aria-describedby') === bubble.id
      if (!textOk || !ariaOk) bad.push({ textOk, ariaOk, content: el.getAttribute('content') })
      const cs = getComputedStyle(bubble)
      if (cs.backgroundColor === 'rgba(0, 0, 0, 0)' || cs.backgroundColor === 'transparent') {
        bad.push({ id, why: 'no-token-bg' })
      }
      if (!cs.color || cs.color === 'rgba(0, 0, 0, 0)') bad.push({ id, why: 'no-token-fg' })
    })
    // content coverage
    if (![...document.querySelectorAll('.sail-tooltip')].some((e) => (e.getAttribute('content') || '').includes('Copy')))
      bad.push({ why: 'content-missing' })
    return { count: document.querySelectorAll('.sail-tooltip').length, bad: bad.slice(0, 12) }
  },
}
