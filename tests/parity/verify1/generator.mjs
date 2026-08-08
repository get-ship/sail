/**
 * VERIFY-1 seeded deterministic layout generator.
 *
 * Produces static .sail sources (no controllers) covering the vocab matrix.
 * Each case carries dimension tags for per-dimension match rates.
 */

import { VOCAB } from './vocab.mjs'

/** Mulberry32 PRNG — deterministic, fast. */
export function mulberry32(seed) {
  let a = seed >>> 0
  return function rand() {
    a |= 0
    a = (a + 0x6d2b79f5) | 0
    let t = Math.imul(a ^ (a >>> 15), 1 | a)
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296
  }
}

function pick(rand, arr) {
  return arr[Math.floor(rand() * arr.length)]
}

function chance(rand, p) {
  return rand() < p
}

const TEXTS = {
  latin_short: 'Hello',
  latin_medium: 'The quick brown fox jumps over the lazy dog.',
  latin_long:
    'Layout proof corpus: flexbox and grid participation, alignment, gap, padding, wrap, and overflow across nested shells of varying depth.',
  cjk_short: '你好世界',
  cjk_medium: '静的レイアウト検証：列と行のネスト、余白、折返し。',
  emoji_short: 'OK ✅ 🚀',
  emoji_medium: 'Ship it 🚢 with 🎯 layout + 🎨 paint',
  mixed: 'Hello 世界 🚀 — mixed script line for measure.',
  whitespace: '  spaced   tabs\there  ',
  wrap_stress:
    'WordWrapWordWrap WordWrapWordWrap WordWrapWordWrap supercalifragilisticexpialidocious measurement stress line.',
}

function textFor(kind, rand) {
  switch (kind) {
    case 'latin':
      return pick(rand, [TEXTS.latin_short, TEXTS.latin_medium, TEXTS.latin_long])
    case 'cjk':
      return pick(rand, [TEXTS.cjk_short, TEXTS.cjk_medium])
    case 'emoji':
      return pick(rand, [TEXTS.emoji_short, TEXTS.emoji_medium])
    case 'mixed':
      return TEXTS.mixed
    case 'whitespace':
      return TEXTS.whitespace
    case 'empty':
      return ''
    default:
      return TEXTS.latin_medium
  }
}

function sxChain(parts) {
  const body = parts.filter(Boolean).join('.')
  return body ? `sx={style.${body}}` : ''
}

function attrs(map) {
  return Object.entries(map)
    .filter(([, v]) => v != null && v !== '' && v !== false)
    .map(([k, v]) => (v === true ? k : `${k}="${v}"`))
    .join(' ')
}

/**
 * Build a leaf span, optionally with text size class via sx is not used —
 * text-* classes are CSS; for taffy metrics we use bare spans + length variety.
 */
function leaf(text, tag = 'span') {
  if (!text) return `<${tag}></${tag}>`
  // Escape minimal XML
  const t = String(text)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
  return `<${tag}>${t}</${tag}>`
}

/**
 * Layout keyword attr surface (from sail-layout keyword checks):
 *   column/row: gap, align, justify, nowrap(row), w, grow, shrink, …
 *   center: gap, inline — NOT align/h
 *   stack: align, justify, w — NOT h/gap in some builds; keep minimal
 *   grid: gap, cols, rows, min, w — NOT h
 * Height always via style.h(token), never h= attr.
 */
export function generateCase(rand, index) {
  const dims = {}
  const depth = 1 + Math.floor(rand() * 4) // 1..4
  dims.depth = String(depth)

  const prims = VOCAB.primitives.values
  const rootPrim = pick(rand, prims)
  dims.primitive = rootPrim

  function node(level) {
    const prim = level === 0 ? rootPrim : pick(rand, prims)
    dims[`primitive_l${level}`] = prim

    const gap =
      prim !== 'stack' && chance(rand, 0.85)
        ? pick(rand, ['0', '1', '2', '3', '4', '6', '8'])
        : null
    if (gap != null) dims.gap = gap

    // align/justify only on flex-ish primitives that accept them
    const allowAlign = prim === 'column' || prim === 'row' || prim === 'stack'
    const allowJustify = prim === 'column' || prim === 'row' || prim === 'stack'
    const align =
      allowAlign && chance(rand, 0.55)
        ? pick(rand, ['start', 'end', 'center', 'stretch'])
        : null
    if (align) dims.align = align
    const justify =
      allowJustify && chance(rand, 0.45)
        ? pick(rand, ['start', 'end', 'center', 'between'])
        : null
    if (justify) dims.justify = justify

    // Compile-valid size tokens only (lint rejects fractions; 72+ not on scale).
    const w = chance(rand, 0.9)
      ? pick(rand, ['full', 'full', 'full', '32', '40', '48', '56', '64', 'screen'])
      : 'full'
    dims.w = w

    // height via style only; scale tokens (×4px)
    const hTok = chance(rand, 0.3) ? pick(rand, ['8', '12', '16', '24', '32']) : null
    if (hTok) dims.h = hTok

    const nowrap = prim === 'row' && chance(rand, 0.4)
    if (prim === 'row') dims.wrap = nowrap ? 'nowrap' : 'default-wrap'

    // grow/shrink are participation ATTRS, not style.* (lint: sail-participation-style)
    const grow =
      (prim === 'column' || prim === 'row') && chance(rand, 0.2)
        ? pick(rand, ['0', '1'])
        : null
    if (grow != null) dims.grow = grow
    const shrink =
      (prim === 'column' || prim === 'row') && chance(rand, 0.15)
        ? pick(rand, ['0', '1'])
        : null
    if (shrink != null) dims.shrink = shrink

    const pad = chance(rand, 0.45) ? pick(rand, ['0', '2', '4', '6']) : null
    if (pad != null) dims.pad = pad

    const overflow = chance(rand, 0.2)
      ? pick(rand, ['hidden', 'clip', 'auto', 'scroll', 'visible'])
      : null
    if (overflow) dims.overflow = overflow

    const minW = chance(rand, 0.12) ? pick(rand, ['0', '32']) : null
    if (minW != null) dims.minW = minW
    const maxH = chance(rand, 0.1) ? pick(rand, ['8', '12', '16', '24']) : null
    if (maxH != null) dims.maxH = maxH
    const minH = chance(rand, 0.08) ? pick(rand, ['0', '8', '12']) : null
    if (minH != null) dims.minH = minH

    const sxParts = []
    if (pad != null) sxParts.push(`pad(${pad})`)
    if (hTok) sxParts.push(`h(${hTok})`)
    if (overflow) sxParts.push(`overflow(${overflow})`)
    if (minW != null) sxParts.push(`minW(${minW})`)
    if (minH != null) sxParts.push(`minH(${minH})`)
    if (maxH != null) sxParts.push(`maxH(${maxH})`)

    /** @type {Record<string, string|boolean|null|undefined>} */
    const a = { w }

    if (prim === 'column' || prim === 'row' || prim === 'grid') {
      if (gap != null) a.gap = gap
    }
    if (prim === 'column' || prim === 'row') {
      if (align) a.align = align
      if (justify) a.justify = justify
      if (grow === '1') a.grow = true
      else if (grow === '0') a.grow = '0'
      if (shrink === '0') a.shrink = '0'
      else if (shrink === '1') a.shrink = '1'
    }
    if (prim === 'row' && nowrap) a.nowrap = true
    if (prim === 'stack') {
      if (align) a.align = align
      if (justify) a.justify = justify
    }
    if (prim === 'center' && gap != null) a.gap = gap
    if (prim === 'grid') {
      const cols = pick(rand, ['2', '3', '4'])
      a.cols = cols
      dims.grid_cols = cols
    }

    const openAttrs = [attrs(a), sxChain(sxParts)].filter(Boolean).join(' ')
    const nChildren = 1 + Math.floor(rand() * (level < depth - 1 ? 3 : 4))
    const children = []
    for (let i = 0; i < nChildren; i++) {
      if (level < depth - 1 && chance(rand, 0.55)) {
        children.push(node(level + 1))
      } else {
        const script = pick(rand, ['latin', 'latin', 'latin', 'cjk', 'emoji', 'mixed'])
        dims.text_script = script
        const kind = chance(rand, 0.15) ? 'wrap_stress' : script
        if (kind === 'wrap_stress') dims.text_length = 'wrap-stress'
        else dims.text_length = pick(rand, ['short', 'medium', 'long'])
        const t =
          kind === 'wrap_stress' ? TEXTS.wrap_stress : textFor(script, rand)
        children.push(leaf(t))
      }
    }
    return `<${prim}${openAttrs ? ' ' + openAttrs : ''}>\n${children.map((c) => '  ' + c).join('\n')}\n</${prim}>`
  }

  const source = node(0)
  return {
    id: `g${String(index).padStart(5, '0')}`,
    source,
    dims,
    family: 'geometry',
    depth,
  }
}

/** Text-measurement-focused cases (work item 3). */
export function generateTextCases(rand, count) {
  const scripts = ['latin', 'cjk', 'emoji', 'mixed', 'whitespace']
  // style.fontSize tokens (closed type scale)
  const sizes = ['xs', 'sm', 'base', 'md', 'lg', 'xl', '2xl']
  const families = [
    { style: null, id: 'body' },
    { style: 'fontFamily(mono)', id: 'mono' },
    { style: 'fontFamily(display)', id: 'display' },
  ]
  // Compile-valid closed width tokens (lint rejects out-of-scale w= values).
  const widths = ['32', '40', '48', '56', '64']
  const out = []
  for (let i = 0; i < count; i++) {
    const script = pick(rand, scripts)
    const size = pick(rand, sizes)
    const fam = pick(rand, families)
    const w = pick(rand, widths)
    const text = textFor(script, rand) || TEXTS.latin_short
    const long = chance(rand, 0.4)
      ? text + ' ' + TEXTS.latin_long
      : text
    // Fixed-width column; font via style.* (class= is banned: sail-no-class).
    const sx = ['pad(2)', `fontSize(${size})`, fam.style].filter(Boolean).join('.')
    const source = `<column w="${w}" sx={style.${sx}}>
  <span>${escapeXml(long)}</span>
</column>`
    out.push({
      id: `t${String(i).padStart(5, '0')}`,
      source,
      dims: {
        text_script: script,
        text_size: size,
        text_family: fam.id,
        container_w: String(w),
        text_length: long.length > 80 ? 'long' : 'medium',
      },
      family: 'text',
      text: long,
      container_w: w,
    })
  }
  return out
}

function escapeXml(s) {
  return String(s)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
}

/**
 * Error-class positive/negative generators (work item 4).
 * Height/width use closed scale tokens (style.h(N) → N×4px).
 * Child overflow requires shrink(0) so flex does not collapse the child into the parent.
 */
export function generateErrorCases(rand, countPerClass = 80) {
  const out = []
  let i = 0

  // --- silent-clip POSITIVE: h(0)+overflow(hidden)+sized child ---
  // Note: these fail `sail compile` (Error severity) by design — LayoutCheck still runs.
  // Browser truth uses hand-lowered HTML matching compiler class emit.
  for (let n = 0; n < countPerClass; n++) {
    const childH = pick(rand, ['8', '12', '16', '20', '24'])
    const source = `<column w="full" sx={style.h(0).overflow(hidden)}>
  <div shrink="0" sx={style.h(${childH})}><span>silent clip content ${n}</span></div>
</column>`
    out.push({
      id: `e-sc-pos-${String(i++).padStart(4, '0')}`,
      source,
      html: `<div class="sail-col sail-ai-stretch s-wfull s-h0 s-overflowhidden"><div class="s-h${childH} s-shrink0"><span>silent clip content ${n}</span></div></div>`,
      family: 'error',
      class_id: 'silent-clip',
      expected: true,
      label: 'positive',
    })
  }

  // --- silent-clip POSITIVE alt: fixed short h + overflow hidden + tall unshrinkable child ---
  for (let n = 0; n < Math.floor(countPerClass / 2); n++) {
    const source = `<column w="full" sx={style.h(8).overflow(hidden)}>
  <div shrink="0" sx={style.h(24)}><span>${TEXTS.latin_long}</span></div>
</column>`
    out.push({
      id: `e-sc-pos2-${String(i++).padStart(4, '0')}`,
      source,
      html: `<div class="sail-col sail-ai-stretch s-wfull s-h8 s-overflowhidden"><div class="s-h24 s-shrink0"><span>${escapeXml(TEXTS.latin_long)}</span></div></div>`,
      family: 'error',
      class_id: 'silent-clip',
      expected: true,
      label: 'positive-overflow-hidden-tall-child',
    })
  }

  // --- silent-clip NEGATIVE: healthy column ---
  for (let n = 0; n < countPerClass; n++) {
    const source = `<column gap="2" w="full" sx={style.pad(2)}>
  <span>Healthy ${n}</span>
  <span>${TEXTS.latin_short}</span>
</column>`
    out.push({
      id: `e-sc-neg-${String(i++).padStart(4, '0')}`,
      source,
      family: 'error',
      class_id: 'silent-clip',
      expected: false,
      label: 'negative',
    })
  }

  // --- silent-clip NEGATIVE: overflow auto (scroll surfaces; not silent-clip Error class) ---
  for (let n = 0; n < Math.floor(countPerClass / 2); n++) {
    const source = `<column w="full" sx={style.h(8).overflow(auto)}>
  <div shrink="0" sx={style.h(24)}><span>scrollable not silent ${n}</span></div>
</column>`
    out.push({
      id: `e-sc-neg2-${String(i++).padStart(4, '0')}`,
      source,
      family: 'error',
      class_id: 'silent-clip',
      expected: false,
      label: 'negative-overflow-auto',
    })
  }

  // --- child-overflows-parent POSITIVE: fixed parent, larger unshrinkable child, overflow visible ---
  for (let n = 0; n < countPerClass; n++) {
    const source = `<column w="full" sx={style.h(8).overflow(visible)}>
  <div shrink="0" sx={style.h(24)}><span>overflows parent ${n}</span></div>
</column>`
    out.push({
      id: `e-co-pos-${String(i++).padStart(4, '0')}`,
      source,
      family: 'error',
      class_id: 'child-overflows-parent',
      expected: true,
      label: 'positive',
    })
  }

  // --- child-overflows-parent NEGATIVE: child fits ---
  for (let n = 0; n < countPerClass; n++) {
    const source = `<column w="full" sx={style.h(24).overflow(visible).pad(2)}>
  <div sx={style.h(8)}><span>fits ${n}</span></div>
</column>`
    out.push({
      id: `e-co-neg-${String(i++).padStart(4, '0')}`,
      source,
      family: 'error',
      class_id: 'child-overflows-parent',
      expected: false,
      label: 'negative',
    })
  }

  // --- height-chain / zero-main-axis POSITIVE: h(0) without overflow hidden ---
  for (let n = 0; n < countPerClass; n++) {
    const source = `<column w="full" sx={style.h(0)}>
  <div shrink="0" sx={style.h(16)}><span>zero main ${n}</span></div>
</column>`
    out.push({
      id: `e-hc-pos-${String(i++).padStart(4, '0')}`,
      source,
      family: 'error',
      class_id: 'height-chain',
      expected: true,
      label: 'positive',
    })
  }

  // --- height-chain NEGATIVE: proper height ---
  for (let n = 0; n < countPerClass; n++) {
    const source = `<column gap="2" w="full" sx={style.pad(4)}>
  <span>chain ok ${n}</span>
  <span>${TEXTS.latin_medium}</span>
</column>`
    out.push({
      id: `e-hc-neg-${String(i++).padStart(4, '0')}`,
      source,
      family: 'error',
      class_id: 'height-chain',
      expected: false,
      label: 'negative',
    })
  }

  // --- height-chain NEGATIVE: empty children (should NOT fire after filter) ---
  for (let n = 0; n < Math.floor(countPerClass / 4); n++) {
    const source = `<column gap="0" w="full"><div></div></column>`
    out.push({
      id: `e-hc-neg2-${String(i++).padStart(4, '0')}`,
      source,
      family: 'error',
      class_id: 'height-chain',
      expected: false,
      label: 'negative-empty-children',
    })
  }

  return out
}

/** Paint cases: L1 hosts + layout shells with region styles. */
export function generatePaintCases(rand, count) {
  const out = []
  for (let i = 0; i < count; i++) {
    const variant = pick(rand, ['chrome-row', 'cards', 'column-pad', 'button-row'])
    let source
    switch (variant) {
      case 'chrome-row':
        source = `<column gap="4" w="full" sx={style.pad(4)}>
  <row gap="2" w="full" align="center">
    <button>Primary ${i}</button>
    <badge>Badge</badge>
  </row>
  <card w="full">
    <span slot="header">Header ${i}</span>
    <span>Card body with latin text.</span>
  </card>
</column>`
        break
      case 'cards':
        source = `<column gap="3" w="full" sx={style.pad(4)}>
  <card w="full"><span>A ${i}</span></card>
  <card w="full"><span>B ${i}</span></card>
</column>`
        break
      case 'column-pad':
        source = `<column gap="2" w="full" sx={style.pad(4)}>
  <span>Paint shell ${i}</span>
  <span>${TEXTS.latin_medium}</span>
</column>`
        break
      default:
        source = `<row gap="2" w="full" sx={style.pad(4)}>
  <button>One</button>
  <button>Two ${i}</button>
  <badge>n</badge>
</row>`
    }
    out.push({
      id: `p${String(i).padStart(5, '0')}`,
      source,
      dims: { paint_variant: variant },
      family: 'paint',
    })
  }
  return out
}

/**
 * Generate the full corpus for a seed.
 * @param {number} seed
 * @param {{ geometry?: number, text?: number, errorsPerClass?: number, paint?: number }} sizes
 */
export function generateCorpus(seed, sizes = {}) {
  const {
    geometry = 2000,
    text = 240,
    errorsPerClass = 80,
    paint = 120,
  } = sizes
  const rand = mulberry32(seed)
  const cases = []
  for (let i = 0; i < geometry; i++) cases.push(generateCase(rand, i))
  cases.push(...generateTextCases(mulberry32(seed ^ 0x1111), text))
  cases.push(...generateErrorCases(mulberry32(seed ^ 0x2222), errorsPerClass))
  cases.push(...generatePaintCases(mulberry32(seed ^ 0x3333), paint))
  return { seed, sizes: { geometry, text, errorsPerClass, paint }, cases }
}
