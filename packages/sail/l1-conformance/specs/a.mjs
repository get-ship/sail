// L1 conformance spec: <a> (native + freeform class stamps). Dual mode:
// bare (no variant) = text link; with variant = button-link chrome.
// Rest paints component.button.*; tone always paints when set.
// default ≠ solid invert. Probe: per-variant chrome + frozen-token on rest default.
// Rows use lowered() markup so utility class stamps match PE freeform.

import { lowered } from '../lowered.mjs'

const link = (label, attrs = '') => {
  const html = lowered('a', { attrs: `href="#"${attrs ? ' ' + attrs : ''}`, children: label })
  if (html) return html
  // Compiler fallback — attribute floor only (no fill); prefer rebuilt compiler.
  return `<a href="#" ${attrs} class="sail-a">${label}</a>`
}

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
      // l1-mark on solid+primary for frozen-token (tone bridge sets --tone).
      [
        (() => {
          const h = link('solid primary mark', 'variant="solid" tone="primary"')
          return h.includes('class="')
            ? h.replace('class="', 'class="l1-mark ')
            : h.replace('<a ', '<a class="l1-mark" ')
        })(),
        link('default rest', 'variant="default"'),
        link('soft rest', 'variant="soft"'),
        link('outline rest', 'variant="outline"'),
        link('ghost rest', 'variant="ghost"'),
        link('solid rest', 'variant="solid"'),
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
      // Freeform stamps rest + tone match arms together. Without tone=, the toned
      // arm can win with unset --tone → transparent fill. Require fill when tone
      // is set; otherwise require freeform stamp + non-clear ink.
      const hasTone = !!el.getAttribute('tone')
      if (variant === 'solid' && hasTone && clear(bg))
        bad.push({ variant, why: 'expected-fill', bg })
      if (variant === 'solid' && !hasTone && !el.className.includes('s-variantsolid'))
        bad.push({ variant, why: 'missing-solid-stamp' })
      if ((variant === 'soft' || variant === 'default') && clear(bg) && clear(color))
        bad.push({ variant, why: 'expected-fill-or-ink', bg, color })
      if ((variant === 'soft' || variant === 'default') && !el.className.includes('s-variant'))
        bad.push({ variant, why: 'missing-freeform-stamp' })
      if (variant === 'solid' && hasTone && bg === color && !clear(bg))
        bad.push({ variant, why: 'text=fill (unreadable)', bg })
      if (variant === 'outline' && clear(border) && clear(color))
        bad.push({ variant, why: 'outline-no-border' })
      if (clear(color))
        bad.push({ variant, why: 'no-text-color' })
    })
    // Frozen-token guard — toned solid primary fill must differ light vs dark
    const mark = (t) =>
      getComputedStyle(document.querySelector(`[theme='${t}'] .l1-mark`)).backgroundColor
    const light = mark('ship')
    const dark = mark('console')
    if (light === dark || light === 'rgba(0, 0, 0, 0)')
      bad.push({ why: 'frozen-token', detail: 'toned solid bg identical or clear light vs dark', light, dark })
    return { count: document.querySelectorAll('a').length, bad: bad.slice(0, 10) }
  },
}
