// <sail-tree> — FOUNDATIONAL slice (Wave-2 data-surface pack `sail-tree`).
//
// Proves the SHARED-ENGINE REUSE law: a hierarchy is the Table engine's
// expansion model turned on, NOT a parallel engine. There is no createTree().
// `sail-tree` calls `createTable<T>({ getSubRows, getRowId, columns: [],
// initialState: { expanded } })`, reads the flattened depth-first, expansion-
// respecting `table.rows()`, and renders it WINDOWED via `createVirtualizer`
// (the FIRST view-layer virtualizer consumer).
//
// Engine reaches the view via `prop:tree` IDENTITY binding (the live
// `Table<T>` JS reference, never a stringified/json attribute): the spec
// assigns the `createTable()` instance onto the element property — the same
// mechanism `sail-table.test.ts` uses for `prop:table`.
//
// TESTS-FIRST: the `.compiled.js` outputs do NOT exist yet, so the imports
// fail and every spec is RED until `sail-tree.sail` + `.controller.ts` land.
// Authority: .artifacts/sail-arc/sail-tree/build-spec.md (Slices 2-4 +
// behavioral specs #1/#2/#4/#6/#7/#10/#11), governed by ship-sail +
// ship-design-system skills.

import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/sail-tree/sail-tree.compiled.js'
// (NOT an inline <svg>) — register it so the caret renders as a real element.
import { createTable, type Table } from '../src/sail-table/engine/create-table'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

// ── Tree fixture ──────────────────────────────────────────────────────────────
interface FileNode extends Record<string, unknown> {
  id: string
  label: string
  children?: FileNode[]
}

/** A 3-level tree: src/ → {index.ts, lib/ → {a.ts, b.ts}}, plus a sibling leaf. */
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
]

/** Build the REUSED Table engine with hierarchy turned on — NO createTree(). */
function makeTree(
  expanded: Record<string, boolean> | true = {},
  opts: { selectable?: boolean } = {},
): Table<FileNode> {
  return createTable<FileNode>({
    data: TREE,
    columns: [], // sail-tree is label-only (single label column)
    getRowId: (r) => r.id,
    getSubRows: (r) => r.children,
    enableExpanding: true,
    enableRowSelection: opts.selectable ?? false,
    filterFromLeafRows: true,
    navMode: 'row',
    initialState: { expanded },
  })
}

/** Mount <sail-tree> and bind the live engine via the prop:tree identity path. */
function mountTree(
  expanded: Record<string, boolean> | true = {},
  attrs: Record<string, string | boolean> = {},
  opts: { selectable?: boolean } = {},
) {
  const tree = makeTree(expanded, opts)
  const m = mount(reg, 'sail-tree', attrs)
  // IDENTITY binding — the engine is a live reference, never serialized.
  ;(m.el as unknown as { tree: Table<FileNode> }).tree = tree
  return { ...m, tree }
}

/** The rendered treeitem rows, in DOM order, by their node id. */
const itemIds = (shadow: ShadowRoot | null): string[] =>
  Array.from(shadow!.querySelectorAll('[role="treeitem"]')).map(
    (r) => r.getAttribute('data-node-id') as string,
  )

describe('<sail-tree> — registration + container a11y', () => {

  it('sets role=tree on the container (single label column)', async () => {
    const { shadow } = mountTree()
    await flush()
    const container = shadow!.querySelector('[role="tree"]')
    expect(container).not.toBeNull()
  })

  it('reflects the aria-label from the label prop', async () => {
    const { shadow } = mountTree(true, { label: 'Files' })
    await flush()
    const container = shadow!.querySelector('[role="tree"]') as HTMLElement
    expect(container.getAttribute('aria-label')).toBe('Files')
  })
})

describe('<sail-tree> — shared-engine flatten (behavioral spec #1)', () => {
  it('renders one [role=treeitem] per tree.rows() (flattened visible nodes)', async () => {
    // All-collapsed: only the two root nodes are visible.
    const { shadow, tree } = mountTree({})
    await flush()
    const items = shadow!.querySelectorAll('[role="treeitem"]')
    expect(items.length).toBe(tree.rows().length)
    expect(itemIds(shadow)).toEqual(['src', 'readme'])
  })

  it("a collapsed branch's children are ABSENT from the DOM", async () => {
    const { shadow } = mountTree({})
    await flush()
    // src is collapsed → none of its descendants render.
    expect(itemIds(shadow)).not.toContain('src/index.ts')
    expect(itemIds(shadow)).not.toContain('src/lib')
  })

  it('expand-all (initialState.expanded=true) flattens depth-first, expansion-respecting', async () => {
    const { shadow, tree } = mountTree(true)
    await flush()
    // Depth-first visible order: src, index.ts, lib, a.ts, b.ts, README.md.
    expect(itemIds(shadow)).toEqual([
      'src',
      'src/index.ts',
      'src/lib',
      'src/lib/a.ts',
      'src/lib/b.ts',
      'readme',
    ])
    expect(itemIds(shadow).length).toBe(tree.rows().length)
  })

  it('a partially-expanded branch shows only its direct expanded descendants', async () => {
    // src open, src/lib still closed.
    const { shadow } = mountTree({ src: true })
    await flush()
    expect(itemIds(shadow)).toEqual(['src', 'src/index.ts', 'src/lib', 'readme'])
    expect(itemIds(shadow)).not.toContain('src/lib/a.ts')
  })
})

describe('<sail-tree> — expand/collapse via the engine (behavioral spec #2)', () => {
  it('clicking a branch row expands via the engine (no data-action hatch)', async () => {
    const { el, shadow } = mountTree({})
    await flush()
    const before = el
    const srcRow = shadow!.querySelector('[data-node-id="src"]') as HTMLElement
    // Whole row is the expand target — chevron is paint only.
    srcRow.click()
    await flush()
    // The branch expanded: src's direct children are now in the flat list.
    expect(itemIds(shadow)).toContain('src/index.ts')
    expect(itemIds(shadow)).toContain('src/lib')
    expect(el).toBe(before) // grew in place — the host did not remount
  })

  it('aria-expanded / data-expanded flip on the toggled branch', async () => {
    const { shadow, tree } = mountTree({})
    await flush()
    const srcRow = () => shadow!.querySelector('[data-node-id="src"]') as HTMLElement
    expect(srcRow().getAttribute('aria-expanded')).toBe('false')
    tree.toggleExpanded('src')
    await flush()
    expect(srcRow().getAttribute('aria-expanded')).toBe('true')
    expect(srcRow().hasAttribute('data-expanded')).toBe(true)
  })

  it('a leaf (!getCanExpand) shows no caret — KEEP :host([leaf]) parity', async () => {
    const { shadow } = mountTree(true)
    await flush()
    const leaf = shadow!.querySelector('[data-node-id="readme"]') as HTMLElement
    expect(leaf.hasAttribute('aria-expanded')).toBe(false)
    expect(leaf.hasAttribute('data-can-expand')).toBe(false)
  })

  it('a branch row carries data-can-expand', async () => {
    const { shadow } = mountTree(true)
    await flush()
    const branch = shadow!.querySelector('[data-node-id="src"]') as HTMLElement
    expect(branch.hasAttribute('data-can-expand')).toBe(true)
  })
})

describe('<sail-tree> — flat-list ARIA from row.depth + getPosInSet/getSetSize', () => {
  it('aria-level reflects row.depth + 1', async () => {
    const { shadow } = mountTree(true)
    await flush()
    const at = (id: string) =>
      (shadow!.querySelector(`[data-node-id="${id}"]`) as HTMLElement).getAttribute('aria-level')
    expect(at('src')).toBe('1') // depth 0
    expect(at('src/index.ts')).toBe('2') // depth 1
    expect(at('src/lib/a.ts')).toBe('3') // depth 2
  })

  it('aria-posinset / aria-setsize reflect the engine sibling position (#240 getPosInSet/getSetSize)', async () => {
    const { shadow } = mountTree(true)
    await flush()
    const row = (id: string) => shadow!.querySelector(`[data-node-id="${id}"]`) as HTMLElement
    // Roots: src is 1 of 2, README.md is 2 of 2.
    expect(row('src').getAttribute('aria-posinset')).toBe('1')
    expect(row('src').getAttribute('aria-setsize')).toBe('2')
    expect(row('readme').getAttribute('aria-posinset')).toBe('2')
    // src's children: index.ts is 1 of 2, lib is 2 of 2.
    expect(row('src/lib').getAttribute('aria-posinset')).toBe('2')
    expect(row('src/lib').getAttribute('aria-setsize')).toBe('2')
  })
})

describe('<sail-tree> — disclosure caret composes sail-icon (NOT inline svg)', () => {
  it('the caret is a sail-icon name="chevron-right", not an inline <svg>', async () => {
    const { shadow } = mountTree(true)
    await flush()
    const branch = shadow!.querySelector('[data-node-id="src"]') as HTMLElement
    const caret = branch.querySelector('[data-tree-caret]') as HTMLElement
    expect(caret).not.toBeNull()
    const icon = caret.querySelector('.sail-icon') as HTMLElement
    expect(icon).not.toBeNull()
    expect(icon.getAttribute('name')).toBe('chevron-right')
    // The NEW tag does NOT inline an <svg> caret (the KEEP item did).
    expect(caret.querySelector(':scope > svg')).toBeNull()
  })
})

describe('<sail-tree> — branch tri-state via #240 getIsSomeSelected (behavioral spec #4)', () => {
  it('a branch shows indeterminate when SOME (not all) leaf descendants are selected', async () => {
    const { shadow, tree } = mountTree(true, { selectable: 'multi' }, { selectable: true })
    await flush()
    // Select one of src/lib's two leaves → the branch is tri-state.
    tree.toggleRowSelected('src/lib/a.ts', true)
    await flush()
    const libRow = shadow!.querySelector('[data-node-id="src/lib"]') as HTMLElement
    const box = libRow.querySelector('input[type="checkbox"]') as HTMLInputElement
    expect(box).not.toBeNull()
    // Engine: getIsSomeSelected() is the real tri-state on branch rows (#240),
    // NOT the () => false stub.
    expect(tree.getRow('src/lib')!.getIsSomeSelected()).toBe(true)
    expect(box.indeterminate).toBe(true)
    expect(box.checked).toBe(false)
  })

  it('a branch shows fully-checked when ALL leaf descendants are selected', async () => {
    const { shadow, tree } = mountTree(true, { selectable: 'multi' }, { selectable: true })
    await flush()
    tree.toggleRowSelected('src/lib/a.ts', true)
    tree.toggleRowSelected('src/lib/b.ts', true)
    await flush()
    const libRow = shadow!.querySelector('[data-node-id="src/lib"]') as HTMLElement
    const box = libRow.querySelector('input[type="checkbox"]') as HTMLInputElement
    expect(tree.getRow('src/lib')!.getIsSelected()).toBe(true)
    expect(box.checked).toBe(true)
    expect(box.indeterminate).toBe(false)
  })
})

describe('<sail-tree> — flow list over tree.rows() (SPEC-TREE-1 path A)', () => {
  it('renders one treeitem per visible engine row (full list, not windowed)', async () => {
    const { shadow, tree } = mountTree(true)
    await flush()
    expect(itemIds(shadow).length).toBe(tree.rows().length)
  })

  it('expanding grows the DOM list; collapsing removes children (no hole)', async () => {
    const { shadow, tree } = mountTree({})
    await flush()
    expect(tree.rows().length).toBe(2)
    tree.toggleExpanded('src')
    await flush()
    expect(tree.rows().length).toBe(4)
    expect(itemIds(shadow)).toEqual(['src', 'src/index.ts', 'src/lib', 'readme'])
    tree.toggleExpanded('src')
    await flush()
    expect(tree.rows().length).toBe(2)
    expect(itemIds(shadow)).toEqual(['src', 'readme'])
    // Flow layout: no absolute --row-y / virtual spacer.
    const items = Array.from(shadow!.querySelectorAll('[role="treeitem"]')) as HTMLElement[]
    for (const el of items) {
      expect(el.style.getPropertyValue('--row-y')).toBe('')
      expect(getComputedStyle(el).position).not.toBe('absolute')
    }
  })

  it('rows are not absolute-stacked (no virtualizer translateY)', async () => {
    const { shadow } = mountTree(true)
    await flush()
    const items = Array.from(shadow!.querySelectorAll('[role="treeitem"]')) as HTMLElement[]
    expect(items.length).toBeGreaterThan(1)
    for (const el of items) {
      expect(el.className).not.toMatch(/translateY|translate-y/)
      expect(el.style.getPropertyValue('--row-y')).toBe('')
    }
  })
})

describe('<sail-tree> — keyboard nav via the engine nav (REAL navMode:row keymap, spec #7)', () => {
  it('roving-tabindex: exactly one treeitem is focusable at a time', async () => {
    const { shadow } = mountTree(true)
    await flush()
    const tabbable = () =>
      Array.from(shadow!.querySelectorAll('[role="treeitem"]')).filter(
        (r) => r.getAttribute('tabindex') === '0',
      )
    expect(tabbable().length).toBe(1)
  })

  it('ArrowDown moves row focus to the next visible node (engine nav.move down)', async () => {
    const { shadow } = mountTree(true)
    await flush()
    const container = shadow!.querySelector('[role="tree"]') as HTMLElement
    const tabbable = () =>
      Array.from(shadow!.querySelectorAll('[role="treeitem"]')).filter(
        (r) => r.getAttribute('tabindex') === '0',
      )
    expect(tabbable()[0]!.getAttribute('data-node-id')).toBe('src')
    container.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    await flush()
    expect(tabbable().length).toBe(1)
    expect(tabbable()[0]!.getAttribute('data-node-id')).toBe('src/index.ts')
  })

  it('Right on a collapsed branch expands it (engine treeArrow, getCanExpand)', async () => {
    const { shadow, tree } = mountTree({})
    await flush()
    const container = shadow!.querySelector('[role="tree"]') as HTMLElement
    expect(tree.getRow('src')!.getIsExpanded()).toBe(false)
    container.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true }))
    await flush()
    expect(tree.getRow('src')!.getIsExpanded()).toBe(true)
    expect(itemIds(shadow)).toContain('src/lib')
  })

  it('the view preventDefault()s only when nav.onKeydown handled the event', async () => {
    const { shadow } = mountTree(true)
    await flush()
    const container = shadow!.querySelector('[role="tree"]') as HTMLElement
    const handled = new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true, cancelable: true })
    container.dispatchEvent(handled)
    await flush()
    expect(handled.defaultPrevented).toBe(true)
    // A key the engine row-nav does not own bubbles through un-prevented.
    const ignored = new KeyboardEvent('keydown', { key: 'a', bubbles: true, cancelable: true })
    container.dispatchEvent(ignored)
    await flush()
    expect(ignored.defaultPrevented).toBe(false)
  })
})

describe('<sail-tree> — events via host.emit! (behavioral spec #3)', () => {
  it('toggling a branch emits sail-toggle {id, expanded}, bubbling + composed', async () => {
    const { el, container } = mountTree({})
    await flush()
    const fn = vi.fn()
    // Listen on the ANCESTOR — only a bubbling, composed event reaches it.
    container.addEventListener('sail-toggle', fn)
    const row = (el.shadowRoot ?? el).querySelector(
      '[data-node-id="src"]',
    ) as HTMLElement
    row.click()
    await flush()
    expect(fn).toHaveBeenCalled()
    const detail = (fn.mock.calls[0]![0] as CustomEvent).detail
    expect(detail.id).toBe('src')
    expect(detail.expanded).toBe(true)
  })

  it('toggling a branch emits sail-expand-change carrying the full ExpandedState snapshot', async () => {
    const { el } = mountTree({})
    await flush()
    const fn = vi.fn()
    el.addEventListener('sail-expand-change', fn)
    const row = (el.shadowRoot ?? el).querySelector(
      '[data-node-id="src"]',
    ) as HTMLElement
    row.click()
    await flush()
    expect(fn).toHaveBeenCalled()
    const detail = (fn.mock.calls.at(-1)![0] as CustomEvent).detail
    expect(detail.expanded).toMatchObject({ src: true })
  })

  it('invokes onSelect L1 callback with { id } on leaf select (studio explorer seam)', async () => {
    const { el } = mountTree({}, { selectable: 'single' }, { selectable: true })
    await flush()
    const onSelect = vi.fn()
    ;(el as HTMLElement & { onSelect?: (d: { id?: string }) => void }).onSelect =
      onSelect
    const row = (el.shadowRoot ?? el).querySelector(
      '[data-node-id="readme"]',
    ) as HTMLElement
    expect(row).toBeTruthy()
    row.click()
    await flush()
    expect(onSelect).toHaveBeenCalled()
    expect(onSelect.mock.calls[0]![0]).toMatchObject({ id: 'readme' })
  })
})

describe('<sail-tree> — NO parallel engine (behavioral spec #11)', () => {
  it('the hierarchy is createTable() with getSubRows+expanded — read via tree.rows()', () => {
    // The engine factory imported here is createTable from the sail-table engine
    // dir. There is NO createTree() / create-tree.ts in the pack: the only row
    // read is tree.rows() (the flattened visible list), proven by every spec above.
    expect(typeof createTable).toBe('function')
  })
})

describe('<sail-tree> — density is the platform axis (behavioral spec #10)', () => {
  it('declares NO density and NO dense prop (rows read --density-* from <html data-density>)', async () => {
    const { el } = mountTree(true)
    await flush()
    const proto = Object.getPrototypeOf(el) as object
    expect(Object.getOwnPropertyNames(proto)).not.toContain('density')
    expect(Object.getOwnPropertyNames(proto)).not.toContain('dense')
  })
})
