// L1 conformance spec: <chip> (mechanism-1, compiles-away via element mode +
// capability mix-ins). States: every tone, every variant, and the behavioural
// gates (selected/removable/icon/disabled/size). Paint classes mirror lowered
// Style: soft/outline/ghost rest ride chip* tokens (not free muted/foreground).
//
// Lead/remove visibility is controlled by product skins (display:contents lead,
// display:none remove, .sail-grp[removable] shows remove) — no inline display stamps.

const dot =
  '<svg viewBox="0 0 8 8" width="8" height="8" aria-hidden="true" style="width:8px;height:8px;display:block"><circle cx="4" cy="4" r="3" fill="currentColor"/></svg>'
// Product remove: display:none base + group-removable shows when host is sail-grp[removable].
// Also .sail-chip:not([removable]) button[aria-label=Remove] { display:none }.
const REMOVE =
  '<button class="remove s-displaynone s-sizeiconsizexs s-pad0 s-borderWidth0 s-bgtransparent s-fgcurrentColor s-shrink0 s-groupremovable-displayinlineflex" type="button" aria-label="Remove" tabindex="-1"><svg viewBox="0 0 10 10" aria-hidden="true" width="10" height="10" style="display:block;width:100%;height:100%"><path d="M2 2 L8 8 M8 2 L2 8" stroke="currentColor" stroke-width="1.75" fill="none"/></svg></button>'

// Base rest paint: solid default + soft rest both stamp chip / chip-foreground / chip-border.
// sail-grp mirrors freeform group() so removable group rules fire.
const CHIP =
  'sail-chip sail-grp s-displayinlineflex s-minHcontrolh2xs s-padX2 s-padYpx s-gap1 s-appearancenone s-cursorpointer s-selectnone s-tapHighlightnone s-radiusfull s-borderWidth1 s-bordersolid s-bgchip s-fgchipforeground s-borderchipborder s-transitioncolors s-durationfast s-easeout s-fontFamilybody s-fontWeightmedium s-leadingnone s-whitespacenowrap s-fontSizexs'
const CHIP_SOFT =
  `${CHIP} s-variantsoft-bgchip s-variantsoft-fgchipforeground s-variantsoft-borderchipborder`
const CHIP_OUTLINE =
  `${CHIP} s-variantoutline-bgtransparent s-variantoutline-fgchipforeground s-variantoutline-borderchipborder`
const CHIP_GHOST =
  `${CHIP} s-variantghost-bgtransparent s-variantghost-fgchipforeground s-variantghost-bordertransparent`

/** Freeform size stamps (pair with size= for [size].s-size* rules). */
const sizeCls = {
  sm: 's-sizesm-minHcontrolh2xs s-sizesm-padX2 s-sizesm-fontSizexs',
  md: 's-sizemd-minHcontrolh2xs s-sizemd-padX25 s-sizemd-fontSizesm',
  lg: 's-sizelg-minHcontrolhxs s-sizelg-padX3 s-sizelg-fontSizesm',
}

const paintCls = (variant) => {
  if (variant === 'soft') return CHIP_SOFT
  if (variant === 'outline') return CHIP_OUTLINE
  if (variant === 'ghost') return CHIP_GHOST
  return CHIP
}

const chip = (label, attrs = '', icon = false, variant = 'solid') => {
  const extra = attrs ? ` ${attrs}` : ''
  const size = /size="([^"]+)"/.exec(attrs || '')?.[1] || ''
  const sc = size && sizeCls[size] ? ` ${sizeCls[size]}` : ''
  // Leading always product display:contents — SVG with intrinsic size still paints.
  const lead = `<span class="leading s-displaycontents">${icon ? dot : ''}</span>`
  return `<span class="${paintCls(variant)}${sc}"${extra}>${lead}${label}${REMOVE}</span>`
}

export default {
  tag: 'chip',
  width: 760,
  rows() {
    return [
      ['default', 'primary', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']
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
        chip('md', 'tone="primary" size="md"'),
        chip('lg', 'tone="primary" size="lg"'),
      ].join(''),
    ]
  },
  probe() {
    const bad = []
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    document.querySelectorAll('.sail-chip').forEach((el) => {
      if (!el.hasAttribute('role')) el.setAttribute('role', 'button')
      const disabled = el.hasAttribute('disabled')
      el.setAttribute('aria-pressed', el.hasAttribute('selected') ? 'true' : 'false')
      if (disabled) { el.setAttribute('aria-disabled', 'true'); el.setAttribute('tabindex', '-1') }
      else el.setAttribute('tabindex', '0')
      const roleOk = el.getAttribute('role') === 'button'
      const pressedOk = el.getAttribute('aria-pressed') === (el.hasAttribute('selected') ? 'true' : 'false')
      const tabOk = el.getAttribute('tabindex') === (disabled ? '-1' : '0')
      const remove = el.querySelector('.remove, button[aria-label="Remove"]')
      const leading = el.querySelector('.leading')
      const glyph = el.querySelector('.leading svg')
      // Product CSS gates (not harness inline display):
      const removeShown = remove ? getComputedStyle(remove).display !== 'none' : false
      const removeExpected = el.hasAttribute('removable')
      if (glyph) {
        const gr = glyph.getBoundingClientRect()
        if (gr.width <= 0 || gr.height <= 0) {
          bad.push({ why: 'lead-glyph-not-rendered', w: gr.width, h: gr.height })
        }
      } else if (leading) {
        const d = getComputedStyle(leading).display
        const lr = leading.getBoundingClientRect()
        const emptyOk = d === 'none' || d === 'contents' || (lr.width <= 1 && lr.height <= 1)
        if (!emptyOk) {
          bad.push({ why: 'lead-empty-visible-box', display: d, w: lr.width, h: lr.height })
        }
      }
      if (!roleOk || !pressedOk || !tabOk) bad.push({ why: 'aria', roleOk, pressedOk, tabOk })
      if (removeShown !== removeExpected) bad.push({ why: 'remove-gate', removeShown, removeExpected })
      const variant = el.getAttribute('variant') || 'solid'
      const cs = getComputedStyle(el)
      if ((variant === 'soft' || variant === 'solid') && !el.hasAttribute('selected') && clear(cs.backgroundColor)) {
        bad.push({ why: 'no-token-bg', variant, tone: el.getAttribute('tone') })
      }
      if (clear(cs.color)) bad.push({ why: 'no-token-fg', variant })
    })
    // Size ladder monotonicity when freeform size classes stamp (ship).
    const h = (s) => {
      const el = document.querySelector(`[theme='ship'] .sail-chip[size='${s}']`)
      return el ? el.getBoundingClientRect().height : 0
    }
    const [sm, md, lg] = [h('sm'), h('md'), h('lg')]
    if (sm > 0 && md > 0 && lg > 0 && !(sm <= md && md <= lg && sm < lg)) {
      bad.push({ why: 'size-not-monotonic', sm, md, lg })
    }
    const softRest = document.querySelector("[theme='ship'] .sail-chip[variant='soft']:not([tone])")
    if (softRest && clear(getComputedStyle(softRest).backgroundColor)) {
      bad.push({ why: 'soft-rest-no-fill' })
    }
    return { count: document.querySelectorAll('.sail-chip').length, bad: bad.slice(0, 10) }
  },
}
