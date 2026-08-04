// L1 conformance spec: <badge> (mechanism-1, compiles-away). variant × tone
// matrix + sizes + pill. Probe asserts per-variant chrome (soft/solid filled,
// solid readable, outline bordered), pill is rounder than square, frozen-token
// guard on the soft-default surface.

const TONES = ['default', 'primary', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']
const bdg = (a, t) => `<span class="sail-badge" ${a}>${t}</span>`

export default {
  tag: 'badge',
  width: 760,
  rows() {
    return [
      ...['soft', 'solid', 'outline'].map((v) => TONES.map((t) => bdg(`variant="${v}" tone="${t}"`, t)).join(' ')),
      ['sm', 'md', 'lg'].map((s) => bdg(`variant="solid" tone="primary" size="${s}"`, s)).join(' ') +
        ' ' + bdg('pill tone="success"', 'pill') + ' ' + bdg('variant="solid" pill tone="accent"', 'pill solid'),
    ]
  },
  probe() {
    const bad = []
    const cs = (el, p) => getComputedStyle(el)[p]
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    document.querySelectorAll('.sail-badge').forEach((el) => {
      const variant = el.getAttribute('variant') || 'soft'
      const bg = cs(el, 'backgroundColor'), fg = cs(el, 'color'), border = cs(el, 'borderTopColor')
      if ((variant === 'soft' || variant === 'solid') && clear(bg)) bad.push({ variant, tone: el.getAttribute('tone'), why: 'expected-fill' })
      if (variant === 'solid' && bg === fg) bad.push({ variant, why: 'text=fill' })
      if (variant === 'outline' && clear(border)) bad.push({ variant, why: 'no-border' })
      if (clear(fg)) bad.push({ variant, why: 'no-text' })
    })
    const pillR = parseFloat(cs(document.querySelector('.sail-badge[pill]'), 'borderRadius'))
    const sqR = parseFloat(cs(document.querySelector('.sail-badge:not([pill])'), 'borderRadius'))
    if (!(pillR > sqR)) bad.push({ why: 'pill-not-rounder', pillR, sqR })
    const mark = (t) => cs(document.querySelector(`[theme='${t}'] .sail-badge[variant='soft'][tone='default']`), 'backgroundColor')
    if (mark('ship') === mark('console')) bad.push({ why: 'frozen-token', light: mark('ship'), dark: mark('console') })
    return { count: document.querySelectorAll('.sail-badge').length, bad: bad.slice(0, 10) }
  },
}
