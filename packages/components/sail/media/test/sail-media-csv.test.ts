// The <sail-media-csv> renderer — CSV parse → the shared table engine → a plain
// <table>.
//
// Harness: `renderComponent` compiles the renderer `.sui` under the
// `sail-media-csv` tag. The renderer parses text into the committed
// `createTable` engine (Backbone #1) and renders that engine's columns/rows as
// a native <table>, which the framework's contracts sheet already styles — so
// the rendered grid is the proof the engine seam holds, and there is no grid
// component to register.
//
// The component is LIGHT DOM and hand-authors no classes: the queries are the
// <table> tag and the row's `data-row-id` (the engine's row identity).
//
// Authority: build-spec Slice 4 (sail-media-csv reuses the table engine; NO
// density/dense on the rendered table), §"Backbone #2 — ViewerRenderer
// contract" (read-only, source-in-never-fetch, sail-viewer-load/-error),
// behavioral specs #6 (CSV → table, one row per data row, NO density attr)
// and #7 (each arm fires sail-viewer-load).

import { readFileSync } from 'node:fs'
import { afterEach, describe, expect, it } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import csvRenderer from '../src/sail-media-csv/sail-media-csv.sui.controller.ts'

const SOURCE = readFileSync(
  new URL('../src/sail-media-csv/sail-media-csv.sui', import.meta.url),
  'utf8',
)

interface ViewerSource {
  contentType?: string
  filename?: string
  text?: string
}

/** The rendered grid — a plain <table>, skinned by the framework contracts. */
const innerTable = (el: HTMLElement): HTMLTableElement | null => el.querySelector('table')

/** Data rows rendered from the engine's row model (one per CSV data row). */
const dataRows = (el: HTMLElement): NodeListOf<Element> =>
  el.querySelectorAll('[data-row-id]')

/** Header cells rendered from the engine's visible columns. */
const headerCells = (el: HTMLElement): NodeListOf<Element> => el.querySelectorAll('th')

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
  const v = renderComponent(SOURCE, 'sail-media-csv', { controller: csvRenderer })
  ;(v.el as unknown as { src: ViewerSource }).src = src
  return v
}

describe('<sail-media-csv> — registration + composition', () => {

  it('renders the engine\'s row model as a table (does NOT re-invent a grid)', async () => {
    const v = mountCsv({ contentType: 'text/csv', text: 'a,b\n1,2' })
    live.push(v)
    await v.flush()
    expect(innerTable(v.el)).not.toBeNull()
    // The header row drives the columns; the engine's column order is the
    // rendered header order.
    expect([...headerCells(v.el)].map((th) => th.textContent)).toEqual(['a', 'b'])
  })
})

describe('<sail-media-csv> — CSV → sail-table render (#6)', () => {
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
    const v = renderComponent(SOURCE, 'sail-media-csv', {
      controller: csvRenderer,
      attrs: { text: 'name\nAlice\nBob' },
    })
    live.push(v)
    await v.flush()
    expect(dataRows(v.el).length).toBe(2)
  })
})

describe('<sail-media-csv> — density is the platform axis (#11)', () => {
  it('sets NEITHER density NOR dense on the rendered table', async () => {
    const v = mountCsv({ contentType: 'text/csv', text: 'a,b\n1,2' })
    live.push(v)
    await v.flush()
    const t = innerTable(v.el)
    expect(t).not.toBeNull()
    // Density is not a component prop here — the table skin reads the
    // --density-* channel via <html data-density>.
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

describe('<sail-media-csv> — ViewerRenderer lifecycle events (#7)', () => {
  it('fires sail-viewer-load on a successful parse', async () => {
    const v = renderComponent(SOURCE, 'sail-media-csv', { controller: csvRenderer })
    live.push(v)
    let loads = 0
    v.el.addEventListener('sail-viewer-load', () => loads++)
    ;(v.el as unknown as { src: ViewerSource }).src = { contentType: 'text/csv', text: 'a,b\n1,2' }
    await v.flush()
    expect(loads).toBeGreaterThan(0)
  })

  it('the load event bubbles + crosses the shadow boundary (composed)', async () => {
    const v = renderComponent(SOURCE, 'sail-media-csv', { controller: csvRenderer })
    live.push(v)
    let caught: CustomEvent | null = null
    v.root.addEventListener('sail-viewer-load', (e) => { caught = e as CustomEvent })
    ;(v.el as unknown as { src: ViewerSource }).src = { contentType: 'text/csv', text: 'a,b\n1,2' }
    await v.flush()
    expect(caught).not.toBeNull()
  })

  it('fires sail-viewer-error {message} on malformed input', async () => {
    const v = renderComponent(SOURCE, 'sail-media-csv', { controller: csvRenderer })
    live.push(v)
    let caught: CustomEvent | null = null
    v.el.addEventListener('sail-viewer-error', (e) => { caught = e as CustomEvent })
    // A single unterminated quote is unparseable RFC-4180.
    ;(v.el as unknown as { src: ViewerSource }).src = { contentType: 'text/csv', text: 'a,b\n"unterminated' }
    await v.flush()
    expect(caught).not.toBeNull()
    expect((caught as unknown as CustomEvent).detail).toHaveProperty('message')
  })
})
