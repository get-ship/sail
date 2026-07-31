// L1 conformance spec: <button> (mechanism-2, native + BUTTON_SKIN_CSS). A bare
// <button> IS the design-system button; tone/size arrive via ordinary
// attribute-keyed props, variant surfaces via the skin. The probe asserts per-variant chrome
// resolves (fills aren't transparent, solid text ≠ fill) AND a cross-theme
// frozen-token guard: the neutral surface must differ light vs dark.

const btn = (label, attrs = '') => `<button ${attrs}>${label}</button>`

export default {
  tag: 'button',
  width: 720,
  rows() {
    return [
      // variants (tone=primary)
      ['default', 'solid', 'soft', 'outline', 'ghost']
        .map((v) => btn(v, `variant="${v}" tone="primary"`)).join(''),
      // tones (variant=solid)
      ['default', 'primary', 'success', 'warning', 'destructive', 'info', 'accent']
        .map((t) => btn(t, `variant="solid" tone="${t}"`)).join(''),
      // sizes + disabled
      [
        btn('sm', 'variant="solid" tone="primary" size="sm"'),
        btn('md', 'variant="solid" tone="primary"'),
        btn('lg', 'variant="solid" tone="primary" size="lg"'),
        btn('disabled', 'variant="solid" tone="primary" disabled'),
        btn('outline off', 'variant="outline" tone="destructive" disabled'),
      ].join(''),
      // frozen-token marker: neutral surface (uses --background), per theme
      btn('neutral', 'class="l1-mark" variant="default"'),
    ]
  },
  probe() {
    const bad = []
    const cs = (el, prop) => getComputedStyle(el)[prop]
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    document.querySelectorAll('button').forEach((el) => {
      const variant = el.getAttribute('variant') || 'default'
      const bg = cs(el, 'backgroundColor')
      const color = cs(el, 'color')
      const border = cs(el, 'borderTopColor')
      if ((variant === 'solid' || variant === 'soft' || variant === 'default') && clear(bg))
        bad.push({ variant, why: 'expected-fill', bg })
      if (variant === 'solid' && bg === color)
        bad.push({ variant, why: 'text=fill (unreadable)', bg })
      if (variant === 'outline' && clear(border))
        bad.push({ variant, why: 'outline-no-border' })
      if (clear(color))
        bad.push({ variant, why: 'no-text-color' })
    })
    // Frozen-token guard — the exact class of bug (--card/--tone-muted-bg) that
    // slipped through before: the neutral surface must resolve differently in
    // light vs dark, or a token is frozen at one mode's value.
    const mark = (t) =>
      getComputedStyle(document.querySelector(`[data-theme-preview='${t}'] .l1-mark`)).backgroundColor
    const light = mark('ship')
    const dark = mark('console')
    if (light === dark)
      bad.push({ why: 'frozen-token', detail: 'neutral bg identical light vs dark', light, dark })
    return { count: document.querySelectorAll('button').length, bad: bad.slice(0, 10) }
  },
}
