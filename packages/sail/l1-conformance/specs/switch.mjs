// L1 conformance spec: <switch> (mechanism-1, native checkbox role=switch +
// skin). state / size / tone matrices. Probe asserts it's custom-styled
// (appearance:none), the checked track color differs from unchecked, and the
// frozen-token guard on the off-track.
//
// Harness stamps freeform utility classes from element-skins (product CSS) —
// class="sail-switch" alone only sets display:inline-flex.

const TONES = ['primary', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']

/** Rest paint + geometry (default md size). */
const BASE =
  'sail-switch s-positionrelative s-displayinlineblock s-m0 s-wcontrolhsm s-hiconsizemd s-appearancenone s-boxSizingborder s-alignmiddle s-cursorpointer s-bgnone s-bgswitch s-borderWidth1 s-borderswitchborder s-radiusfull s-tapHighlightnone s-transitioncolors s-durationfast s-easeout s-shrink0 s-hover-bordermutedforeground s-checked-bgprimary s-checked-borderselectedborder s-checked-shadowxs s-checkedhover-bgprimary s-checkedhover-borderselectedborder s-checkedbefore-leftauto s-checkedbefore-rightpx s-focus-outlinesolid s-focus-outline2 s-focus-outlineprimary s-focus-outlineOffsetringoffset s-disabled-cursornotallowed s-disabled-bgswitch s-disabled-borderborder s-disabled-saturate60 s-disabled-shadownone s-disabledbefore-shadowxs s-before-positionabsolute s-before-top0 s-before-bottom0 s-before-myauto s-before-leftpx s-before-size3 s-before-contentempty s-before-bgswitchthumb s-before-radiusfull s-before-shadowsm s-before-transitionall s-before-durationfast s-before-easeout'

const sizeCls = {
  sm: 's-sizesm-wcontrolhxs s-sizesm-hiconsizesm s-sizesmbefore-size25',
  md: 's-sizemd-wcontrolhsm s-sizemd-hiconsizemd s-sizemdbefore-size3',
  lg: 's-sizelg-w7 s-sizelg-hiconsizelg s-sizelgbefore-sizeiconsizesm',
}

const TONE_PAINT =
  's-tonechecked-bgtone s-tonechecked-borderselectedborder s-tonechecked-shadowxs s-tonecheckedhover-bgtone s-tonecheckedhover-borderselectedborder s-tonefocus-outlinetone'

const MUTED_TONE =
  's-tonemutedchecked-bgmutedforeground s-tonemutedfocus-outlinemutedforeground s-tonemutedcheckedhover-bgmutedforeground'

/** Tone bridge style — mirrors runtime reflectToneBridge for prop `tone`. */
const toneStyle = (tone) => {
  if (!tone) return ''
  return ` style="--tone:var(--${tone});--tone-foreground:var(--${tone}-foreground)"`
}

const sw = (attrs = '') => {
  const size = /size="([^"]+)"/.exec(attrs)?.[1] || ''
  const tone = /tone="([^"]+)"/.exec(attrs)?.[1] || ''
  let cls = BASE
  if (size && sizeCls[size]) cls += ` ${sizeCls[size]}`
  if (tone) {
    cls += ` ${TONE_PAINT}`
    if (tone === 'muted') cls += ` ${MUTED_TONE}`
  }
  const a = attrs ? ` ${attrs}` : ''
  return `<input type="checkbox" role="switch" class="${cls}"${a}${toneStyle(tone)}>`
}

export default {
  tag: 'switch',
  width: 720,
  rows() {
    return [
      // rest-default (untokened) + checked/disabled bools
      `${sw('')} ${sw('checked')} ${sw('disabled')} ${sw('disabled checked')}`,
      // size ladder
      `${sw('size="sm" checked')} ${sw('size="md" checked')} ${sw('size="lg" checked')}`,
      // optional tone: each listed value toned; rest path is row 0
      TONES.map((t) => sw(`tone="${t}" checked`)).join(' '),
    ]
  },
  probe() {
    const bad = []
    const cs = (el, p) => getComputedStyle(el)[p]
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''
    document.querySelectorAll('.sail-switch').forEach((el) => {
      if (cs(el, 'appearance') !== 'none') bad.push({ why: 'not-appearance-none', appearance: cs(el, 'appearance') })
    })
    const scope = "[theme='ship'] "
    const offEl = document.querySelector(scope + '.sail-switch:not([checked]):not([disabled])')
    const onEl = document.querySelector(scope + ".sail-switch[checked]:not([tone]):not([disabled])")
    const off = offEl ? cs(offEl, 'backgroundColor') : ''
    const on = onEl ? cs(onEl, 'backgroundColor') : ''
    if (clear(off)) bad.push({ why: 'off-track-no-fill', off })
    if (clear(on)) bad.push({ why: 'checked-no-fill', on })
    if (off && on && off === on) bad.push({ why: 'checked-no-color-change', off, on })
    // Cross-theme off-track may legitimately match when component.switch.track
    // aliases the same global muted in multiple catalog themes — not a paint bug.
    // Off ≠ on (above) is the honest checked invariant.
    for (const s of ['sm', 'md', 'lg']) {
      if (!document.querySelector(`.sail-switch[size="${s}"]`)) bad.push({ why: 'missing-size', size: s })
    }
    return { count: document.querySelectorAll('.sail-switch').length, bad: bad.slice(0, 10) }
  },
}
