// Slice 5 — behavioral spec for <sui-component-viewer>, the COMPOSITION primitive
// (build-spec §The viewing components #6 / Slice 5 / behavioral specs #3,#10,#11).
//
// TESTS-FIRST / RED: this file imports the not-yet-authored tag's
// `sui-component-viewer.compiled.js` + its co-located `lockstep.ts` helper (the
// Slice-5 deliverables) — so the suite is RED by construction until the impl
// lands.
//
// The viewer COMPOSES the four committed tags (sui-render + sui-prop-knobs +
// sui-prop-matrix + sui-source-panel) + the folded props/slots {#for} + an
// internal sui-error-boundary + sui-tabs/sui-tab-panel. It does NOT rebuild any
// of them — the authority/charts convention: import the tag's compiled.js AND
// every composed tag's compiled.js so the real `customElements.define` runs and
// the shadow root + lifecycle behave as in the browser.
//
// All structured data (the live source signal, the CEM `attributes[]`, the
// bound `compile()`, slots/examples) arrives via `prop:*` IDENTITY — assigned
// as JS PROPERTIES on the host — NEVER an object/json attribute (the `sui-table`
// rule; the host owns discovery). Scalar control props (`component-name`,
// `viewport-width`) ride attributes.

import { readFileSync, readdirSync } from 'node:fs'
import { resolve } from 'node:path'
import { afterEach, describe, expect, it } from 'vitest'
import { compileSui } from '@ship/sui-dom/test-utils'
import { signal } from '@ship/sui-runtime'

// RED: the Slice-5 deliverables register the tag + bring the composed engine /
// controls. `@ship/sui-primitives` registers sui-error-boundary / sui-tabs /
// sui-tab-panel / sui-stack / sui-empty-state / sui-code the viewer composes.
import '@ship/sui-primitives'
import '../src/sui-render/sui-render.compiled.js'
import '../src/sui-prop-knobs/sui-prop-knobs.compiled.js'
import '../src/sui-prop-matrix/sui-prop-matrix.compiled.js'
import '../src/sui-source-panel/sui-source-panel.compiled.js'
import '../src/sui-component-viewer/sui-component-viewer.compiled.js'
import { wireLockstep, degradeMode, type CemAttr } from '../src/sui-component-viewer/lockstep'

const TAG = 'sui-component-viewer'

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

// CEM fixture — the `attributes[]` shape the host bakes from a component's
// `.custom-elements.json`. Canonical enum names (variant/tone/size) so the
// matrix derives a grid; the folded props table reads name/type/default.
const BUTTON_CEM: CemAttr[] = [
  { name: 'variant', type: { text: "'solid' | 'soft' | 'ghost'" }, default: 'solid', values: ['solid', 'soft', 'ghost'] },
  { name: 'tone', type: { text: "'default' | 'primary'" }, default: 'default', values: ['default', 'primary'] },
  { name: 'size', type: { text: "'sm' | 'md' | 'lg'" }, default: 'md', values: ['sm', 'md', 'lg'] },
  { name: 'label', type: { text: 'string' }, default: '' },
  { name: 'disabled', type: { text: 'boolean' }, default: 'false' },
]

// A trivially-valid component-mode source whose declared scalar props match the
// CEM, so the live stage + matrix cells mount real specimens reflecting props.
const SPECIMEN_TAG = 'sui-viewer-specimen'
const SPECIMEN_SRC =
  `<prop name="variant" values="solid|soft|ghost" default="solid" reflect />\n` +
  `<prop name="tone" values="default|primary" default="default" reflect />\n` +
  `<prop name="size" values="sm|md|lg" default="md" reflect />\n` +
  `<button data-test="probe">{variant}-{tone}</button>`

// ===========================================================================
// 1. lockstep.ts — the pure wiring + degradation helpers (the soft-300 split)
// ===========================================================================
describe('lockstep.ts — degradeMode (the no-CEM gate)', () => {
  it('reports degraded when the CEM is absent', () => {
    expect(degradeMode(null)).toBe(true)
    expect(degradeMode(undefined)).toBe(true)
  })
  it('reports degraded when the CEM is empty', () => {
    expect(degradeMode([])).toBe(true)
  })
  it('reports NOT degraded when the CEM has props', () => {
    expect(degradeMode(BUTTON_CEM)).toBe(false)
  })
})

describe('lockstep.ts — wireLockstep (prop-change → shared store)', () => {
  it('mirrors a sui-prop-change detail.values into the shared signal', () => {
    const shared = signal<Record<string, unknown>>({})
    const stop = wireLockstep(shared, (v) => v)
    stop.onPropChange({ name: 'variant', value: 'soft', values: { variant: 'soft' } })
    expect(shared()).toEqual({ variant: 'soft' })
  })

  it('mirrors a sui-matrix-cell-click props into the shared signal (cell→knob round-trip)', () => {
    const shared = signal<Record<string, unknown>>({ variant: 'solid' })
    const stop = wireLockstep(shared, (v) => v)
    stop.onCellClick({ props: { variant: 'ghost', tone: 'primary' }, cellId: 'x' })
    expect(shared()).toEqual({ variant: 'ghost', tone: 'primary' })
  })

  it('seeds the shared signal from a sui-prop-reset (defaults re-seeded)', () => {
    const shared = signal<Record<string, unknown>>({})
    const stop = wireLockstep(shared, (v) => v)
    stop.onReset({ values: { variant: 'solid', size: 'md' } })
    expect(shared()).toEqual({ variant: 'solid', size: 'md' })
  })
})

// ===========================================================================
// 2. The mounted tag — composition + shared-context lockstep + degradation
// ===========================================================================
let host: HTMLElement | null = null
const flush = () => new Promise<void>((r) => setTimeout(r, 0))

interface ViewerProps {
  source?: unknown // a signal OR a string (identity)
  cem?: CemAttr[]
  compile?: unknown
  slots?: string
}
function mountViewer(attrs: Record<string, string> = {}, props: ViewerProps = {}): HTMLElement {
  const el = document.createElement(TAG)
  for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, v)
  for (const [k, v] of Object.entries(props)) (el as unknown as Record<string, unknown>)[k] = v
  document.body.appendChild(el)
  host = el
  return el
}
afterEach(() => {
  host?.remove()
  host = null
})

const root = (el: HTMLElement) => (el.shadowRoot ?? el) as ParentNode

describe('<sui-component-viewer> — registration + composition', () => {

  it('attaches a shadow root', () => {
    const el = mountViewer({ 'component-name': SPECIMEN_TAG }, { compile: makeBoundCompile(), source: SPECIMEN_SRC, cem: BUTTON_CEM })
    expect(el.shadowRoot).not.toBeNull()
  })

  it('composes all four viewing children + an internal sui-error-boundary', async () => {
    const el = mountViewer(
      { 'component-name': SPECIMEN_TAG },
      { compile: makeBoundCompile(), source: SPECIMEN_SRC, cem: BUTTON_CEM },
    )
    await flush()
    const r = root(el)
    expect(r.querySelector('sui-render')).not.toBeNull()
    expect(r.querySelector('sui-prop-knobs')).not.toBeNull()
    expect(r.querySelector('sui-prop-matrix')).not.toBeNull()
    expect(r.querySelector('sui-source-panel')).not.toBeNull()
    expect(r.querySelector('sui-error-boundary')).not.toBeNull()
  })

  it('flips matrix / props / source views via sui-tabs + sui-tab-panel', async () => {
    const el = mountViewer(
      { 'component-name': SPECIMEN_TAG },
      { compile: makeBoundCompile(), source: SPECIMEN_SRC, cem: BUTTON_CEM },
    )
    await flush()
    expect(root(el).querySelector('sui-tabs')).not.toBeNull()
    expect(root(el).querySelectorAll('sui-tab-panel').length).toBeGreaterThanOrEqual(2)
  })
})

describe('<sui-component-viewer> — the embeddable stage (live render)', () => {
  it('embeds with prop:source + prop:compile ALONE (no host catalog) and still renders a live stage', async () => {
    // The minimal embedding: source + compile, no CEM. The view-mode render path
    // needs no CEM — the stage <sui-render> still mounts the specimen.
    const VIEW_SRC = `<button data-test="live">hi</button>`
    const el = mountViewer({}, { compile: makeBoundCompile(), source: VIEW_SRC })
    await flush()
    const render = root(el).querySelector('sui-render') as HTMLElement
    expect(render).not.toBeNull()
    // the live source signal is threaded into the stage render
    expect((render as unknown as Record<string, unknown>).source).toBe(VIEW_SRC)
    expect((render as unknown as Record<string, unknown>).compile).toBeTypeOf('function')
  })

  it('a prop:source flip re-renders the stage (the agentic-live path)', async () => {
    // Source enters as a LIVE signal identity owned by the host. Flipping it
    // pushes the new text into the stage render with no remount of the viewer.
    const src = signal<string>(`<button>one</button>`)
    const el = mountViewer({}, { compile: makeBoundCompile(), source: src })
    await flush()
    const render = root(el).querySelector('sui-render') as HTMLElement
    expect((render as unknown as Record<string, unknown>).source).toBe(`<button>one</button>`)
    src.set(`<button>two</button>`)
    await flush()
    expect((render as unknown as Record<string, unknown>).source).toBe(`<button>two</button>`)
  })
})

describe('<sui-component-viewer> — shared-context lockstep', () => {
  it('publishes the propValues signal under "viewer:propValues" for child sui-prop-* to read', async () => {
    const el = mountViewer(
      { 'component-name': SPECIMEN_TAG },
      { compile: makeBoundCompile(), source: SPECIMEN_SRC, cem: BUTTON_CEM },
    )
    await flush()
    // useShared walks DOM ancestry for the provider key — the viewer publishes
    // it on itself under the stable 'viewer:propValues' key.
    const published = (el as unknown as Record<string, unknown>)['__sui_ctx_viewer:propValues']
    expect(published).toBeDefined()
    expect(published).toHaveProperty('set')
  })

  it('a knob change updates the live render AND the matrix highlight in LOCKSTEP', async () => {
    const el = mountViewer(
      { 'component-name': SPECIMEN_TAG },
      { compile: makeBoundCompile(), source: SPECIMEN_SRC, cem: BUTTON_CEM },
    )
    await flush()
    // Simulate the knobs emitting a prop change (the delegated path the host
    // wires). The viewer mirrors it into the shared store; the stage render
    // (component-mode) gets the propValues, and the matrix's `active` updates.
    const knobs = root(el).querySelector('sui-prop-knobs') as HTMLElement
    knobs.dispatchEvent(
      new CustomEvent('sui-prop-change', {
        detail: { name: 'variant', value: 'ghost', values: { variant: 'ghost', tone: 'default', size: 'md' } },
        bubbles: true,
        composed: true,
      }),
    )
    await flush()
    const stageRender = Array.from(root(el).querySelectorAll('sui-render')).find(
      (r) => (r as HTMLElement).getAttribute('mode') !== 'tag' || !(r as HTMLElement).closest('[data-matrix-cell]'),
    ) as HTMLElement
    const matrix = root(el).querySelector('sui-prop-matrix') as HTMLElement
    const stagePv = (stageRender as unknown as Record<string, unknown>).propValues as Record<string, unknown> | undefined
    const matrixActive = (matrix as unknown as Record<string, unknown>).active as Record<string, unknown> | undefined
    expect(stagePv?.variant ?? matrixActive?.variant).toBe('ghost')
    expect(matrixActive?.variant).toBe('ghost')
  })

  it('re-surfaces a child sui-prop-change for a host that wants to record it', async () => {
    const el = mountViewer(
      { 'component-name': SPECIMEN_TAG },
      { compile: makeBoundCompile(), source: SPECIMEN_SRC, cem: BUTTON_CEM },
    )
    await flush()
    let surfaced = false
    el.addEventListener('sui-prop-change', () => {
      surfaced = true
    })
    const knobs = root(el).querySelector('sui-prop-knobs') as HTMLElement
    knobs.dispatchEvent(
      new CustomEvent('sui-prop-change', {
        detail: { name: 'tone', value: 'primary', values: { tone: 'primary' } },
        bubbles: true,
        composed: true,
      }),
    )
    await flush()
    expect(surfaced).toBe(true)
  })
})

describe('<sui-component-viewer> — the folded props/slots table', () => {
  it('renders one row per prop: name / type / default (— for none)', async () => {
    const el = mountViewer(
      { 'component-name': SPECIMEN_TAG },
      { compile: makeBoundCompile(), source: SPECIMEN_SRC, cem: BUTTON_CEM },
    )
    await flush()
    const rows = root(el).querySelectorAll('[data-prop-row]')
    expect(rows.length).toBe(BUTTON_CEM.length)
    const text = (root(el) as Element).textContent ?? ''
    expect(text).toContain('variant')
    expect(text).toContain('disabled')
    // label has an empty-string default → rendered as an em-dash, not blank.
    expect(text).toContain('—')
  })

  it('shows a "N props · M slots" metaLabel (0 slots on the static path)', async () => {
    const el = mountViewer(
      { 'component-name': SPECIMEN_TAG },
      { compile: makeBoundCompile(), source: SPECIMEN_SRC, cem: BUTTON_CEM },
    )
    await flush()
    const text = (root(el) as Element).textContent ?? ''
    expect(text).toMatch(/5\s*props/)
    expect(text).toMatch(/0\s*slots/)
  })

  it('shows a sui-empty-state in the slots section when no prop:slots is supplied', async () => {
    const el = mountViewer(
      { 'component-name': SPECIMEN_TAG },
      { compile: makeBoundCompile(), source: SPECIMEN_SRC, cem: BUTTON_CEM },
    )
    await flush()
    // Slots are daemon-only on the static path → empty-state, not an error.
    expect(root(el).querySelector('[data-slots-section] sui-empty-state, sui-empty-state[part="slots-empty"]')).not.toBeNull()
  })
})

describe('<sui-component-viewer> — no-CEM degradation (hide, not error)', () => {
  it('with prop:cem absent: renders source + render but HIDES knobs/matrix/props-table', async () => {
    const el = mountViewer({}, { compile: makeBoundCompile(), source: SPECIMEN_SRC })
    await flush()
    const r = root(el)
    // source + live render survive (the agentic-live path)...
    expect(r.querySelector('sui-render')).not.toBeNull()
    expect(r.querySelector('sui-source-panel')).not.toBeNull()
    // ...the CEM-driven surfaces are HIDDEN (the [hidden] attr / :host([hidden])),
    // not absent-and-not-errored. They may be present but hidden, or omitted.
    const knobs = r.querySelector('sui-prop-knobs') as HTMLElement | null
    const matrix = r.querySelector('sui-prop-matrix') as HTMLElement | null
    const propsTable = r.querySelector('[data-props-table]') as HTMLElement | null
    const isHidden = (e: HTMLElement | null) => e == null || e.hasAttribute('hidden') || (e.closest('[hidden]') !== null)
    expect(isHidden(knobs)).toBe(true)
    expect(isHidden(matrix)).toBe(true)
    expect(isHidden(propsTable)).toBe(true)
  })

  it('does NOT throw / error-surface when the CEM is absent (degrade, not error)', async () => {
    const el = mountViewer({}, { compile: makeBoundCompile(), source: SPECIMEN_SRC })
    await flush()
    // The internal sui-error-boundary did not trip into its fallback — the
    // viewer rendered its degraded shell cleanly.
    const r = root(el)
    expect(r.querySelector('[part="error-fallback"]')).toBeNull()
    // and the stage is live, not blanked
    expect(r.querySelector('sui-render')).not.toBeNull()
  })
})

// ===========================================================================
// 3. The read-only-over-source invariant — the no-IDE fence (test-enforced)
// ===========================================================================
describe('read-only-over-source invariant (the no-IDE fence)', () => {
  it('a prop:source signal is NEVER written back by the viewer — flipping it from the host is one-way', async () => {
    // The viewer reads the source signal but never calls `.set` on it. We pass a
    // wrapped signal whose `.set` is spied; the viewer must never invoke it.
    const inner = signal<string>(SPECIMEN_SRC)
    let writes = 0
    const watched = Object.assign(() => inner(), {
      set: (...args: Parameters<typeof inner.set>) => {
        writes++
        return inner.set(...args)
      },
    }) as typeof inner
    const el = mountViewer({ 'component-name': SPECIMEN_TAG }, { compile: makeBoundCompile(), source: watched, cem: BUTTON_CEM })
    await flush()
    // Drive a knob change — the only thing the viewer mutates is propValues, NOT
    // the source signal (no write-back, the no-IDE fence).
    const knobs = root(el).querySelector('sui-prop-knobs') as HTMLElement
    knobs.dispatchEvent(
      new CustomEvent('sui-prop-change', {
        detail: { name: 'variant', value: 'soft', values: { variant: 'soft' } },
        bubbles: true,
        composed: true,
      }),
    )
    await flush()
    expect(writes).toBe(0)
  })

  it('GREP: no .controller.ts / lockstep.ts in the pack calls .set on a source signal', () => {
    // Static guard over the whole pack `src` tree — no controller writes back to
    // the source prop. The source signal is named `source`; a write-back would be
    // `source.set(` / `state.source = ` on the source-derived signal.
    const srcDir = resolve(__dirname, '../src')
    const offenders: string[] = []
    const walk = (dir: string): void => {
      for (const ent of readdirSync(dir, { withFileTypes: true })) {
        const p = resolve(dir, ent.name)
        if (ent.isDirectory()) {
          walk(p)
          continue
        }
        if (!/\.(controller|lockstep)\.ts$/.test(ent.name) && ent.name !== 'lockstep.ts') continue
        const body = readFileSync(p, 'utf8')
        // A write-back to the SOURCE signal specifically. The source signal in
        // every controller is named `source`. Match `source.set(` (the write
        // form) but NOT `setSource` / `someSource.set` of a different field.
        if (/\bsource\.set\s*\(/.test(body)) offenders.push(p)
      }
    }
    walk(srcDir)
    expect(offenders).toEqual([])
  })
})

// ===========================================================================
// 4. Boundary purity — no daemon/SDK coupling anywhere in the pack src tree
// ===========================================================================
describe('boundary purity (the no-backend fence — pack-wide grep)', () => {
  const srcFiles = (): string[] => {
    const srcDir = resolve(__dirname, '../src')
    const out: string[] = []
    const walk = (dir: string): void => {
      for (const ent of readdirSync(dir, { withFileTypes: true })) {
        const p = resolve(dir, ent.name)
        if (ent.isDirectory()) walk(p)
        else if (/\.(ts|sui)$/.test(ent.name)) out.push(p)
      }
    }
    walk(srcDir)
    return out
  }

  it('no pack src file imports @ship/sdk-client, names ship.*, EventSource, or <data>', () => {
    for (const f of srcFiles()) {
      const body = readFileSync(f, 'utf8')
      expect(body, f).not.toContain('@ship/sdk-client')
      expect(body, f).not.toContain('EventSource')
      expect(body, f).not.toMatch(/<data\b/)
      // a `ship.<method>(` call form (not the package name `@ship/...`)
      expect(body, f).not.toMatch(/\bship\.[a-zA-Z]/)
    }
  })

  it('exactly ONE fetch CALL SITE in the pack — the injected wasmUrl in wasm-bootstrap.ts (never a daemon path)', () => {
    // Strip comments before counting so a JSDoc mention of `fetch(...)` is not
    // miscounted as a call site. The invariant is exactly ONE real fetch call,
    // and it must live in wasm-bootstrap.ts.
    const stripComments = (s: string): string =>
      s.replace(/\/\*[\s\S]*?\*\//g, '').replace(/(^|[^:])\/\/[^\n]*/g, '$1')
    let fetchSites = 0
    for (const f of srcFiles()) {
      const code = stripComments(readFileSync(f, 'utf8'))
      const m = code.match(/\bfetch\s*\(/g)
      if (m) {
        fetchSites += m.length
        expect(f, `fetch outside wasm-bootstrap.ts: ${f}`).toMatch(/wasm-bootstrap\.ts$/)
      }
    }
    expect(fetchSites).toBe(1)
  })
})

describe('<sui-component-viewer> — controller purity (grep)', () => {
  it('the controller + lockstep never construct view markup', () => {
    for (const name of ['sui-component-viewer.controller.ts', 'lockstep.ts']) {
      const body = readFileSync(resolve(__dirname, `../src/sui-component-viewer/${name}`), 'utf8')
      expect(body, name).not.toMatch(/\.innerHTML\s*=/)
      expect(body, name).not.toMatch(/\.outerHTML\s*=/)
      expect(body, name).not.toMatch(/\.insertAdjacentHTML\s*\(/)
    }
  })
})
