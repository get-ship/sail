// <sail-tree> — RANGE MULTI-SELECT slice (Wave-2 data-surface pack `sail-tree`).
//
// Beyond foundational single/branch selection, this slice proves shift-click
// range selection spans the contiguous run over the DISPLAYED (flattened,
// depth-first, expansion-respecting) tree order — the REUSED Table selection
// lane's `toggleRowSelected(id, undefined, { range })` scoped to `tree.rows()`,
// pivoting around the `selectionAnchor` (the last plain click). NO new engine.
//
// What a USER perceives (the behavioral contract):
//   • a plain click sets the selection anchor; a subsequent shift-click selects
//     every node BETWEEN the anchor and the target in the on-screen order, so the
//     visible run highlights as one block (`data-selected` on each);
//   • the span is taken over the FLATTENED tree order — a deep child between two
//     shallow nodes is inside the run; a node hidden under a collapsed branch is
//     NOT in `tree.rows()` and therefore NOT in the span;
//   • the anchor stays put so extending the shift-click grows/shrinks the run
//     around the same pivot;
//   • each range change emits `sail-node-select {selectedIds}` once (deduped via
//     host.emit!), carrying the full id set.
//
// TESTS-FIRST: range selection over the tree window is not wired in the view
// today (foundational covers single-node + branch tri-state only), so the
// shift-click → span → DOM-highlight → emit path is RED until it lands. NO impl
// written. Authority: .artifacts/sail-arc/sail-tree/build-spec.md (behavioral spec
// #4 "shift-click selects the contiguous span from the anchor over displayed
// order" + Slice 3 selection), reusing the committed selection lane
// (sail-table/engine/selection.ts `selectRange`).

import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/sail-tree/sail-tree.compiled.js'
import { createTable, type Table } from '../src/sail-table/engine/create-table'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

interface FileNode extends Record<string, unknown> {
  id: string
  label: string
  children?: FileNode[]
}

// Flattened (expand-all) displayed order is:
//   src, src/index.ts, src/lib, src/lib/a.ts, src/lib/b.ts, readme, license
// — a deep child (src/lib/a.ts) sits between shallow nodes, which is exactly the
// case a flat range over depth-first order must span correctly.
const TREE: FileNode[] = [
  {
    id: 'src',
    label: 'src',
    children: [
      { id: 'src/index.ts', label: 'index.ts' },
      {
        id: 'src/lib',
        label: 'lib',
        children: [
          { id: 'src/lib/a.ts', label: 'a.ts' },
          { id: 'src/lib/b.ts', label: 'b.ts' },
        ],
      },
    ],
  },
  { id: 'readme', label: 'README.md' },
  { id: 'license', label: 'LICENSE' },
]

function makeTree(expanded: Record<string, boolean> | true = true): Table<FileNode> {
  return createTable<FileNode>({
    data: TREE,
    columns: [],
    getRowId: (r) => r.id,
    getSubRows: (r) => r.children,
    enableExpanding: true,
    enableRowSelection: true,
    filterFromLeafRows: true,
    navMode: 'row',
    initialState: { expanded },
  })
}

function mountTree(
  expanded: Record<string, boolean> | true = true,
  attrs: Record<string, string | boolean> = { selectable: 'multi' },
) {
  const tree = makeTree(expanded)
  const m = mount(reg, 'sail-tree', attrs)
  ;(m.el as unknown as { tree: Table<FileNode> }).tree = tree
  return { ...m, tree }
}

const rowFor = (shadow: ShadowRoot | null, id: string): HTMLElement =>
  shadow!.querySelector(`[data-node-id="${id}"]`) as HTMLElement

/** ids of rows currently carrying [data-selected], in DOM order. */
const selectedIds = (shadow: ShadowRoot | null): string[] =>
  Array.from(shadow!.querySelectorAll('[role="treeitem"][data-selected]')).map(
    (r) => r.getAttribute('data-node-id') as string,
  )

/** Click a row's body (never its caret — that toggles), with optional shiftKey. */
const clickRow = (shadow: ShadowRoot | null, id: string, shift = false): void => {
  const row = rowFor(shadow, id)
  row.dispatchEvent(new MouseEvent('click', { bubbles: true, composed: true, shiftKey: shift }))
}

describe('<sail-tree> — shift-click range select over the displayed tree order', () => {
  it('a plain click selects a single node and sets the anchor', async () => {
    const { shadow } = mountTree(true)
    await flush()
    clickRow(shadow, 'src/index.ts')
    await flush()
    expect(selectedIds(shadow)).toEqual(['src/index.ts'])
  })

  it('shift-click spans the contiguous run from the anchor over the flattened order', async () => {
    const { shadow } = mountTree(true)
    await flush()
    // Anchor on index.ts, then shift-click b.ts → the run index.ts..b.ts is one
    // block over the depth-first order, INCLUDING the deep child a.ts and the
    // intervening branch lib.
    clickRow(shadow, 'src/index.ts')
    await flush()
    clickRow(shadow, 'src/lib/b.ts', true)
    await flush()
    expect(selectedIds(shadow)).toEqual([
      'src/index.ts',
      'src/lib',
      'src/lib/a.ts',
      'src/lib/b.ts',
    ])
  })

  it('the span is taken over displayed order — a deep child between shallow nodes is included', async () => {
    const { shadow, tree } = mountTree(true)
    await flush()
    // Anchor via engine (branch click would toggle expand and drop children).
    tree.toggleRowSelected('src', true)
    await flush()
    clickRow(shadow, 'src/lib/a.ts', true) // target at depth-2
    await flush()
    // Everything from src down to a.ts in flattened order:
    expect(tree.selectedRowIds()).toEqual([
      'src',
      'src/index.ts',
      'src/lib',
      'src/lib/a.ts',
    ])
  })

  it('shift-click extending the range pivots around the SAME anchor', async () => {
    const { shadow, tree } = mountTree(true)
    await flush()
    // Anchor via engine so branch expand state stays open.
    tree.toggleRowSelected('src/lib', true)
    await flush()
    clickRow(shadow, 'readme', true) // span lib..readme
    await flush()
    expect(selectedIds(shadow)).toEqual(['src/lib', 'src/lib/a.ts', 'src/lib/b.ts', 'readme'])
    // Re-extend to license — anchor still lib, so the run grows to include license.
    clickRow(shadow, 'license', true)
    await flush()
    expect(selectedIds(shadow)).toContain('license')
    expect(selectedIds(shadow)).toContain('src/lib')
  })

  it('a node hidden under a collapsed branch is NOT in the displayed span', async () => {
    // src open, src/lib COLLAPSED → a.ts/b.ts are absent from tree.rows().
    const { shadow, tree } = mountTree({ src: true })
    await flush()
    // Anchor via engine — row click on src would collapse and hide the span.
    tree.toggleRowSelected('src', true)
    await flush()
    clickRow(shadow, 'readme', true) // span over the VISIBLE order only
    await flush()
    // Displayed order is src, index.ts, lib, readme — the collapsed a.ts/b.ts are
    // not in tree.rows(), so the range never touches them.
    expect(tree.selectedRowIds()).toEqual(['src', 'src/index.ts', 'src/lib', 'readme'])
    expect(tree.selectedRowIds()).not.toContain('src/lib/a.ts')
  })

  it('every node in the visible run carries [data-selected] (one highlighted block)', async () => {
    const { shadow } = mountTree(true)
    await flush()
    clickRow(shadow, 'src/index.ts')
    await flush()
    clickRow(shadow, 'src/lib/a.ts', true)
    await flush()
    for (const id of ['src/index.ts', 'src/lib', 'src/lib/a.ts']) {
      expect(rowFor(shadow, id).hasAttribute('data-selected')).toBe(true)
    }
    // Nodes outside the run stay unselected.
    expect(rowFor(shadow, 'src').hasAttribute('data-selected')).toBe(false)
    expect(rowFor(shadow, 'src/lib/b.ts').hasAttribute('data-selected')).toBe(false)
  })

  it('emits sail-node-select with the full selected id set on a range select', async () => {
    const { el, shadow } = mountTree(true)
    await flush()
    const fn = vi.fn()
    el.addEventListener('sail-node-select', fn)
    clickRow(shadow, 'src/index.ts')
    await flush()
    clickRow(shadow, 'src/lib/b.ts', true)
    await flush()
    expect(fn).toHaveBeenCalled()
    const detail = (fn.mock.calls.at(-1)![0] as CustomEvent).detail
    expect(detail.selectedIds).toEqual([
      'src/index.ts',
      'src/lib',
      'src/lib/a.ts',
      'src/lib/b.ts',
    ])
    expect(detail.selected).toBe(true)
  })

  it('sail-node-select bubbles + crosses the shadow boundary (composed)', async () => {
    const { shadow, container } = mountTree(true)
    await flush()
    let caught: CustomEvent | null = null
    container.addEventListener('sail-node-select', (e) => {
      caught = e as CustomEvent
    })
    clickRow(shadow, 'readme')
    await flush()
    clickRow(shadow, 'license', true)
    await flush()
    expect(caught).not.toBeNull()
    expect(caught!.detail.selectedIds).toEqual(['readme', 'license'])
  })

  it('shift-click WITHOUT a prior anchor selects only the target', async () => {
    const { shadow } = mountTree(true)
    await flush()
    // No plain click first → the range degenerates to the single target.
    clickRow(shadow, 'src/lib', true)
    await flush()
    expect(selectedIds(shadow)).toEqual(['src/lib'])
  })

  it('range select is gated by selectable=multi (single mode does not span)', async () => {
    const { shadow, tree } = mountTree(true, { selectable: 'single' })
    await flush()
    clickRow(shadow, 'src/index.ts')
    await flush()
    clickRow(shadow, 'src/lib/b.ts', true)
    await flush()
    // single mode: a second click replaces the selection — never a multi-row span.
    expect(tree.selectedRowIds().length).toBeLessThanOrEqual(1)
  })
})
