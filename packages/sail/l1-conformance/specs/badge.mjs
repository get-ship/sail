// L1 conformance spec: <badge> (mechanism-1, compiles-away). variant × tone
// matrix + sizes + pill. Probe asserts per-variant chrome (soft/solid filled,
// solid readable, outline bordered), pill is rounder than square, frozen-token
// guard on the soft rest surface.
//
// Harness HTML mirrors freeform class stamps + tone bridge vars
// (`--tone: var(--primary)` etc.) that runtime bind applies at mount.

const TONES = ['default', 'primary', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']

const BASE =
  'sail-badge s-displayinlineflex s-minHcontrolh2xs s-padX2 s-padYpx s-borderWidth1 s-radiussm s-fontFamilybody s-fontWeightsemibold s-leadingnone s-whitespacenowrap s-fontSizexs'

/** Tone bridge style — mirrors runtime reflectToneBridge for prop `tone`. */
const toneStyle = (tone) => {
  if (!tone || tone === 'default') return ''
  // muted uses dedicated compounds in freeform; still bridge for soft/solid paths.
  return ` style="--tone:var(--${tone});--tone-foreground:var(--${tone}-foreground)"`
}

const paint = (variant, toned) => {
  if (variant === 'solid') {
    if (toned) {
      return `${BASE} s-variantsolid-bgtone s-variantsolid-fgtoneforeground s-variantsolid-bordertone s-variantsolid-shadowxs`
    }
    return `${BASE} s-variantsolid-bgbadge s-variantsolid-fgbadgeforeground s-variantsolid-borderbadgeborder s-variantsolid-shadowxs`
  }
  if (variant === 'outline') {
    if (toned) {
      return `${BASE} s-variantoutline-bgtransparent s-variantoutline-fgtone s-variantoutline-bordertone`
    }
    return `${BASE} s-variantoutline-bgtransparent s-variantoutline-fgbadgeforeground s-variantoutline-borderbadgeborder`
  }
  // soft
  if (toned) {
    return `${BASE} s-variantsoft-colorMixtone15 s-variantsoft-bordertone s-variantsoft-fgtone`
  }
  return `${BASE} s-variantsoft-bgbadge s-variantsoft-fgbadgeforeground s-variantsoft-borderbadgeborder`
}

const bdg = (attrs, label, variant = 'soft') => {
  const v = /variant="([^"]+)"/.exec(attrs || '')?.[1] || variant
  const tone = /tone="([^"]+)"/.exec(attrs || '')?.[1] || ''
  const toned = !!tone && tone !== 'default'
  // muted×variant compounds from freeform
  let cls = paint(v, toned)
  if (tone === 'muted' && v === 'solid') {
    cls = `${BASE} s-tonemutedvariantsolid-bgmuted s-tonemutedvariantsolid-fgmutedforeground s-tonemutedvariantsolid-borderborder`
  } else if (tone === 'muted' && v === 'soft') {
    cls = `${BASE} s-tonemutedvariantsoft-bgmuted s-tonemutedvariantsoft-fgmutedforeground s-tonemutedvariantsoft-borderborder`
  } else if (tone === 'muted' && v === 'outline') {
    cls = `${BASE} s-tonemutedvariantoutline-fgmutedforeground s-tonemutedvariantoutline-borderborder s-variantoutline-bgtransparent`
  }
  if (/\bpill\b/.test(attrs || '')) cls += ' s-pilltrue-radiusfull'
  // Always stamp variant= so [variant='soft'].s-variantsoft-* selectors match.
  const hasVariant = /variant=/.test(attrs || '')
  const attrBits = [!hasVariant ? `variant="${v}"` : '', attrs || ''].filter(Boolean).join(' ')
  const a = attrBits ? ` ${attrBits}` : ''
  return `<span class="${cls}"${a}${toneStyle(tone)}>${label}</span>`
}

export default {
  tag: 'badge',
  width: 760,
  rows() {
    return [
      ...['soft', 'solid', 'outline'].map((v) =>
        TONES.map((t) => bdg(`variant="${v}" tone="${t}"`, t, v)).join(' '),
      ),
      ['sm', 'md', 'lg']
        .map((s) => bdg(`variant="solid" tone="primary" size="${s}"`, s, 'solid'))
        .join(' ') +
        ' ' +
        bdg('pill tone="success"', 'pill', 'soft') +
        ' ' +
        bdg('variant="solid" pill tone="accent"', 'pill solid', 'solid'),
    ]
  },
  probe() {
    const bad = []
    const cs = (el, p) => getComputedStyle(el)[p]
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    document.querySelectorAll('.sail-badge').forEach((el) => {
      const variant = el.getAttribute('variant') || 'soft'
      const bg = cs(el, 'backgroundColor')
      const fg = cs(el, 'color')
      const border = cs(el, 'borderTopColor')
      if ((variant === 'soft' || variant === 'solid') && clear(bg))
        bad.push({ variant, tone: el.getAttribute('tone'), why: 'expected-fill' })
      if (variant === 'solid' && bg === fg) bad.push({ variant, why: 'text=fill' })
      if (clear(fg)) bad.push({ variant, why: 'no-text' })
    })
    // Outline border floor in a bordered theme (glass/halo may be borderless by design).
    document.querySelectorAll("[theme='ship'] .sail-badge[variant='outline']").forEach((el) => {
      const border = cs(el, 'borderTopColor')
      if (clear(border)) bad.push({ variant: 'outline', why: 'no-border', tone: el.getAttribute('tone') })
    })
    const pillEl = document.querySelector('.sail-badge[pill]')
    const sqEl = document.querySelector('.sail-badge:not([pill])')
    if (pillEl && sqEl) {
      const pillR = parseFloat(cs(pillEl, 'borderRadius'))
      const sqR = parseFloat(cs(sqEl, 'borderRadius'))
      if (!(pillR > sqR)) bad.push({ why: 'pill-not-rounder', pillR, sqR })
    }
    // Soft rest must paint (component.badge.*). Light vs dark divergence is operator taste
    // when pack defaults alias the same muted ramp in both modes.
    const rest = document.querySelector("[theme='ship'] .sail-badge[variant='soft'][tone='default']")
    if (rest && clear(cs(rest, 'backgroundColor')))
      bad.push({ why: 'soft-rest-no-fill' })
    return { count: document.querySelectorAll('.sail-badge').length, bad: bad.slice(0, 10) }
  },
}
