// Slice 3 — behavioral spec for <sui-prop-matrix>, the Storybook-style state
// sweep (build-spec §3 / §The viewing components #3).
//
// TESTS-FIRST / RED: this file imports the not-yet-authored tag's
// `<tag>.compiled.js` (emitted on demand from the sibling `.sui` by the test
// shim) and its pure `matrix.ts` derivation helpers — so the suite is RED by
// construction until the impl lands.
//
// The matrix COMPOSES the committed `sui-render`: each cell is its own
// `<sui-render>` (component-mode, that cell's props), so the test registers
// BOTH compiled tags — the authority/charts convention: import the tag's
// `<tag>.compiled.js` + any composed tag's `compiled.js` (the real
// `customElements.define`, so the shadow root + lifecycle behave as in the
// browser). The matrix never rebuilds the engine; it derives axes from the
// CEM, builds the cartesian grid, and wires per-cell `sui-render` children
// that share ONE injected `compile()`.
//
// All structured data (CEM `attributes[]`, the bound `compile()`, the source
// string, the active prop-values, the slot markup) arrives via `prop:*`
// identity — assigned as JS PROPERTIES on the host — NEVER an object/json
// attribute (the `sui-table` rule; the host owns discovery). Scalar control
// props (`component-name`) ride attributes.

import { afterEach, describe, expect, it } from 'vitest'
import { compileSui } from '@ship/sui-dom/test-utils'

// RED: the Slice-3 deliverables. `sui-prop-matrix.compiled.js` registers the
// tag; `sui-render.compiled.js` brings the committed engine the cells compose;
// `@ship/sui-primitives` registers the composed sui-error-boundary /
// .sui-empty-state. The pure helpers live in `matrix.ts`.
import '@ship/sui-primitives'
import '../src/sui-render/sui-render.compiled.js'
import '../src/sui-prop-matrix/sui-prop-matrix.compiled.js'
import { deriveAxes, buildCells, type CemAttr } from '../src/sui-prop-matrix/matrix'

const TAG = 'sui-prop-matrix'

// ---------------------------------------------------------------------------
// The bound compile() the host injects, built over the SAME production compiler
// (`compileSui`) the engine seam uses — wrapped to the engine's discriminated
// result shape. NO fetch, NO path, NO daemon — pure.
// ---------------------------------------------------------------------------
type CompileResult =
  | { kind: 'ok'; js: string; css: string | null; dts: string | null; ssrHtml: string | null }
  | { kind: 'err'; message: string; line: number | null; col: number | null; category: string | null }

function makeBoundCompile(): (source: string, opts?: Record<string, unknown>) => CompileResult {
  return (source, opts = {}) => {
    try {
      const c = compileSui(source, opts as never)
      return { kind: 'ok', js: c.js, css: c.css ?? null, dts: c.dts ?? null, ssrHtml: c.ssrHtml ?? null }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      const m = /\((\d+):(\d+)\)/.exec(msg)
      return { kind: 'err', message: msg, line: m ? Number(m[1]) : null, col: m ? Number(m[2]) : null, category: 'compile' }
    }
  }
}

// ---------------------------------------------------------------------------
// CEM fixtures — the `attributes[]` shape the compiler emits (cem.rs):
// `{ name, type:{text}, default?, values? }`. The matrix reads `values[]` to
// pick axes; canonical enum names (variant/tone/size) drive the derivation.
// ---------------------------------------------------------------------------
const VARIANT_TONE: CemAttr[] = [
  { name: 'variant', type: { text: "'solid' | 'soft' | 'ghost'" }, default: 'solid', values: ['solid', 'soft', 'ghost'] },
  { name: 'tone', type: { text: "'default' | 'primary' | 'destructive'" }, default: 'default', values: ['default', 'primary', 'destructive'] },
  { name: 'size', type: { text: "'sm' | 'md' | 'lg'" }, default: 'md', values: ['sm', 'md', 'lg'] },
  { name: 'label', type: { text: 'string' }, default: '' },
]
const TONE_ONLY: CemAttr[] = [
  { name: 'tone', type: { text: "'default' | 'primary'" }, default: 'default', values: ['default', 'primary'] },
  { name: 'label', type: { text: 'string' }, default: '' },
]
const SIZE_ONLY: CemAttr[] = [
  { name: 'size', type: { text: "'sm' | 'lg'" }, default: 'sm', values: ['sm', 'lg'] },
]
const NO_ENUM: CemAttr[] = [
  { name: 'label', type: { text: 'string' }, default: '' },
  { name: 'disabled', type: { text: 'boolean' }, default: 'false' },
]

// A trivially-valid component-mode source whose declared scalar props match the
// matrix axes, so each cell mounts a real specimen reflecting the cell props.
const SPECIMEN_TAG = 'sui-matrix-specimen'
const SPECIMEN_SRC =
  `<prop name="variant" values="solid|soft|ghost" default="solid" reflect />\n` +
  `<prop name="tone" values="default|primary|destructive" default="default" reflect />\n` +
  `<prop name="size" values="sm|md|lg" default="md" reflect />\n` +
  `<button data-test="probe">{variant}-{tone}</button>`

// ---------------------------------------------------------------------------
// Mount via the real custom element (the charts/authority convention): create
// the tag, assign scalar attrs + structured JS PROPERTIES, append to the body
// so connectedCallback attaches the shadow root, then flush.
// ---------------------------------------------------------------------------
interface MatrixProps {
  compile?: unknown
  props?: CemAttr[]
  source?: string
  active?: Record<string, unknown>
  slotContent?: string
}
let host: HTMLElement | null = null
function mountMatrix(attrs: Record<string, string> = {}, props: MatrixProps = {}): HTMLElement {
  const el = document.createElement(TAG)
  for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, v)
  // Structured/identity props are JS PROPERTIES (the `prop:*` path), assigned
  // BEFORE connect so the first render sees them.
  for (const [k, v] of Object.entries(props)) (el as unknown as Record<string, unknown>)[k] = v
  document.body.appendChild(el)
  host = el
  return el
}
const flush = () => new Promise<void>((r) => setTimeout(r, 0))
afterEach(() => {
  host?.remove()
  host = null
})

const root = (el: HTMLElement) => (el.shadowRoot ?? el) as ParentNode
const cells = (el: HTMLElement) => Array.from(root(el).querySelectorAll('[data-matrix-cell]'))

// ===========================================================================
// Pure derivation — deriveAxes / buildCells (extracted studio logic)
// ===========================================================================
describe('matrix.ts — deriveAxes', () => {
  it('picks variant as the primary axis and tone as the secondary', () => {
    const axes = deriveAxes(VARIANT_TONE)
    expect(axes.map((a) => a.name)).toEqual(['variant', 'tone'])
    expect(axes[0].values).toEqual(['solid', 'soft', 'ghost'])
    expect(axes[1].values).toEqual(['default', 'primary', 'destructive'])
  })

  it('falls back to size as the secondary when tone is absent', () => {
    const noTone: CemAttr[] = [
      { name: 'variant', type: { text: "'a' | 'b'" }, values: ['a', 'b'] },
      { name: 'size', type: { text: "'sm' | 'lg'" }, values: ['sm', 'lg'] },
    ]
    const axes = deriveAxes(noTone)
    expect(axes.map((a) => a.name)).toEqual(['variant', 'size'])
  })

  it('uses tone as the SOLE axis when variant is absent', () => {
    const axes = deriveAxes(TONE_ONLY)
    expect(axes.map((a) => a.name)).toEqual(['tone'])
  })

  it('uses size as the sole axis when neither variant nor tone exists', () => {
    const axes = deriveAxes(SIZE_ONLY)
    expect(axes.map((a) => a.name)).toEqual(['size'])
  })

  it('returns no axes when the CEM has no enum props', () => {
    expect(deriveAxes(NO_ENUM)).toEqual([])
  })
})

describe('matrix.ts — buildCells', () => {
  it('builds the FULL cartesian product of a two-axis grid', () => {
    const built = buildCells(deriveAxes(VARIANT_TONE))
    // 3 variants × 3 tones = 9 cells
    expect(built).toHaveLength(9)
    const pairs = built.map((cell) => `${cell.props.variant}/${cell.props.tone}`)
    expect(new Set(pairs).size).toBe(9)
    expect(pairs).toContain('ghost/destructive')
  })

  it('builds one cell per value for a sole axis', () => {
    const built = buildCells(deriveAxes(TONE_ONLY))
    expect(built).toHaveLength(2)
    expect(built.map((cell) => cell.props.tone)).toEqual(['default', 'primary'])
  })

  it('gives every cell a STABLE unique id (the {#for} key)', () => {
    const built = buildCells(deriveAxes(VARIANT_TONE))
    const ids = built.map((cell) => cell.id)
    expect(new Set(ids).size).toBe(ids.length)
    const again = buildCells(deriveAxes(VARIANT_TONE))
    expect(again.map((cell) => cell.id)).toEqual(ids)
  })

  it('returns no cells when there are no axes', () => {
    expect(buildCells([])).toEqual([])
  })
})

// ===========================================================================
// The rendered grid — composes <sui-render> per cell
// ===========================================================================
describe('<sui-prop-matrix> — registration + grid shell', () => {

  it('attaches a shadow root', () => {
    const el = mountMatrix({ 'component-name': SPECIMEN_TAG }, { compile: makeBoundCompile(), props: VARIANT_TONE, source: SPECIMEN_SRC })
    expect(el.shadowRoot).not.toBeNull()
  })

  it('renders one labelled cell per cartesian combination', async () => {
    const el = mountMatrix(
      { 'component-name': SPECIMEN_TAG },
      { compile: makeBoundCompile(), props: VARIANT_TONE, source: SPECIMEN_SRC },
    )
    await flush()
    // 3 variants × 3 tones = 9 cells
    expect(cells(el)).toHaveLength(9)
    const text = (root(el) as Element).textContent ?? ''
    expect(text).toContain('ghost')
    expect(text).toContain('destructive')
  })

  it('shows an empty-state (no cells) when the CEM has no enum props', async () => {
    const el = mountMatrix(
      { 'component-name': SPECIMEN_TAG },
      { compile: makeBoundCompile(), props: NO_ENUM, source: SPECIMEN_SRC },
    )
    await flush()
    expect(cells(el)).toHaveLength(0)
    expect(root(el).querySelector('.sui-empty-state')).not.toBeNull()
  })
})

describe('<sui-prop-matrix> — each cell is its own <sui-render>', () => {
  it('mounts a <sui-render> per cell, sharing the ONE injected compile()', async () => {
    const el = mountMatrix(
      { 'component-name': SPECIMEN_TAG },
      { compile: makeBoundCompile(), props: VARIANT_TONE, source: SPECIMEN_SRC },
    )
    await flush()
    const renders = root(el).querySelectorAll('sui-render')
    expect(renders.length).toBe(9)
    const compiles = new Set(Array.from(renders).map((r) => (r as unknown as Record<string, unknown>).compile))
    expect(compiles.size).toBe(1)
    expect([...compiles][0]).toBeTypeOf('function')
  })

  it('passes each cell its own merged props (cell axis values) to the sui-render', async () => {
    const el = mountMatrix(
      { 'component-name': SPECIMEN_TAG },
      { compile: makeBoundCompile(), props: VARIANT_TONE, source: SPECIMEN_SRC },
    )
    await flush()
    const renders = Array.from(root(el).querySelectorAll('sui-render')) as HTMLElement[]
    const propValueSets = renders.map((r) => (r as unknown as Record<string, unknown>).propValues as Record<string, unknown>)
    expect(propValueSets.every((pv) => pv && 'variant' in pv && 'tone' in pv)).toBe(true)
    const pairs = new Set(propValueSets.map((pv) => `${pv.variant}/${pv.tone}`))
    expect(pairs.size).toBe(9)
  })

  it('drives every cell in component-mode with the shared component-name + source', async () => {
    const el = mountMatrix(
      { 'component-name': SPECIMEN_TAG },
      { compile: makeBoundCompile(), props: VARIANT_TONE, source: SPECIMEN_SRC },
    )
    await flush()
    const renders = Array.from(root(el).querySelectorAll('sui-render')) as HTMLElement[]
    expect(renders.every((r) => r.getAttribute('mode') === 'tag')).toBe(true)
    expect(renders.every((r) => r.getAttribute('component-name') === SPECIMEN_TAG)).toBe(true)
    expect(renders.every((r) => (r as unknown as Record<string, unknown>).source === SPECIMEN_SRC)).toBe(true)
  })

  it('applies composed slot-content to every cell', async () => {
    const SLOT = `<span data-test="cell-slot">x</span>`
    const el = mountMatrix(
      { 'component-name': SPECIMEN_TAG },
      { compile: makeBoundCompile(), props: VARIANT_TONE, source: SPECIMEN_SRC, slotContent: SLOT },
    )
    await flush()
    const renders = Array.from(root(el).querySelectorAll('sui-render')) as HTMLElement[]
    expect(renders.every((r) => (r as unknown as Record<string, unknown>).slots === SLOT)).toBe(true)
  })

  it('{#for} over cells is keyed — cell DOM identity survives a re-derive of the SAME axes', async () => {
    const el = mountMatrix(
      { 'component-name': SPECIMEN_TAG },
      { compile: makeBoundCompile(), props: VARIANT_TONE, source: SPECIMEN_SRC },
    )
    await flush()
    const first = cells(el)[0]
    // Re-assign the SAME CEM (a fresh array, identical content): a keyed {#for}
    // recycles the existing cell nodes rather than tearing the grid down.
    ;(el as unknown as Record<string, unknown>).props = [...VARIANT_TONE]
    await flush()
    expect(cells(el)[0]).toBe(first)
  })
})

describe('<sui-prop-matrix> — active-cell highlight', () => {
  it('marks the cell matching the active prop-values', async () => {
    const el = mountMatrix(
      { 'component-name': SPECIMEN_TAG },
      { compile: makeBoundCompile(), props: VARIANT_TONE, source: SPECIMEN_SRC, active: { variant: 'soft', tone: 'primary' } },
    )
    await flush()
    const active = root(el).querySelectorAll('[data-matrix-cell][data-active]')
    expect(active.length).toBe(1)
    const label = (active[0] as HTMLElement).textContent ?? ''
    expect(label).toContain('soft')
    expect(label).toContain('primary')
  })

  it('moves the highlight reactively when active changes', async () => {
    const el = mountMatrix(
      { 'component-name': SPECIMEN_TAG },
      { compile: makeBoundCompile(), props: VARIANT_TONE, source: SPECIMEN_SRC, active: { variant: 'solid', tone: 'default' } },
    )
    await flush()
    ;(el as unknown as Record<string, unknown>).active = { variant: 'ghost', tone: 'destructive' }
    await flush()
    const active = root(el).querySelectorAll('[data-matrix-cell][data-active]')
    expect(active.length).toBe(1)
    const label = (active[0] as HTMLElement).textContent ?? ''
    expect(label).toContain('ghost')
    expect(label).toContain('destructive')
  })

  it('highlights NO cell when active matches none of the combinations', async () => {
    const el = mountMatrix(
      { 'component-name': SPECIMEN_TAG },
      { compile: makeBoundCompile(), props: VARIANT_TONE, source: SPECIMEN_SRC, active: { variant: 'nonexistent', tone: 'primary' } },
    )
    await flush()
    expect(root(el).querySelectorAll('[data-matrix-cell][data-active]').length).toBe(0)
  })
})

describe('<sui-prop-matrix> — sui-matrix-cell-click', () => {
  it('emits sui-matrix-cell-click {props,cellId} (bubbles + composed) on a cell click', async () => {
    const el = mountMatrix(
      { 'component-name': SPECIMEN_TAG },
      { compile: makeBoundCompile(), props: VARIANT_TONE, source: SPECIMEN_SRC },
    )
    await flush()
    let detail: { props?: Record<string, unknown>; cellId?: string } | null = null
    let bubbles = false
    let composed = false
    el.addEventListener('sui-matrix-cell-click', (e) => {
      detail = (e as CustomEvent).detail as { props?: Record<string, unknown>; cellId?: string }
      bubbles = (e as CustomEvent).bubbles
      composed = (e as CustomEvent).composed
    })
    const cell = cells(el)[0] as HTMLElement
    cell.dispatchEvent(new MouseEvent('click', { bubbles: true, composed: true }))
    await flush()
    expect(detail).not.toBeNull()
    expect(detail!.cellId).toBeTypeOf('string')
    expect(detail!.props).toBeTypeOf('object')
    expect(detail!.props).toHaveProperty('variant')
    expect(detail!.props).toHaveProperty('tone')
    expect(bubbles).toBe(true)
    expect(composed).toBe(true)
  })

  it('the emitted props identify the SPECIFIC cell clicked', async () => {
    const el = mountMatrix(
      { 'component-name': SPECIMEN_TAG },
      { compile: makeBoundCompile(), props: TONE_ONLY, source: SPECIMEN_SRC },
    )
    await flush()
    let detail: { props?: Record<string, unknown> } | null = null
    el.addEventListener('sui-matrix-cell-click', (e) => {
      detail = (e as CustomEvent).detail as { props?: Record<string, unknown> }
    })
    // click the SECOND tone cell (tone=primary)
    const cell = cells(el)[1] as HTMLElement
    cell.dispatchEvent(new MouseEvent('click', { bubbles: true, composed: true }))
    await flush()
    expect(detail!.props!.tone).toBe('primary')
  })
})

describe('<sui-prop-matrix> — per-cell error boundary', () => {
  it('the grid is NOT blanked — every cell brings its own sui-error-boundary (per-cell recovery)', async () => {
    const el = mountMatrix(
      { 'component-name': SPECIMEN_TAG },
      { compile: makeBoundCompile(), props: VARIANT_TONE, source: SPECIMEN_SRC },
    )
    await flush()
    // The grid is intact (all 9 cells present): one bad combination cannot
    // collapse the whole grid because each cell's sui-render is independently
    // boundary-wrapped.
    expect(cells(el)).toHaveLength(9)
    const renders = root(el).querySelectorAll('sui-render')
    const withBoundary = Array.from(renders).filter(
      (r) => ((r.shadowRoot ?? r) as ParentNode).querySelector('sui-error-boundary') != null,
    )
    expect(withBoundary.length).toBe(renders.length)
  })
})
