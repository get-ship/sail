// TESTS-FIRST — FOUNDATIONAL slice: the <sui-media-csv> renderer (the ONE
// renderer that proves the seam — CSV parse → render via the committed
// sui-table).
//
// Status: RED. The renderer source + controller are greenfield:
//   - `../src/sui-media-csv/sui-media-csv.sui`
//   - `../src/sui-media-csv/sui-media-csv.controller.ts`
// Neither exists yet, so `readFileSync`/the controller import throws and every
// spec errors RED. DO NOT add the impl here. DO NOT weaken into passing stubs.
//
// Harness: `renderComponent` compiles the renderer `.sui` under the
// `sui-media-csv` tag; the composed committed `sui-table` (+ its footer's
// sui-pagination, + sui-empty-state) are registered via their primitives
// `.compiled.js` (sui-pagination's nav buttons are bare <button>s, dissolved
// from <sui-button> — no separate registration needed). The CSV renderer
// parses text and feeds the committed engine — the rendered grid is the proof
// the composition seam holds.
//
// Authority: build-spec Slice 4 (sui-media-csv composes sui-table; NO
// density/dense on the composed table), §"Backbone #2 — ViewerRenderer
// contract" (read-only, source-in-never-fetch, sui-viewer-load/-error),
// behavioral specs #6 (CSV → sui-table, one row per data row, NO density attr)
// and #7 (each arm fires sui-viewer-load).

import { readFileSync } from 'node:fs'
import { afterEach, describe, expect, it } from 'vitest'
import { renderComponent } from '@ship/test-setup'
// The composed committed grid + the primitives its data footer / empty state
// pull in. Cross-pack relative imports into @ship/sui-primitives's src tree:
// the compiled-js shim resolves the sibling `.sui` by path (primitives'
// package `exports` only maps `.`).
import '../../primitives/src/sui-table/sui-table.compiled.js'
import '../../primitives/src/sui-pagination/sui-pagination.compiled.js'
import '../../primitives/src/sui-empty-state/sui-empty-state.compiled.js'
import '../../primitives/src/sui-input/sui-input.compiled.js'
import '../../primitives/src/sui-icon/sui-icon.compiled.js'
// RED: greenfield CSV renderer controller.
import csvRenderer from '../src/sui-media-csv/sui-media-csv.controller'

const SOURCE = readFileSync(
  new URL('../src/sui-media-csv/sui-media-csv.sui', import.meta.url),
  'utf8',
)

interface ViewerSource {
  contentType?: string
  filename?: string
  text?: string
}

const shadow = (el: HTMLElement): ShadowRoot => el.shadowRoot as ShadowRoot

/** The composed sui-table host inside the renderer's shadow. */
const innerTable = (el: HTMLElement): HTMLElement | null =>
  shadow(el).querySelector('sui-table')

/** Data rows rendered by the composed engine-backed sui-table. */
const dataRows = (el: HTMLElement): NodeListOf<Element> => {
  const t = innerTable(el)
  return (t?.shadowRoot ?? shadow(el)).querySelectorAll('[data-row-id]')
}

const live: Array<{ unmount(): void }> = []
afterEach(() => {
  for (const v of live.splice(0)) {
    try {
      v.unmount()
    } catch {
      /* best-effort */
    }
  }
})

/** Mount the CSV renderer with text supplied via the `prop:src` identity
 *  property (a resolved ViewerSource — the renderer never fetches). */
function mountCsv(src: ViewerSource) {
  const v = renderComponent(SOURCE, 'sui-media-csv', { controller: csvRenderer })
  ;(v.el as unknown as { src: ViewerSource }).src = src
  return v
}

describe('<sui-media-csv> — registration + composition', () => {

  it('composes the committed sui-table (does NOT re-invent a grid)', async () => {
    const v = mountCsv({ contentType: 'text/csv', text: 'a,b\n1,2' })
    live.push(v)
    await v.flush()
    expect(innerTable(v.el)).not.toBeNull()
  })
})

describe('<sui-media-csv> — CSV → sui-table render (#6)', () => {
  it('renders one data row per CSV data row (header row → columns)', async () => {
    const v = mountCsv({ contentType: 'text/csv', text: 'name,age\nAlice,30\nBob,25' })
    live.push(v)
    await v.flush()
    // Two data rows (header consumed as columns).
    expect(dataRows(v.el).length).toBe(2)
  })

  it('auto-detects tab-separated content (text/tab-separated-values)', async () => {
    const v = mountCsv({ contentType: 'text/tab-separated-values', text: 'a\tb\n1\t2\n3\t4' })
    live.push(v)
    await v.flush()
    expect(dataRows(v.el).length).toBe(2)
  })

  it('accepts the text via the type="string" SSR/declarative fallback prop', async () => {
    const v = renderComponent(SOURCE, 'sui-media-csv', {
      controller: csvRenderer,
      attrs: { text: 'name\nAlice\nBob' },
    })
    live.push(v)
    await v.flush()
    expect(dataRows(v.el).length).toBe(2)
  })
})

describe('<sui-media-csv> — density is the platform axis (#11)', () => {
  it('sets NEITHER density NOR dense on the composed sui-table', async () => {
    const v = mountCsv({ contentType: 'text/csv', text: 'a,b\n1,2' })
    live.push(v)
    await v.flush()
    const t = innerTable(v.el)
    expect(t).not.toBeNull()
    // The pre-existing sui-table density/dense props must NOT be propagated —
    // rows inherit the --density-* channel via <html data-density>.
    expect(t!.hasAttribute('density')).toBe(false)
    expect(t!.hasAttribute('dense')).toBe(false)
  })

  it('exposes no density/dense prop on the renderer itself', async () => {
    const v = mountCsv({ contentType: 'text/csv', text: 'a,b\n1,2' })
    live.push(v)
    await v.flush()
    const proto = Object.getPrototypeOf(v.el) as object
    expect(Object.getOwnPropertyNames(proto)).not.toContain('density')
    expect(Object.getOwnPropertyNames(proto)).not.toContain('dense')
  })
})

describe('<sui-media-csv> — ViewerRenderer lifecycle events (#7)', () => {
  it('fires sui-viewer-load on a successful parse', async () => {
    const v = renderComponent(SOURCE, 'sui-media-csv', { controller: csvRenderer })
    live.push(v)
    let loads = 0
    v.el.addEventListener('sui-viewer-load', () => loads++)
    ;(v.el as unknown as { src: ViewerSource }).src = { contentType: 'text/csv', text: 'a,b\n1,2' }
    await v.flush()
    expect(loads).toBeGreaterThan(0)
  })

  it('the load event bubbles + crosses the shadow boundary (composed)', async () => {
    const v = renderComponent(SOURCE, 'sui-media-csv', { controller: csvRenderer })
    live.push(v)
    let caught: CustomEvent | null = null
    v.root.addEventListener('sui-viewer-load', (e) => { caught = e as CustomEvent })
    ;(v.el as unknown as { src: ViewerSource }).src = { contentType: 'text/csv', text: 'a,b\n1,2' }
    await v.flush()
    expect(caught).not.toBeNull()
  })

  it('fires sui-viewer-error {message} on malformed input', async () => {
    const v = renderComponent(SOURCE, 'sui-media-csv', { controller: csvRenderer })
    live.push(v)
    let caught: CustomEvent | null = null
    v.el.addEventListener('sui-viewer-error', (e) => { caught = e as CustomEvent })
    // A single unterminated quote is unparseable RFC-4180.
    ;(v.el as unknown as { src: ViewerSource }).src = { contentType: 'text/csv', text: 'a,b\n"unterminated' }
    await v.flush()
    expect(caught).not.toBeNull()
    expect((caught as unknown as CustomEvent).detail).toHaveProperty('message')
  })
})
