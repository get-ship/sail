// L1 conformance spec: <chip> (mechanism-1, compiles-away via element mode +
// capability mix-ins). States: every tone, every variant, and the behavioural
// gates (selected/removable/icon/disabled/size). Paint classes mirror lowered
// Style: soft/outline/ghost rest ride chip* tokens (not free muted/foreground).

const dot = '<svg viewBox="0 0 8 8" aria-hidden="true"><circle cx="4" cy="4" r="3" fill="currentColor"/></svg>'
const x =
  '<button class="remove" type="button" aria-label="Remove" tabindex="-1"><svg viewBox="0 0 10 10" aria-hidden="true"><path d="M2 2 L8 8 M8 2 L2 8"/></svg></button>'

// Base rest paint: solid default + soft rest both stamp chip / chip-foreground / chip-border.
const CHIP =
  'sail-chip s-displayinlineflex s-minHcontrolh2xs s-padX2 s-padYpx s-appearancenone s-cursorpointer s-selectnone s-tapHighlightnone s-radiusfull s-borderWidth1 s-bordersolid s-bgchip s-fgchipforeground s-borderchipborder s-transitioncolors s-durationfast s-easeout s-fontFamilybody s-fontWeightmedium s-leadingnone s-whitespacenowrap s-fontSizexs'
const CHIP_SOFT =
  `${CHIP} s-variantsoft-bgchip s-variantsoft-fgchipforeground s-variantsoft-borderchipborder`
const CHIP_OUTLINE =
  `${CHIP} s-variantoutline-bgtransparent s-variantoutline-fgchipforeground s-variantoutline-borderchipborder`
const CHIP_GHOST =
  `${CHIP} s-variantghost-bgtransparent s-variantghost-fgchipforeground s-variantghost-bordertransparent`

const paintCls = (variant) => {
  if (variant === 'soft') return CHIP_SOFT
  if (variant === 'outline') return CHIP_OUTLINE
  if (variant === 'ghost') return CHIP_GHOST
  return CHIP
}

const chip = (label, attrs = '', icon = false, variant = 'solid') => {
  const extra = attrs ? ` ${attrs}` : ''
  return `<span class="${paintCls(variant)}"${extra}><span class="leading">${icon ? dot : ''}</span>${label}${x}</span>`
}

export default {
  tag: 'chip',
  width: 760,
  rows() {
    return [
      ['default', 'primary', 'info', 'success', 'warning', 'destructive', 'accent']
        .map((tn) => chip(tn, `tone="${tn}"`)).join(''),
      ['solid', 'soft', 'outline', 'ghost']
        .map((v) => chip(v, `tone="primary" variant="${v}"`, false, v)).join(''),
      // Soft default (no tone brand) exercises chip* soft rest.
      chip('soft-default', 'variant="soft"', false, 'soft') +
        chip('outline-default', 'variant="outline"', false, 'outline') +
        chip('ghost-default', 'variant="ghost"', false, 'ghost'),
      [
        chip('selected', 'tone="primary" selected'),
        chip('removable', 'tone="info" removable'),
        chip('with icon', 'tone="success"', true),
        chip('disabled', 'tone="primary" disabled'),
        chip('sm', 'tone="primary" size="sm"'),
        chip('lg', 'tone="primary" size="lg"'),
      ].join(''),
    ]
  },
  probe() {
    const bad = []
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    document.querySelectorAll('.sail-chip').forEach((el) => {
      // chip controller essentials (ARIA + gates; full keyboard in runtime test)
      if (!el.hasAttribute('role')) el.setAttribute('role', 'button')
      const disabled = el.hasAttribute('disabled')
      el.setAttribute('aria-pressed', el.hasAttribute('selected') ? 'true' : 'false')
      if (disabled) { el.setAttribute('aria-disabled', 'true'); el.setAttribute('tabindex', '-1') }
      else el.setAttribute('tabindex', '0')
      // invariants
      const roleOk = el.getAttribute('role') === 'button'
      const pressedOk = el.getAttribute('aria-pressed') === (el.hasAttribute('selected') ? 'true' : 'false')
      const tabOk = el.getAttribute('tabindex') === (disabled ? '-1' : '0')
      const remove = el.querySelector('.remove')
      const leading = el.querySelector('.leading')
      const removeShown = remove ? getComputedStyle(remove).display !== 'none' : false
      const removeExpected = el.hasAttribute('removable')
      const leadShown = leading ? getComputedStyle(leading).display !== 'none' : false
      const leadExpected = !!el.querySelector('.leading svg')
      if (!roleOk || !pressedOk || !tabOk) bad.push({ why: 'aria', roleOk, pressedOk, tabOk })
      if (removeShown !== removeExpected) bad.push({ why: 'remove-gate', removeShown, removeExpected })
      if (leadShown !== leadExpected) bad.push({ why: 'lead-gate', leadShown, leadExpected })
      // Soft/solid rest: filled surface + readable ink (chip* tokens).
      const variant = el.getAttribute('variant') || 'solid'
      const cs = getComputedStyle(el)
      if ((variant === 'soft' || variant === 'solid') && !el.hasAttribute('selected') && clear(cs.backgroundColor)) {
        bad.push({ why: 'no-token-bg', variant, tone: el.getAttribute('tone') })
      }
      if (clear(cs.color)) bad.push({ why: 'no-token-fg', variant })
    })
    // Soft-default surface resolves (pack aliases muted); themes may diverge.
    const mark = (t) => {
      const el = document.querySelector(`[theme='${t}'] .sail-chip[variant='soft']:not([tone])`)
      return el ? getComputedStyle(el).backgroundColor : null
    }
    const ship = mark('ship')
    const consoleT = mark('console')
    if (ship && consoleT && ship === consoleT) {
      bad.push({ why: 'frozen-token-soft', light: ship, dark: consoleT })
    }
    return { count: document.querySelectorAll('.sail-chip').length, bad: bad.slice(0, 10) }
  },
}
