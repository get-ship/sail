// <sail-tree> — LAZY-CHILDREN / SERVER-EXPANSION seam (Wave-2 data-surface pack).
//
// Lazy children are a CONSUMER concern threaded through the engine's reactive
// `data` + `getSubRows` seam — NOT engine code, and emphatically NOT `ship.*`
// inside the pack. This slice proves the round-trip a real consumer performs
// WITHOUT putting fetch/SDK in `sail-tree`/`sail-tree-grid`:
//
//   1. a node declares it has children-to-load (a `getSubRows` that returns a
//      single synthetic "pending" placeholder child until the real ones load);
//   2. toggling that branch fires `sail-expand-change {expanded}` from the view;
//   3. the (test-mocked) consumer loader resolves the children and sets them
//      onto the reactive `data` signal the engine was built over;
//   4. the engine RE-FLATTENS reactively and the real children appear — no
//      remount of the host, no second engine;
//   5. while pending, a "loading children" placeholder row renders (consumer-
//      injected; the engine has no loading state).
//
// The KEY invariant under test: the PACK itself contains zero fetch/EventSource/
// `ship.*`/`@ship/sdk-client`/`<data>` — the entire async round-trip lives in
// the (mocked) consumer. `manualExpanding:true` (pre-flattened server rows) is
// NOT exercised — that option is absent from the engine; if wanted it is a
// flagged Table-engine feature request.
//
// TESTS-FIRST: the "loading children" placeholder render + the expand-change
// driven re-flatten through the view are not wired today, so these specs are RED
// until the seam lands. NO impl written. Authority:
// .artifacts/sail-arc/sail-tree/build-spec.md (Slice 6 + behavioral specs #9/#13;
// §"Lazy / server children (the seam)").

import { afterEach, describe, expect, it, vi } from 'vitest'
import { readFileSync, readdirSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import '../src/sail-tree/sail-tree.compiled.js'
import { signal, type Signal } from '@ship/sail-runtime'
import { createTable, type Table } from '../src/sail-table/engine/create-table'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

interface LazyNode extends Record<string, unknown> {
  id: string
  label: string
  /** True for a branch whose children are loaded on first expand. */
  lazy?: boolean
  /** Synthetic flag the view reads to render a "loading…" placeholder row. */
  pending?: boolean
  children?: LazyNode[]
}

const PENDING_CHILD = (parentId: string): LazyNode => ({
  id: `${parentId}/__pending__`,
  label: 'Loading…',
  pending: true,
})

/** Root tree: one lazy branch whose children are NOT present until loaded. */
const ROOTS: LazyNode[] = [
  { id: 'remote', label: 'remote', lazy: true },
  { id: 'readme', label: 'README.md' },
]

/** The children a consumer "fetches" for the lazy branch. */
const REMOTE_CHILDREN: LazyNode[] = [
  { id: 'remote/one.ts', label: 'one.ts' },
  { id: 'remote/two.ts', label: 'two.ts' },
]

/**
 * Build the REUSED engine over a reactive `data` signal so a consumer can swap
 * the loaded children in and the engine re-flattens. `getSubRows` returns a
 * single synthetic PENDING child for a lazy branch with no real children yet —
 * this is the consumer's choice, NOT engine behavior. NO createTree().
 */
function makeLazyTree(
  data: Signal<LazyNode[]>,
  loadingIds: Signal<Set<string>>,
): Table<LazyNode> {
  return createTable<LazyNode>({
    data,
    columns: [],
    getRowId: (r) => r.id,
    getSubRows: (r) => {
      if (r.children && r.children.length > 0) return r.children
      // A lazy branch surfaces a pending placeholder while its load is inflight,
      // so the row is expandable and a "Loading…" row appears under it.
      if (r.lazy && loadingIds().has(r.id)) return [PENDING_CHILD(r.id)]
      if (r.lazy) return [] // expandable, children resolved lazily on first toggle
      return r.children
    },
    enableExpanding: true,
    filterFromLeafRows: true,
    navMode: 'row',
    initialState: { expanded: {} },
  })
}

function mountLazy() {
  const data = signal<LazyNode[]>(ROOTS)
  const loadingIds = signal<Set<string>>(new Set())
  const tree = makeLazyTree(data, loadingIds)
  const m = mount(reg, 'sail-tree', { selectable: 'single' })
  ;(m.el as unknown as { tree: Table<LazyNode> }).tree = tree
  return { ...m, tree, data, loadingIds }
}

const itemIds = (shadow: ShadowRoot | null): string[] =>
  Array.from(shadow!.querySelectorAll('[role="treeitem"]')).map(
    (r) => r.getAttribute('data-node-id') as string,
  )

const caretFor = (shadow: ShadowRoot | null, id: string): HTMLElement =>
  (shadow!.querySelector(`[data-node-id="${id}"]`) as HTMLElement).querySelector(
    '[data-action="toggle"]',
  ) as HTMLElement

describe('<sail-tree> — lazy-children consumer seam (behavioral spec #9)', () => {
  it('a lazy branch renders with a caret even before its children are loaded', async () => {
    const { shadow } = mountLazy()
    await flush()
    const remote = shadow!.querySelector('[data-node-id="remote"]') as HTMLElement
    // The lazy branch is expandable (getCanExpand true) so the user can open it.
    expect(remote.hasAttribute('data-can-expand')).toBe(true)
  })

  it('expanding a lazy branch fires sail-expand-change carrying the expansion snapshot', async () => {
    const { el, shadow } = mountLazy()
    await flush()
    const fn = vi.fn()
    el.addEventListener('sail-expand-change', fn)
    caretFor(shadow, 'remote').click()
    await flush()
    expect(fn).toHaveBeenCalled()
    const detail = (fn.mock.calls.at(-1)![0] as CustomEvent).detail
    expect(detail.expanded).toMatchObject({ remote: true })
  })

  it('a "Loading…" placeholder row renders while the consumer load is inflight', async () => {
    const { shadow, tree, loadingIds } = mountLazy()
    await flush()
    // The consumer (mocked) flips the loading flag on expand → the synthetic
    // pending child surfaces as a placeholder row under the branch.
    tree.toggleExpanded('remote')
    loadingIds.set(new Set(['remote']))
    await flush()
    expect(itemIds(shadow)).toContain('remote/__pending__')
    const placeholder = shadow!.querySelector(
      '[data-node-id="remote/__pending__"]',
    ) as HTMLElement
    // The placeholder row is flagged so the view styles it as loading (a
    // skeleton / spinner / [data-pending]) rather than a real selectable node.
    expect(placeholder.hasAttribute('data-pending')).toBe(true)
  })

  it('the engine re-flattens when the consumer sets the loaded children (no remount)', async () => {
    const { el, shadow, tree, data, loadingIds } = mountLazy()
    await flush()
    const before = el
    tree.toggleExpanded('remote')
    loadingIds.set(new Set(['remote']))
    await flush()
    expect(itemIds(shadow)).toContain('remote/__pending__')
    // Consumer "fetch" resolves: write the real children onto the reactive data
    // signal + clear the loading flag → the engine re-flattens reactively.
    data.set([{ id: 'remote', label: 'remote', lazy: true, children: REMOTE_CHILDREN }, { id: 'readme', label: 'README.md' }])
    loadingIds.set(new Set())
    await flush()
    expect(itemIds(shadow)).toContain('remote/one.ts')
    expect(itemIds(shadow)).toContain('remote/two.ts')
    // The pending placeholder is gone, and the host never remounted.
    expect(itemIds(shadow)).not.toContain('remote/__pending__')
    expect(el).toBe(before)
  })

  it('a consumer can drive the load off the sail-expand-change event end-to-end', async () => {
    const { el, shadow, tree, data, loadingIds } = mountLazy()
    await flush()
    // Wire a (mocked) consumer loader exactly as a real consumer would: listen
    // for sail-expand-change, resolve children, set the data signal. NO ship.*.
    const loader = vi.fn(async (id: string) => {
      loadingIds.set(new Set([id]))
      await Promise.resolve()
      data.set([
        { id: 'remote', label: 'remote', lazy: true, children: REMOTE_CHILDREN },
        { id: 'readme', label: 'README.md' },
      ])
      loadingIds.set(new Set())
    })
    el.addEventListener('sail-expand-change', (e) => {
      const expanded = (e as CustomEvent).detail.expanded as Record<string, boolean>
      for (const id of Object.keys(expanded)) if (expanded[id]) void loader(id)
    })
    caretFor(shadow, 'remote').click()
    await flush()
    await flush()
    expect(loader).toHaveBeenCalledWith('remote')
    expect(itemIds(shadow)).toContain('remote/one.ts')
    expect(tree.getRow('remote')!.getIsExpanded()).toBe(true)
  })
})

// ── Boundary purity (behavioral spec #13 — TEST-asserted, NOT sail-lint) ───────
describe('sail-tree pack — boundary purity (no fetch / SDK / ship.* / <data>)', () => {
  const here = dirname(fileURLToPath(import.meta.url))
  const packDirs = [
    join(here, '../src/sail-tree'),
    join(here, '../src/sail-tree-grid'),
  ]

  /** Every source file in the pack dirs (excludes compiled .js output). */
  function packSources(): { path: string; text: string }[] {
    const out: { path: string; text: string }[] = []
    for (const dir of packDirs) {
      for (const entry of readdirSync(dir)) {
        if (entry.endsWith('.compiled.js')) continue
        if (!/\.(ts|sui|css)$/.test(entry)) continue
        const path = join(dir, entry)
        out.push({ path, text: readFileSync(path, 'utf8') })
      }
    }
    return out
  }

  const BANNED: { name: string; re: RegExp }[] = [
    { name: 'fetch(', re: /\bfetch\s*\(/ },
    { name: 'EventSource', re: /\bEventSource\b/ },
    { name: 'ship.* call', re: /\bship\.\w+\.\w+\s*\(/ },
    { name: '@ship/sdk-client import', re: /@ship\/sdk-client/ },
    { name: '<data> directive', re: /<data\b/ },
  ]

  it('scans a non-empty set of pack source files', () => {
    expect(packSources().length).toBeGreaterThan(0)
  })

  for (const banned of BANNED) {
    it(`contains no ${banned.name} anywhere in the pack`, () => {
      const offenders = packSources()
        .filter((f) => banned.re.test(f.text))
        .map((f) => f.path)
      expect(offenders).toEqual([])
    })
  }
})
