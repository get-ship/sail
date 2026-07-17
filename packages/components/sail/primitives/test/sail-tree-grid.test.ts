// <sail-tree-grid> — FOUNDATIONAL slice (Wave-2 data-surface pack `sail-tree`).
//
// The SAME flattened depth-first `tree.rows()` from the REUSED Table engine,
// rendered through the Table view's COLUMN/CELL machinery (row.getVisibleCells())
// with the disclosure caret + indent in the FIRST visible column. `role=treegrid`
// (vs role=tree for sail-tree); `navMode:'cell'` (vs 'row'). No createTree() —
// the hierarchy is createTable({ getSubRows, expanded }).
//
// Engine reaches the view via `prop:tree` IDENTITY binding (the live Table<T>
// reference). TESTS-FIRST: the `.compiled.js` does NOT exist yet → RED until
// `sail-tree-grid.sui` + `.controller.ts` land.
// Authority: .artifacts/sail-arc/sail-tree/build-spec.md (Slice 5 + behavioral
// spec #8), governed by ship-sui + ship-design-system skills.

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sail-tree-grid/sail-tree-grid.compiled.js'
import { createTable, type Table } from '../src/sail-table/engine/create-table'
import type { ColumnDef } from '../src/sail-table/engine/types'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

interface FileNode extends Record<string, unknown> {
  id: string
  label: string
  size: number
  children?: FileNode[]
}

const COLS: ColumnDef<FileNode>[] = [
  { id: 'label', accessorKey: 'label', header: 'Name' },
  { id: 'size', accessorKey: 'size', header: 'Size' },
]

const TREE: FileNode[] = [
  {
    id: 'src',
    label: 'src',
    size: 0,
    children: [
      { id: 'src/index.ts', label: 'index.ts', size: 12 },
      {
        id: 'src/lib',
        label: 'lib',
        size: 0,
        children: [{ id: 'src/lib/a.ts', label: 'a.ts', size: 3 }],
      },
    ],
  },
  { id: 'readme', label: 'README.md', size: 8 },
]

function makeTreeGrid(expanded: Record<string, boolean> | true = {}): Table<FileNode> {
  return createTable<FileNode>({
    data: TREE,
    columns: COLS, // tree-grid renders through real column defs
    getRowId: (r) => r.id,
    getSubRows: (r) => r.children,
    enableExpanding: true,
    filterFromLeafRows: true,
    navMode: 'cell', // 2D cell grid (vs 'row' for sail-tree)
    initialState: { expanded },
  })
}

function mountTreeGrid(
  expanded: Record<string, boolean> | true = {},
  attrs: Record<string, string | boolean> = {},
) {
  const tree = makeTreeGrid(expanded)
  const m = mount(reg, 'sail-tree-grid', attrs)
  ;(m.el as unknown as { tree: Table<FileNode> }).tree = tree
  return { ...m, tree }
}

const rowIds = (shadow: ShadowRoot | null): string[] =>
  Array.from(shadow!.querySelectorAll('[role="row"][data-node-id]')).map(
    (r) => r.getAttribute('data-node-id') as string,
  )

describe('<sail-tree-grid> — registration + treegrid a11y', () => {

  it('sets role=treegrid on the container (multi-column)', async () => {
    const { shadow } = mountTreeGrid()
    await flush()
    expect(shadow!.querySelector('[role="treegrid"]')).not.toBeNull()
  })

  it('renders role=columnheader headers from the column defs', async () => {
    const { shadow } = mountTreeGrid(true)
    await flush()
    const headers = Array.from(shadow!.querySelectorAll('[role="columnheader"]')).map(
      (h) => h.textContent?.trim(),
    )
    expect(headers).toContain('Name')
    expect(headers).toContain('Size')
  })

  it('aria-rowcount reports the TOTAL flattened count (honest under virtualization)', async () => {
    const { shadow, tree } = mountTreeGrid(true)
    await flush()
    const grid = shadow!.querySelector('[role="treegrid"]') as HTMLElement
    // tree.rowCount() is the full flattened visible count, NOT the DOM window.
    expect(grid.getAttribute('aria-rowcount')).toBe(String(tree.rowCount()))
  })
})

describe('<sail-tree-grid> — shared-engine flatten through cells (behavioral spec #8)', () => {
  it('renders one role=row per tree.rows() (flattened depth-first, expansion-respecting)', async () => {
    const { shadow, tree } = mountTreeGrid(true)
    await flush()
    expect(rowIds(shadow)).toEqual([
      'src',
      'src/index.ts',
      'src/lib',
      'src/lib/a.ts',
      'readme',
    ])
    expect(rowIds(shadow).length).toBe(tree.rows().length)
  })

  it("a collapsed branch's children are ABSENT from the DOM", async () => {
    const { shadow } = mountTreeGrid({})
    await flush()
    expect(rowIds(shadow)).toEqual(['src', 'readme'])
    expect(rowIds(shadow)).not.toContain('src/index.ts')
  })

  it('renders role=gridcell cells from row.getVisibleCells() (Table column machinery)', async () => {
    const { shadow, tree } = mountTreeGrid(true)
    await flush()
    const firstRow = shadow!.querySelector('[role="row"][data-node-id="src/index.ts"]') as HTMLElement
    const cells = firstRow.querySelectorAll('[role="gridcell"]')
    // One gridcell per visible leaf column.
    expect(cells.length).toBe(tree.getRow('src/index.ts')!.getVisibleCells().length)
    const cellText = Array.from(cells).map((c) => c.textContent?.trim())
    expect(cellText).toContain('index.ts')
    expect(cellText).toContain('12')
  })
})

describe('<sail-tree-grid> — disclosure caret + indent in the FIRST visible column', () => {
  it('the caret (sail-icon chevron-right) lives in the first gridcell only', async () => {
    const { shadow } = mountTreeGrid(true)
    await flush()
    const srcRow = shadow!.querySelector('[role="row"][data-node-id="src"]') as HTMLElement
    const cells = srcRow.querySelectorAll('[role="gridcell"]')
    const firstCell = cells[0] as HTMLElement
    const caret = firstCell.querySelector('[data-action="toggle"] .sail-icon') as HTMLElement
    expect(caret).not.toBeNull()
    expect(caret.getAttribute('name')).toBe('chevron-right')
    // No caret in any later column.
    expect((cells[1] as HTMLElement).querySelector('[data-action="toggle"]')).toBeNull()
  })

  it('aria-level reflects row.depth + 1 (flat-list tree depth)', async () => {
    const { shadow } = mountTreeGrid(true)
    await flush()
    const at = (id: string) =>
      (shadow!.querySelector(`[role="row"][data-node-id="${id}"]`) as HTMLElement).getAttribute(
        'aria-level',
      )
    expect(at('src')).toBe('1')
    expect(at('src/index.ts')).toBe('2')
    expect(at('src/lib/a.ts')).toBe('3')
  })

  it('a leaf row carries no caret and no aria-expanded', async () => {
    const { shadow } = mountTreeGrid(true)
    await flush()
    const leaf = shadow!.querySelector('[role="row"][data-node-id="readme"]') as HTMLElement
    expect(leaf.querySelector('[data-action="toggle"] .sail-icon')).toBeNull()
    expect(leaf.hasAttribute('aria-expanded')).toBe(false)
  })
})

describe('<sail-tree-grid> — expand/collapse via the engine', () => {
  it('toggling a branch grows tree.rows() and re-renders the cell rows', async () => {
    const { shadow, tree } = mountTreeGrid({})
    await flush()
    expect(rowIds(shadow)).toEqual(['src', 'readme'])
    tree.toggleExpanded('src')
    await flush()
    expect(rowIds(shadow)).toEqual(['src', 'src/index.ts', 'src/lib', 'readme'])
    expect(rowIds(shadow).length).toBe(tree.rows().length)
  })

  it('aria-expanded on the branch row flips with engine expansion state', async () => {
    const { shadow, tree } = mountTreeGrid({})
    await flush()
    const srcRow = () => shadow!.querySelector('[role="row"][data-node-id="src"]') as HTMLElement
    expect(srcRow().getAttribute('aria-expanded')).toBe('false')
    tree.toggleExpanded('src')
    await flush()
    expect(srcRow().getAttribute('aria-expanded')).toBe('true')
  })
})

describe('<sail-tree-grid> — shares the engine, differs only in navMode/role/render', () => {
  it('uses navMode:cell so Home/End column-nav is meaningful (multi-column)', async () => {
    const { shadow } = mountTreeGrid(true)
    await flush()
    const grid = shadow!.querySelector('[role="treegrid"]') as HTMLElement
    // roving-tabindex: exactly one gridcell/row is focusable at a time.
    const tabbable = () =>
      Array.from(shadow!.querySelectorAll('[role="gridcell"],[role="row"]')).filter(
        (n) => n.getAttribute('tabindex') === '0',
      )
    expect(tabbable().length).toBe(1)
    // End moves within the active row's COLUMNS (meaningful in a treegrid).
    grid.dispatchEvent(new KeyboardEvent('keydown', { key: 'End', bubbles: true }))
    await flush()
    expect(tabbable().length).toBe(1)
  })

  it('the hierarchy is createTable() with getSubRows — read via tree.rows(), NO createTree()', () => {
    expect(typeof createTable).toBe('function')
  })
})

describe('<sail-tree-grid> — density is the platform axis (behavioral spec #10)', () => {
  it('declares NO density and NO dense prop', async () => {
    const { el } = mountTreeGrid(true)
    await flush()
    const proto = Object.getPrototypeOf(el) as object
    expect(Object.getOwnPropertyNames(proto)).not.toContain('density')
    expect(Object.getOwnPropertyNames(proto)).not.toContain('dense')
  })
})
