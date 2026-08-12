// L1 conformance spec: <card> (mechanism-1, compiles-away).
// visual.cross: tone · variant · padding · interactive · accent-bar · status-rail · fill.
// Lowered Style stamps — dual-gate border needs s-bordersolid.

const HOST =
  'sail-card s-positionrelative s-overflowclip s-padY4 s-padX5 s-isolate s-radiuslg s-border1border s-bordersolid s-fgcardforeground s-shadowcardrest s-transitioncolors s-durationfast s-easeout s-bgcard s-backdropcardbackdropfilter s-borderImagecornerimage'

const padCls = (padding) => {
  if (padding === 'none') return ' s-paddingnone-pad0'
  if (padding === 'sm') return ' s-paddingsm-padY2 s-paddingsm-padX3'
  if (padding === 'lg') return ' s-paddinglg-padY5 s-paddinglg-padX6'
  return ' s-paddingmd-padY4 s-paddingmd-padX5' // md default
}
const card = (a = '', content = 'Card content', extraCls = '') => {
  const m = a.match(/padding=["']([^"']+)["']/)
  const p = m ? m[1] : 'md'
  // strip base pad when match pad is applied so monotonic probe works
  const base = HOST.replace(' s-padY4 s-padX5', '')
  return `<div class="${base}${padCls(p)}${extraCls ? ` ${extraCls}` : ''}" data-sail-host="card" ${a} style="width:190px">${content}</div>`
}

export default {
  tag: 'card',
  width: 760,
  rows() {
    return [
      card('padding="md"', 'Rest md', 'l1-mark') +
        card('padding="none"', 'none') +
        card('padding="sm"', 'sm') +
        card('padding="lg"', 'lg'),
      card('variant="default"', 'default') +
        card('variant="outline"', 'outline') +
        card('fill', 'fill') +
        card('interactive', 'interactive') +
        card('accent-bar="off"', 'bar off') +
        card('accent-bar="hover" tone="primary"', 'bar hover') +
        card('accent-bar="static" tone="primary"', 'bar static') +
        card('status-rail', 'status-rail'),
      card('tone="primary"', 'primary') +
        card('tone="info"', 'info') +
        card('tone="success"', 'success') +
        card('tone="warning"', 'warning') +
        card('tone="destructive"', 'destructive') +
        card('tone="accent"', 'accent') +
        card('tone="muted"', 'muted'),
    ]
  },
  probe() {
    const bad = []
    const csv = (el, p) => getComputedStyle(el)[p]
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    document.querySelectorAll('.sail-card').forEach((el) => {
      const outline = el.getAttribute('variant') === 'outline'
      if (!outline && clear(csv(el, 'backgroundColor'))) {
        bad.push({ why: 'no-surface', fill: el.hasAttribute('fill') })
      }
    })
    const scope = "[theme='ship'] "
    const pad = (v) =>
      parseFloat(getComputedStyle(document.querySelector(`${scope}.sail-card[padding='${v}']`)).paddingTop)
    const [sm, md, lg] = [pad('sm'), pad('md'), pad('lg')]
    if (!(sm < md && md < lg)) bad.push({ why: 'padding-not-monotonic', sm, md, lg })
    if (!document.querySelector('.sail-card[interactive]')) bad.push({ why: 'interactive-missing' })
    if (!document.querySelector('.sail-card[status-rail]')) bad.push({ why: 'status-rail-missing' })
    if (!document.querySelector('.sail-card[accent-bar="static"]')) bad.push({ why: 'accent-bar-missing' })
    if (!document.querySelector('.sail-card[fill]')) bad.push({ why: 'fill-missing' })
    if (!document.querySelector('.sail-card[variant="outline"]')) bad.push({ why: 'outline-missing' })
    const mark = (t) =>
      getComputedStyle(document.querySelector(`[theme='${t}'] .l1-mark`)).backgroundColor
    if (mark('ship') === mark('console')) bad.push({ why: 'frozen-token', light: mark('ship') })
    return { count: document.querySelectorAll('.sail-card').length, bad: bad.slice(0, 10) }
  },
}
