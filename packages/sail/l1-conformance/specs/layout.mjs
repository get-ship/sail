// L1 conformance spec: the LAYOUT family — <row> <column> <stack> <grid> <center>.
//
// This is L1's fourth mechanism and the only one with no tag and no runtime: the
// preprocessor KEYWORD-LOWERS these to a <div> plus utility classes, and injects
// exactly the rules the file actually used. It gets its own path (rather than
// element mode) for a real reason — layout is the one family that must accept
// DYNAMIC values (`align={pick()}` lowers to a class binding with the full rule
// set injected), and element mode forbids dynamic attributes outright.
//
// The axis IS the name, which is the whole vocabulary:
//   row = x   column = y   stack = z   grid = 2D   (+ center)
//
// So the spec must not hand-copy CSS: it COMPILES a fixture with the real
// compiler, pulls the emitted stylesheet out, and renders the real lowered class
// lists against it. Naming drift between the lowering and the rules fails loudly
// (see `classes-not-in-compiled-css`) instead of silently passing on stale copies.

import { execFileSync } from 'node:child_process'
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))
const ROOT = resolve(HERE, '../../../..')
const TMP = resolve(HERE, '../.layout-fixture')

// The fixture exercises every keyword and every channel the probe asserts on.
const FIXTURE = `<component>{ props: {} }</component>
<column gap="4">
  <row gap="2" align="center" justify="between"><span>a</span><span>b</span></row>
  <column gap="1"><span>a</span><span>b</span></column>
  <stack align="center" justify="center"><span>under</span><span>over</span></stack>
  <grid min="xs" gap="3"><span>1</span><span>2</span><span>3</span><span>4</span></grid>
  <center><span>mid</span></center>
</column>
`

function compiledLayoutCss() {
  mkdirSync(TMP, { recursive: true })
  writeFileSync(resolve(TMP, 'fixture.sail'), FIXTURE)
  execFileSync(resolve(ROOT, 'target/debug/sail-compiler'), ['build', resolve(TMP, 'fixture.sail'), '--out', TMP], {
    stdio: 'pipe',
  })
  const js = readFileSync(resolve(TMP, 'fixture.js'), 'utf8')
  // the emitted module carries its stylesheet as `const __suiStyles = "…"`
  const m = js.match(/const __suiStyles = ("(?:[^"\\]|\\.)*")/)
  if (!m) throw new Error('no __suiStyles in compiled fixture — the emit format changed')
  return JSON.parse(m[1])
}

const CSS = compiledLayoutCss()

// The lowered class lists, verbatim — what the preprocessor turns the keywords
// into. The probe cross-checks each one against CSS so a rename can't slip past.
const ROW = 'sail-row sail-g-2 sail-ai-center sail-jc-between'
const COL = 'sail-col sail-g-1'
const STACKZ = 'sail-stackz sail-ai-center sail-jc-center'
const GRID = 'sail-grid sail-gm-xs sail-g-3'
const CENTER = 'sail-center'

const box = (t) => `<span style="padding:4px 10px;background:var(--tone-muted-border);border-radius:4px">${t}</span>`

export default {
  tag: 'layout',
  width: 760,
  css: CSS,
  rows() {
    return [
      `<div class="${ROW}" data-t="row" style="width:320px">${box('a')}${box('b')}</div>`,
      `<div class="${COL}" data-t="col" style="width:120px">${box('a')}${box('b')}</div>`,
      `<div class="${STACKZ}" data-t="stackz" style="width:140px;height:52px">` +
        `<span data-under style="background:var(--primary);width:100%;height:100%;display:block"></span>` +
        `<span data-over style="padding:2px 8px;background:var(--background);border:1px solid var(--border)">over</span>` +
        `</div>`,
      `<div class="${GRID}" data-t="grid" style="width:560px">${box(1)}${box(2)}${box(3)}${box(4)}</div>`,
      `<div class="${CENTER}" data-t="center" style="width:200px;height:44px">${box('mid')}</div>`,
    ]
  },
  probe() {
    const bad = []
    const cs = (el, p) => getComputedStyle(el)[p]
    const scope = "[data-theme-preview='ship-light'] "
    const q = (t) => document.querySelector(`${scope}[data-t='${t}']`)
    const kids = (t) => [...q(t).children].map((c) => c.getBoundingClientRect())

    // 0. every class the lowering emits must actually EXIST in the compiled CSS.
    // This is the anti-drift check: rename a class in lint/rules/layout.rs without
    // updating the lowering (or vice versa) and this fires immediately.
    const sheet = [...document.styleSheets]
      .flatMap((s) => {
        try {
          return [...s.cssRules].map((r) => r.cssText)
        } catch {
          return []
        }
      })
      .join('\n')
    for (const cls of [...document.querySelectorAll('[data-t]')].flatMap((el) => [...el.classList])) {
      if (!sheet.includes(`.${cls}`)) bad.push({ why: 'classes-not-in-compiled-css', cls })
    }

    // 1. ROW is the X axis: children sit side by side, same top.
    {
      const [a, b] = kids('row')
      if (!(b.left > a.left)) bad.push({ why: 'row-not-horizontal', a: a.left, b: b.left })
      if (Math.abs(a.top - b.top) > 1) bad.push({ why: 'row-children-not-aligned' })
      // gap="2" resolves to the SPACING TOKEN --space-2, exactly. Not a magic
      // number, and not merely "some gap".
      const want = getComputedStyle(q('row')).getPropertyValue('--space-2').trim()
      const got = cs(q('row'), 'columnGap')
      if (!want || got !== want) bad.push({ why: 'gap-not-the-token', got, want })
      if (cs(q('row'), 'justifyContent') !== 'space-between') bad.push({ why: 'row-justify-dead' })
      if (cs(q('row'), 'alignItems') !== 'center') bad.push({ why: 'row-align-dead' })
    }

    // 2. COLUMN is the Y axis: children stack downward, same left.
    {
      const [a, b] = kids('col')
      if (!(b.top > a.top)) bad.push({ why: 'column-not-vertical', a: a.top, b: b.top })
      if (Math.abs(a.left - b.left) > 1) bad.push({ why: 'column-children-not-aligned' })
    }

    // 3. STACK is the Z axis — the reason it exists. Children OVERLAP (same cell),
    // and the box still takes its size from them (a single-cell grid, not
    // position:absolute, which is exactly the distinction that makes it useful).
    {
      const [under, over] = kids('stackz')
      const overlaps =
        over.left < under.right && over.right > under.left && over.top < under.bottom && over.bottom > under.top
      if (!overlaps) bad.push({ why: 'stack-does-not-layer', under, over })
      if (cs(q('stackz'), 'display') !== 'grid') bad.push({ why: 'stack-not-grid' })
      // later sibling paints on top: it must be inside the earlier one's box
      if (!(over.top >= under.top - 1 && over.bottom <= under.bottom + 1))
        bad.push({ why: 'stack-child-escaped-the-cell' })
      if (q('stackz').getBoundingClientRect().height <= 0) bad.push({ why: 'stack-collapsed' })
    }

    // 4. GRID is 2D: auto-fit tracks off the CONTAINER width (no media query), so
    // four children at min=xs land on more than one column AND more than one row.
    {
      const k = kids('grid')
      const cols = new Set(k.map((r) => Math.round(r.left))).size
      const rows_ = new Set(k.map((r) => Math.round(r.top))).size
      if (cols < 2) bad.push({ why: 'grid-single-column', cols })
      if (rows_ < 2) bad.push({ why: 'grid-single-row', rows: rows_ })
      if (cs(q('grid'), 'display') !== 'grid') bad.push({ why: 'grid-not-grid' })
    }

    // 5. CENTER centers on BOTH axes — the layout everyone hand-rolls and half get wrong.
    {
      const p = q('center').getBoundingClientRect()
      const c = kids('center')[0]
      const dx = Math.abs((c.left + c.width / 2) - (p.left + p.width / 2))
      const dy = Math.abs((c.top + c.height / 2) - (p.top + p.height / 2))
      if (dx > 1.5) bad.push({ why: 'center-not-centered-x', dx })
      if (dy > 1.5) bad.push({ why: 'center-not-centered-y', dy })
    }

    // 6. min-width:0 on every box + child. Without it a layout box carries its
    // content's intrinsic width as a flex/grid child and shoves its parent wider —
    // the overflow bug the primitives exist to make structurally impossible.
    for (const t of ['row', 'col', 'grid']) {
      if (cs(q(t), 'minWidth') !== '0px') bad.push({ why: 'no-min-width-0', t })
    }

    return { count: document.querySelectorAll('[data-t]').length, bad: bad.slice(0, 12) }
  },
}
