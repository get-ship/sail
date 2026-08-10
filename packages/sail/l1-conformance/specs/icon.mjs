// L1 conformance spec: <icon> (mechanism-1, compiles-away). Wraps an svg,
// sizes via the channel, inherits currentColor. Probe: renders, sizes are
// monotonic, glyph inherits color.
// Harness mirrors freeform size utility stamps from element-skins.

const svg =
  '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2l2.4 7.4H22l-6 4.6 2.3 7.4-6.3-4.7L5.7 21.4 8 14 2 9.4h7.6z"/></svg>'

const sizeCls = {
  xs: 's-sizexs-sizeiconsize2xs',
  sm: 's-sizesm-sizeiconsizesm',
  md: 's-sizemd-sizeiconsizemd',
  lg: 's-sizelg-sizeiconsizexl',
  xl: 's-sizexl-sizeiconsize2xl',
}

const ic = (size) => {
  const s = size || 'md'
  const attr = size ? ` size="${size}"` : ''
  const sc = sizeCls[s] || sizeCls.md
  return `<span class="sail-icon s-displayinlineblock s-sizeiconsizemd s-alignmiddle s-fgcurrentColor s-glyphfill s-shrink0 ${sc}"${attr}>${svg}</span>`
}

export default {
  tag: 'icon',
  width: 640,
  rows() {
    return [
      ['xs', 'sm', 'md', 'lg', 'xl'].map((s) => ic(s)).join(' '),
      `<span style="color:var(--primary)">${ic()}</span> <span style="color:var(--destructive)">${ic()}</span> <span style="color:var(--success)">${ic()}</span>`,
    ]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sail-icon').forEach((el) => {
      if (el.getBoundingClientRect().width <= 0)
        bad.push({ why: 'not-rendered', size: el.getAttribute('size') })
    })
    const scope = "[theme='ship'] "
    const w = (s) =>
      document.querySelector(`${scope}.sail-icon[size='${s}']`)?.getBoundingClientRect().width ?? 0
    const [xs, sm, lg] = [w('xs'), w('sm'), w('lg')]
    if (!(xs < sm && sm < lg)) bad.push({ why: 'size-not-monotonic', xs, sm, lg })
    return { count: document.querySelectorAll('.sail-icon').length, bad: bad.slice(0, 8) }
  },
}
