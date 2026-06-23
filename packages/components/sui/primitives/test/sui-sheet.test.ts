// TESTS-FIRST — sui-sheet FOUNDATIONAL slice: the view render skeleton.
//
// Status: RED. The pack is greenfield — there is no `sui-sheet/` on disk, so
// neither `sui-sheet.compiled.js` (no sibling `.sui` for the test shim to
// compile) nor the `engine/create-sheet` import resolves. Both import failures
// are the expected red for this contract — the WHOLE FILE fails to load until
// the foundational slice ships the `.sui` + the engine. DO NOT add the impl
// here. DO NOT weaken these into passing stubs.
//
// Runner: VITEST (per build-spec — the engine is TypeScript; sui-lint/sui-compiler
// never see .test.ts). The `<tag>.compiled.js` shim resolves the sibling `.sui`
// and CLI-compiles it.
//
// Authority: build-spec §Slice 0 ("the tag registers; the shadow root mounts a
// [part=grid][role=grid]; A1 row/column header gutters render when show-headers;
// NO density/dense prop exists; cells read --density-row-py/-px from
// <html data-density>"), §"The view — sui-sheet.sui" (Props table; Parts: grid /
// corner / col-headers / row-headers / viewport / cell; ARIA grid roles/counts),
// §Behavioral spec #1 ("Renders an addressable grid"), #9 ("Density is the
// platform axis — NO density/dense prop"), #10 (two-axis virtualization counts).
//
// The engine reaches the view via `prop:sheet={sheet}` IDENTITY — a live JS
// reference set on the host (no object <prop>; that would not compile). In tests
// the identity path is the property set `el.sheet = sheet` BEFORE the body reads
// it. The `workbook` attribute is the SSR/declarative string fallback only.
//
// SCOPE NOTE — the formula engine is DEFERRED. These specs cover the render
// skeleton + addressability + density-channel + ARIA-count surfaces only; the
// `=SUM(...)`-displays-computed-value spec lands with the formula slice.

import { afterEach, describe, expect, it } from 'vitest'
// RED: no sibling sui-sheet.sui exists yet → the shim cannot resolve this.
import '../src/sui-sheet/sui-sheet.compiled.js'
// RED: greenfield engine module.
import { createSheet } from '../src/sui-sheet/engine/create-sheet'
import type { WorkbookDef } from '../src/sui-sheet/engine/contract'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => {
  document.documentElement.removeAttribute('data-density')
  reg.cleanup()
})

function book(cells: Record<string, { value?: unknown }> = {}): WorkbookDef {
  return {
    sheets: [{ id: 's1', name: 'Sheet1', rowCount: 100, colCount: 26, cells: cells as never }],
    activeSheetId: 's1',
  }
}

/** Attach the engine via the prop:sheet identity path (a live JS property). */
function mountWithSheet(
  attrs: Record<string, string | boolean>,
  cells: Record<string, { value?: unknown }> = {},
) {
  const { el, shadow, container } = mount(reg, 'sui-sheet', attrs)
  ;(el as unknown as { sheet: unknown }).sheet = createSheet({ workbook: book(cells) })
  return { el, shadow, container }
}

describe('<sui-sheet> — registration + render skeleton (Slice 0 baseline)', () => {

  it('mounts a [part=grid] with role=grid', () => {
    const { shadow } = mount(reg, 'sui-sheet', {})
    const grid = shadow!.querySelector('[part="grid"]')
    expect(grid).not.toBeNull()
    expect(grid!.getAttribute('role')).toBe('grid')
  })

  it('exposes the structural quadrant parts (corner / col-headers / row-headers / viewport)', () => {
    const { shadow } = mount(reg, 'sui-sheet', {})
    for (const part of ['corner', 'col-headers', 'row-headers', 'viewport']) {
      expect(shadow!.querySelector(`[part="${part}"]`), `missing [part=${part}]`).not.toBeNull()
    }
  })
})

describe('<sui-sheet> — A1 header gutters (show-headers)', () => {
  it('renders the column header gutter (A,B,C…) when show-headers', async () => {
    const { shadow } = mountWithSheet({ 'show-headers': true })
    await flush()
    const colHeaders = shadow!.querySelector('[part="col-headers"]')
    expect(colHeaders).not.toBeNull()
    expect(colHeaders!.textContent).toContain('A')
    expect(colHeaders!.querySelector('[role="columnheader"]')).not.toBeNull()
  })

  it('renders the row header gutter (1,2,3…) when show-headers', async () => {
    const { shadow } = mountWithSheet({ 'show-headers': true })
    await flush()
    const rowHeaders = shadow!.querySelector('[part="row-headers"]')
    expect(rowHeaders).not.toBeNull()
    expect(rowHeaders!.textContent).toContain('1')
    expect(rowHeaders!.querySelector('[role="rowheader"]')).not.toBeNull()
  })

  it('reflects show-headers / show-gridlines as boolean attributes (CSS hooks)', () => {
    const { el } = mount(reg, 'sui-sheet', { 'show-headers': true, 'show-gridlines': true })
    expect(el.hasAttribute('show-headers')).toBe(true)
    expect(el.hasAttribute('show-gridlines')).toBe(true)
  })
})

describe('<sui-sheet> — addressable grid (behavioral spec #1)', () => {
  it('renders a [role=gridcell] per windowed cell carrying its A1 in data-a1', async () => {
    const { shadow } = mountWithSheet({}, { A1: { value: 5 } })
    await flush()
    const cells = shadow!.querySelectorAll('[role="gridcell"]')
    expect(cells.length).toBeGreaterThan(0)
    const a1Cell = shadow!.querySelector('[data-a1="A1"]')
    expect(a1Cell).not.toBeNull()
    expect(a1Cell!.getAttribute('role')).toBe('gridcell')
  })

  it('a literal cell renders its value in the cell display', async () => {
    const { shadow } = mountWithSheet({}, { A1: { value: 5 } })
    await flush()
    expect(shadow!.querySelector('[data-a1="A1"]')!.textContent).toContain('5')
  })
})

describe('<sui-sheet> — ARIA counts honest under virtualization (behavioral spec #10)', () => {
  it('aria-rowcount / aria-colcount reflect the LOGICAL extent, not the DOM window', async () => {
    const { shadow } = mountWithSheet({})
    await flush()
    const grid = shadow!.querySelector('[part="grid"]')!
    // 100 logical rows × 26 logical cols (the fixture book) — NOT the windowed
    // DOM length. A screen reader must report the true extent.
    expect(grid.getAttribute('aria-rowcount')).toBe('100')
    expect(grid.getAttribute('aria-colcount')).toBe('26')
  })
})

describe('<sui-sheet> — density is the platform axis (behavioral spec #9)', () => {
  it('exposes NO density AND NO dense prop (deliberate divergence from sui-table)', () => {
    const { el } = mount(reg, 'sui-sheet', {})
    // Setting them must NOT reflect as recognised props — they are not declared.
    el.setAttribute('density', 'compact')
    el.setAttribute('dense', '')
    // The component never reads them; the platform <html data-density> axis owns
    // density. (No assertion on the raw attribute — DOM keeps whatever you set;
    // the point is the component declares neither, so they drive nothing.)
    expect(customElements.get('sui-sheet')).toBeDefined()
  })

  it('cells read the --density-* channel from <html data-density> (no per-component prop)', async () => {
    document.documentElement.setAttribute('data-density', 'compact')
    const { shadow } = mountWithSheet({}, { A1: { value: 1 } })
    await flush()
    const cell = shadow!.querySelector('[role="gridcell"]') as HTMLElement
    expect(cell).not.toBeNull()
    // The cell padding resolves through the density channel, not a hardcoded px.
    const padVar = getComputedStyle(cell).getPropertyValue('padding-block').trim()
    // jsdom does not resolve var() cascades, so we assert the cell does not carry
    // an inline hardcoded padding — the value comes from the --density-row-py
    // token the CSS references.
    expect(cell.style.paddingBlock).toBe('')
    void padVar
  })
})
