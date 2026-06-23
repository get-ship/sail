// <sui-tree-grid> — FULL COLUMN FEATURES slice (Wave-2 data-surface pack `sui-tree`).
//
// Beyond the foundational tree-grid (render flattened tree.rows() through the
// column/cell machinery), this slice proves the tree-grid inherits the REUSED
// Table engine's ORTHOGONAL column lane — sort / visibility / resize — with NO
// new engine. The hierarchy IS createTable({ getSubRows, expanded }); the column
// lane is the same `toggleSort` / `setColumnVisibility` / `setColumnSizing` /
// `visibleLeafColumns` / `getColumnSizeVarValue` the sui-table view drives.
//
// What a USER perceives (the behavioral contract):
//   • clicking a `[role=columnheader]` cycles its sort none→asc→desc→none, the
//     header's `aria-sort` reflects (`none`/`ascending`/`descending`), the
//     flattened rows reorder WITHIN each tree level, and `sui-sort {id,direction}`
//     fires (host.emit!, bubbling + composed);
//   • hiding a column (engine `setColumnVisibility`) drops BOTH its header and
//     every row's cell for that column — `visibleLeafColumns()` is the single
//     source the header row + `getVisibleCells()` both read;
//   • resizing a column (a `[part=resize-handle]` drag → engine `setColumnSizing`)
//     reflects the new width through `getColumnSizeVarValue` onto the grid
//     template, honest under virtualization.
//
// The disclosure caret + indent stay pinned to the FIRST visible column even
// after a sort/hide/resize reshuffles the lane.
//
// TESTS-FIRST: the tree-grid view does NOT wire header sort/hide/resize today
// (foundational only renders plain `[role=columnheader]` text + static cells),
// so these specs are RED until the column-feature wiring lands. NO impl written.
// Authority: .artifacts/sui-arc/sui-tree/build-spec.md (Slice 5 "column header
// sort/hide via the REUSED sui-dropdown-menu pattern" + behavioral spec #8),
// reusing the committed createTable column lane (column-mutators.ts/column-api.ts).

import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/sui-tree-grid/sui-tree-grid.compiled.js'
import '../src/sui-icon/sui-icon.compiled.js'
import { createTable, type Table } from '../src/sui-table/engine/create-table'
import type { ColumnDef } from '../src/sui-table/engine/types'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

interface FileNode extends Record<string, unknown> {
  id: string
  label: string
  size: number
  kind: string
  children?: FileNode[]
}

const COLS: ColumnDef<FileNode>[] = [
  { id: 'label', accessorKey: 'label', header: 'Name' },
  { id: 'size', accessorKey: 'size', header: 'Size' },
  { id: 'kind', accessorKey: 'kind', header: 'Kind' },
]

// A two-level tree whose siblings sort visibly WITHIN their parent (Feature #13
// sorts recurse per level): src/{c.ts, a.ts, b.ts} under a root that also holds
// a sibling leaf. Sorting `label` asc orders src's children a < b < c.
const TREE: FileNode[] = [
  {
    id: 'src',
    label: 'src',
    size: 0,
    kind: 'dir',
    children: [
      { id: 'src/c.ts', label: 'c.ts', size: 30, kind: 'ts' },
      { id: 'src/a.ts', label: 'a.ts', size: 10, kind: 'ts' },
      { id: 'src/b.ts', label: 'b.ts', size: 20, kind: 'ts' },
    ],
  },
  { id: 'readme', label: 'README.md', size: 8, kind: 'md' },
]

/** Build the REUSED Table engine (navMode:'cell', real columns) — NO createTree(). */
function makeGrid(
  expanded: Record<string, boolean> | true = true,
  cols: ColumnDef<FileNode>[] = COLS,
): Table<FileNode> {
  return createTable<FileNode>({
    data: TREE,
    columns: cols,
    getRowId: (r) => r.id,
    getSubRows: (r) => r.children,
    enableExpanding: true,
    filterFromLeafRows: true,
    navMode: 'cell',
    initialState: { expanded },
  })
}

function mountGrid(
  expanded: Record<string, boolean> | true = true,
  attrs: Record<string, string | boolean> = {},
  cols: ColumnDef<FileNode>[] = COLS,
) {
  const tree = makeGrid(expanded, cols)
  const m = mount(reg, 'sui-tree-grid', attrs)
  // IDENTITY binding — the engine (carrying the column defs) is a live reference.
  ;(m.el as unknown as { tree: Table<FileNode> }).tree = tree
  return { ...m, tree }
}

const rowIds = (shadow: ShadowRoot | null): string[] =>
  Array.from(shadow!.querySelectorAll('[role="row"][data-node-id]')).map(
    (r) => r.getAttribute('data-node-id') as string,
  )

const headerLabels = (shadow: ShadowRoot | null): string[] =>
  Array.from(shadow!.querySelectorAll('[role="columnheader"]')).map(
    (h) => h.textContent?.trim() ?? '',
  )

const headerFor = (shadow: ShadowRoot | null, label: string): HTMLElement =>
  Array.from(shadow!.querySelectorAll('[role="columnheader"]')).find(
    (h) => h.textContent?.trim() === label,
  ) as HTMLElement

// ── Sort through the reused engine (behavioral spec #8 header sort) ───────────
describe('<sui-tree-grid> — column header SORT via the reused engine', () => {
  it('clicking a columnheader cycles its aria-sort none → ascending → descending → none', async () => {
    const { shadow } = mountGrid(true)
    await flush()
    const name = () => headerFor(shadow, 'Name')
    // No sort initially — every sortable header reports aria-sort=none.
    expect(name().getAttribute('aria-sort')).toBe('none')
    name().click()
    await flush()
    expect(name().getAttribute('aria-sort')).toBe('ascending')
    name().click()
    await flush()
    expect(name().getAttribute('aria-sort')).toBe('descending')
    name().click()
    await flush()
    expect(name().getAttribute('aria-sort')).toBe('none')
  })

  it('header click drives the engine toggleSort (engine getIsSorted reflects)', async () => {
    const { shadow, tree } = mountGrid(true)
    await flush()
    expect(tree.getColumn('label')!.getIsSorted()).toBe(false)
    headerFor(shadow, 'Name').click()
    await flush()
    expect(tree.getColumn('label')!.getIsSorted()).toBe('asc')
  })

  it('sorting reorders the flattened rows WITHIN each tree level (depth-first preserved)', async () => {
    const { shadow, tree } = mountGrid(true)
    await flush()
    // Source order of src's children: c, a, b.
    expect(rowIds(shadow)).toEqual([
      'src',
      'src/c.ts',
      'src/a.ts',
      'src/b.ts',
      'readme',
    ])
    tree.toggleSort('label') // asc
    await flush()
    // src's children now a < b < c, but src stays the parent above them and the
    // README sibling stays at root — the flatten is still depth-first.
    expect(rowIds(shadow)).toEqual([
      'src',
      'src/a.ts',
      'src/b.ts',
      'src/c.ts',
      'readme',
    ])
  })

  it('emits sui-sort {id, direction} on header click, bubbling + composed', async () => {
    const { shadow, container } = mountGrid(true)
    await flush()
    const fn = vi.fn()
    // Listen on the ANCESTOR — only a bubbling, composed event reaches it.
    container.addEventListener('sui-sort', fn)
    headerFor(shadow, 'Size').click()
    await flush()
    expect(fn).toHaveBeenCalled()
    const detail = (fn.mock.calls[0]![0] as CustomEvent).detail
    expect(detail.id).toBe('size')
    expect(detail.direction).toBe('ascending')
  })

  it('a sorted header carries data-sorted so the sort glyph renders', async () => {
    const { shadow } = mountGrid(true)
    await flush()
    const size = () => headerFor(shadow, 'Size')
    expect(size().hasAttribute('data-sorted')).toBe(false)
    size().click()
    await flush()
    expect(size().getAttribute('data-sorted')).toBe('asc')
  })
})

// ── Visibility through the reused engine (Slice 5 hide via column menu) ────────
describe('<sui-tree-grid> — column VISIBILITY via the reused engine', () => {
  it('hiding a column drops its header (visibleLeafColumns is the single source)', async () => {
    const { shadow, tree } = mountGrid(true)
    await flush()
    expect(headerLabels(shadow)).toEqual(['Name', 'Size', 'Kind'])
    tree.setColumnVisibility({ size: false })
    await flush()
    expect(headerLabels(shadow)).toEqual(['Name', 'Kind'])
  })

  it('hiding a column drops that column cell from every row (getVisibleCells)', async () => {
    const { shadow, tree } = mountGrid(true)
    await flush()
    const cellCount = () =>
      (
        shadow!.querySelector('[role="row"][data-node-id="src/a.ts"]') as HTMLElement
      ).querySelectorAll('[role="gridcell"]').length
    expect(cellCount()).toBe(3)
    tree.setColumnVisibility({ kind: false })
    await flush()
    expect(cellCount()).toBe(2)
  })

  it('the disclosure caret + indent stay in the FIRST visible column after a hide', async () => {
    const { shadow, tree } = mountGrid(true)
    await flush()
    // Hiding the leading "Name" column promotes "Size" to first — the caret must
    // follow to the new first visible cell, never strand on a hidden column.
    tree.setColumnVisibility({ label: false })
    await flush()
    const srcRow = shadow!.querySelector('[role="row"][data-node-id="src"]') as HTMLElement
    const cells = srcRow.querySelectorAll('[role="gridcell"]')
    const firstCaret = (cells[0] as HTMLElement).querySelector('[part="caret"] sui-icon')
    expect(firstCaret).not.toBeNull()
    expect((cells[1] as HTMLElement).querySelector('[part="caret"]')).toBeNull()
  })

  it('a column header exposes a hide control (the reused dropdown-menu pattern)', async () => {
    const { shadow } = mountGrid(true)
    await flush()
    // Every hideable column header carries a [data-action="column-menu"] trigger
    // (parity with the sui-table column header menu) so the user can hide it.
    const trigger = headerFor(shadow, 'Kind').querySelector('[data-action="column-menu"]')
    expect(trigger).not.toBeNull()
  })
})

// ── Resize through the reused engine (Feature #8 onChange/onEnd) ───────────────
describe('<sui-tree-grid> — column RESIZE via the reused engine', () => {
  it('exposes a resize handle on each resizable column header', async () => {
    const { shadow } = mountGrid(true)
    await flush()
    const handle = headerFor(shadow, 'Name').querySelector('[part="resize-handle"]')
    expect(handle).not.toBeNull()
  })

  it('resizing a column reflects its width through getColumnSizeVarValue', async () => {
    const { shadow, tree } = mountGrid(true)
    await flush()
    const before = parseFloat(tree.getColumnSizeVarValue('size')!)
    tree.setColumnSizing({ size: before + 100 })
    await flush()
    // Invariant (not the px literal): the resolved size var tracks the engine's
    // committed size — header and body read the SAME source, and the resize
    // delta is applied.
    const resolved = tree.getColumnSizeVarValue('size')!
    expect(parseFloat(resolved)).toBe(before + 100)
    expect(parseFloat(resolved)).toBeGreaterThan(before)
    const head = shadow!.querySelector('[role="treegrid"]') as HTMLElement
    // The grid template consumes the resolved var verbatim, so the rendered
    // track carries whatever value the engine resolved.
    const tpl = head.getAttribute('style') ?? ''
    expect(tpl).toContain(resolved)
  })

  it('a pointer-drag on the resize handle pumps the engine (live onChange)', async () => {
    const { shadow, tree } = mountGrid(true)
    await flush()
    const spy = vi.spyOn(tree, 'setColumnSizing')
    const handle = headerFor(shadow, 'Name').querySelector(
      '[part="resize-handle"]',
    ) as HTMLElement
    handle.dispatchEvent(new MouseEvent('pointerdown', { bubbles: true, clientX: 100 }))
    document.dispatchEvent(new MouseEvent('pointermove', { bubbles: true, clientX: 140 }))
    document.dispatchEvent(new MouseEvent('pointerup', { bubbles: true, clientX: 140 }))
    await flush()
    expect(spy).toHaveBeenCalled()
  })
})

// ── The column lane is the REUSED engine — NO new engine ──────────────────────
describe('<sui-tree-grid> — column features reuse the Table lane (NO new engine)', () => {
  it('the engine factory is createTable (the column lane is its orthogonal lane)', () => {
    // sort/visibility/resize all route through the createTable column mutators —
    // there is no createTree() and no tree-specific column engine.
    expect(typeof createTable).toBe('function')
    const tree = makeGrid(true)
    expect(typeof tree.toggleSort).toBe('function')
    expect(typeof tree.setColumnVisibility).toBe('function')
    expect(typeof tree.setColumnSizing).toBe('function')
  })
})
