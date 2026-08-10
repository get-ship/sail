// L1 conformance spec: <a> (native + A_SKIN_CSS). Dual mode:
// bare (no variant) = text link; with variant = button-link chrome.
// Rest paints component.button.*; tone always paints when set.
// default ≠ solid invert. Probe: per-variant chrome + frozen-token on rest default.

const link = (label, attrs = '') => `<a href="#" ${attrs}>${label}</a>`

export default {
  tag: 'a',
  width: 720,
  rows() {
    return [
      // bare text link + variants (tone=primary)
      [
        link('bare'),
        ...['default', 'solid', 'soft', 'outline', 'ghost'].map((v) =>
          link(v, `variant="${v}" tone="primary"`),
        ),
      ].join(''),
      // tones (variant=solid)
      ['primary', 'success', 'warning', 'destructive', 'info', 'accent', 'muted']
        .map((t) => link(t, `variant="solid" tone="${t}"`))
        .join(''),
      // rest tokens (no tone) + soft/outline rest + disabled paint
      [
        link('default rest', 'class="l1-mark" variant="default"'),
        link('soft rest', 'variant="soft"'),
        link('outline rest', 'variant="outline"'),
        link('ghost rest', 'variant="ghost"'),
        link('disabled', 'variant="solid" tone="primary" aria-disabled="true"'),
      ].join(''),
    ]
  },
  probe() {
    const bad = []
    const cs = (el, prop) => getComputedStyle(el)[prop]
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    document.querySelectorAll('a').forEach((el) => {
      const variant = el.getAttribute('variant')
      // Bare text links — no button fill required
      if (!variant) {
        if (clear(cs(el, 'color'))) bad.push({ variant: 'bare', why: 'no-text-color' })
        return
      }
      const bg = cs(el, 'backgroundColor')
      const color = cs(el, 'color')
      const border = cs(el, 'borderTopColor')
      if ((variant === 'solid' || variant === 'soft' || variant === 'default') && clear(bg))
        bad.push({ variant, why: 'expected-fill', bg })
      if (variant === 'solid' && el.getAttribute('tone') && bg === color)
        bad.push({ variant, why: 'text=fill (unreadable)', bg })
      if (variant === 'outline' && clear(border))
        bad.push({ variant, why: 'outline-no-border' })
      if (clear(color))
        bad.push({ variant, why: 'no-text-color' })
    })
    // Frozen-token guard — rest default (component.button) must differ light vs dark
    const mark = (t) =>
      getComputedStyle(document.querySelector(`[theme='${t}'] .l1-mark`)).backgroundColor
    const light = mark('ship')
    const dark = mark('console')
    if (light === dark)
      bad.push({ why: 'frozen-token', detail: 'rest default bg identical light vs dark', light, dark })
    return { count: document.querySelectorAll('a').length, bad: bad.slice(0, 10) }
  },
}
