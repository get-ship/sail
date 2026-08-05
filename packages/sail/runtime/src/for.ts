// Keyed list reconciliation. Renders an array signal as a sequence
// of DOM nodes, preserving identity (focus, scroll, animation state)
// across reorders, inserts, removes, and updates.
//
// The sail compiler turns `<for {list} as="item" key={item.id}>…</for>` into
// a call to bindFor at hydrate time. Authors don't import this directly.
//

import { effect, type Signal } from './signal'
import { createScope, withScopeArgs, type EffectScope } from './effect-scope'
import type { Disposer } from './bind'
import { findNearestBoundary } from './error-boundary'
import { runEnter, runLeave } from './transition-runner'

/**
 * Inputs to `bindFor` — what to iterate, how to identify items, and
 * how to render them.
 */
export interface BindForOptions<T> {
  /** Anchor placeholder — siblings are inserted AFTER this node. */
  anchor: Comment
  /** Reactive source of the array. */
  items: () => readonly T[]
  /** Per-item key — must be stable + unique per item identity. */
  key: (item: T, index: number) => unknown
  /** Render one item; returns its top-level nodes + an optional cleanup. */
  render: (item: T, index: number) => RenderedItem
}

/**
 * The shape returned by a `BindForOptions.render` call — DOM nodes for
 * one row plus optional cleanup and update hooks.
 */
export interface RenderedItem {
  nodes: Node[]
  cleanup?: () => void
  /**
   * Optional in-place updater. Called on reuse — when the same key
   * appears in the next list with a different item value or position.
   * Lets the row push the new item/index into its internal signals
   * without tearing down DOM. Without this, reused rows keep their
   * original closure and won't reflect updates.
   */
  update?: (item: unknown, index: number) => void
}

interface ItemRecord<T> {
  key: unknown
  item: T
  /** Last index the row was rendered at; paired with `item` so the
   *  identity-skip in step 2 only fires when both are unchanged. */
  __i: number
  nodes: Node[]
  cleanup?: (() => void) | undefined
  update?: ((item: unknown, index: number) => void) | undefined
  /** Disposes the effects (bindings) created while rendering THIS row. The
   *  row is rendered inside its own scope so its effects tear down when the
   *  row is removed — not when the reconciler's list effect re-runs. */
  scope?: EffectScope | undefined
}

// `matched` packs each matched record's (newIndex, prevIndex) pair into
// adjacent Int32 slots: matched[2*i]=new-array index, matched[2*i+1]=prev-array
// index. mask/pred/tails are caller-owned scratch (length >= count). On
// return, mask[i]=1 iff matched record i is in the LIS of its prev positions.
// O(n log n) patience sort.
function computeLisMaskInto(
  matched: Int32Array, count: number,
  mask: Uint8Array, pred: Int32Array, tails: Int32Array,
): void {
  mask.fill(0, 0, count)
  if (count === 0) return
  let tailsLen = 0
  for (let i = 0; i < count; i++) {
    const v = matched[2 * i + 1]!
    let lo = 0, hi = tailsLen
    while (lo < hi) {
      const mid = (lo + hi) >>> 1
      if (matched[2 * tails[mid]! + 1]! < v) lo = mid + 1
      else hi = mid
    }
    pred[i] = lo > 0 ? tails[lo - 1]! : -1
    tails[lo] = i
    if (lo === tailsLen) tailsLen++
  }
  let idx = tails[tailsLen - 1]!
  while (idx >= 0) { mask[idx] = 1; idx = pred[idx]! }
}

/**
 * Bind a keyed list to the DOM after `opts.anchor`. The renderer is
 * called once per new key; rows whose keys persist across renders keep
 * their original nodes (and any focus / scroll / animation state).
 *
 * Reconciliation uses a longest-increasing-subsequence algorithm to
 * minimise DOM moves. Matched keys already in a stable relative order
 * are never moved; only out-of-order and new keys are inserted.
 *
 * Compiler target: `<for {list} as="item" key={item.id}>…</for>`.
 *
 * @param opts Anchor, reactive items source, key fn, and render fn.
 * @returns Disposer that detaches every row and stops updates.
 *
 * @example
 * ```ts
 * const items = signal<Row[]>([])
 * bindFor({
 *   anchor,
 *   items: () => items(),
 *   key: (row) => row.id,
 *   render: (row) => {
 *     const li = document.createElement('li')
 *     li.textContent = row.label
 *     return { nodes: [li] }
 *   },
 * })
 * ```
 *
 * @see bindForSignal for the common case where `items` is a signal.
 */
export function bindFor<T>(opts: BindForOptions<T>): Disposer {
  const { anchor, items, key, render } = opts
  let prev: ItemRecord<T>[] = []
  let active = true
  // Rows whose key disappeared from the items array and whose
  // out-transition is in flight. Tracked so the disposer can flush
  // them synchronously without waiting for animations.
  const departing = new Set<ItemRecord<T>>()

  // Scratch — allocated once at bindFor setup, .clear()'d each render so
  // reconciliation reuses a constant working set instead of allocating
  // O(N) transient collections per effect run.
  const nextKeys = new Map<unknown, number>() // key → first-seen index
  const prevByKey = new Map<unknown, ItemRecord<T>>()
  const prevPos = new Map<unknown, number>()
  const stable = new Set<number>()
  const freshNodes: Node[] = []
  let nextRecords: ItemRecord<T>[] = []
  let matched = new Int32Array(0)        // [newIdx, prevIdx] pairs
  let lisMaskBuf = new Uint8Array(0), lisPred = new Int32Array(0), lisTails = new Int32Array(0)

  const flush = (r: ItemRecord<T>) => {
    for (const n of r.nodes) {
      try {
        n.parentNode?.removeChild(n)
      } catch {
        // Already detached.
      }
    }
    r.cleanup?.()
    r.scope?.dispose()
  }

  // Last-seen items array, retained so we can short-circuit when the
  // reactive source re-fires with the same reference (e.g. a derived
  // upstream re-evaluates and returns its cached array). Cleared by
  // disposal alongside `prev`.
  let lastItems: readonly T[] | null = null

  const dispose = effect(() => {
    if (!active) return
    const next = items()
    // Same-array short-circuit: if the reactive source produced the
    // exact same array reference, no key set or position changed, so
    // the reconciler has nothing to do. Per-row state still flows
    // through each row's own signals — those re-render independently.
    if (next === lastItems) return
    lastItems = next
    const n = next.length

    // Grow scratch typed arrays to the new high-water mark; never shrink.
    if (matched.length < n * 2) matched = new Int32Array(Math.max(matched.length * 2, n * 2, 16))
    if (lisMaskBuf.length < n) {
      const cap = Math.max(lisMaskBuf.length * 2, n, 8)
      lisMaskBuf = new Uint8Array(cap); lisPred = new Int32Array(cap); lisTails = new Int32Array(cap)
    }

    for (let i = 0; i < n; i++) {
      const item = next[i]!
      const k = key(item, i)
      if (nextKeys.has(k)) {
        // Duplicate key: silent overwrite would drop the earlier item
        // from the rendered output. Warn once per render — the user's
        // key function is broken (or the list contains duplicate
        // identity). Reconciliation continues using last-wins to keep
        // the DOM in a consistent state.
        console.warn(
          '[sail] {#for}: duplicate key',
          k,
          'at index',
          i,
          '(prior at',
          nextKeys.get(k),
          '); the earlier item will be dropped from the rendered output.',
          'Fix the key function so each item maps to a unique value.',
        )
      }
      nextKeys.set(k, i)
    }

    for (const r of prev) prevByKey.set(r.key, r)

    // 1. Remove items that are gone in the next array. Rows with an
    //    out-transition stay in DOM until the animation resolves; the
    //    LIS reposition below skips them naturally since they're
    //    absent from `nextRecords`. No transition → synchronous flush
    //    (preserves the historical non-animated behaviour).
    for (const r of prev) {
      if (nextKeys.has(r.key)) continue
      const leaving = runLeave(r.nodes)
      if (leaving === null) {
        flush(r)
        continue
      }
      departing.add(r)
      leaving.then(() => {
        if (!departing.delete(r)) return
        flush(r)
      })
    }

    // 2. Build new records list — reuse matched keys, render new ones.
    //    No DOM mutations in this pass.
    nextRecords.length = 0
    freshNodes.length = 0
    let anyReused = false
    for (let i = 0; i < n; i++) {
      const item = next[i]!
      const k = key(item, i)
      const existing = prevByKey.get(k)
      let record: ItemRecord<T>
      if (existing) {
        anyReused = true
        // Skip the per-row update push when the item reference and its
        // position are both unchanged. The row's internal signals would
        // short-circuit on Object.is anyway, but the function-call /
        // closure-allocation overhead per row dominates at N=1000.
        if (existing.item !== item || existing.__i !== i) {
          existing.item = item
          existing.__i = i
          try {
            existing.update?.(item, i)
          } catch (err) {
            const handle = findNearestBoundary(anchor)
            if (handle) {
              handle.catch(err, 'render')
            } else {
              console.error('[sail] uncaught throw (no error boundary):', err)
            }
          }
        }
        record = existing
      } else {
        // Reaching the row render used to cost FIVE allocations before the body ran:
        //
        //   renderInScope(() => runProtected(anchor, 'render', () => render(item, i)))
        //
        // two call-site arrows, the scope, the tuple `renderInScope` returns, and the
        // disposer closure inside it — then two more from the conditional spreads
        // below. On a 1,000-row list that is 7,000 objects to call one function 1,000
        // times. Only the scope is real; the rest was ceremony. Threading the args
        // through `withScopeArgs` and inlining the boundary catch removes the rest.
        const scope = createScope()
        let r: RenderedItem | undefined
        try {
          r = withScopeArgs(scope, render, item, i)
        } catch (err) {
          const handle = findNearestBoundary(anchor)
          if (handle) {
            handle.catch(err, 'render')
          } else {
            console.error('[sail] uncaught throw (no error boundary):', err)
          }
        }
        if (!r) {
          // Render threw and was caught by a boundary (or logged when no boundary is
          // in scope). Dispose the (empty) row scope and skip this item: the boundary
          // will unmount the subtree shortly.
          scope.dispose()
          continue
        }
        record = {
          key: k,
          item,
          __i: i,
          nodes: r.nodes,
          scope,
          cleanup: r.cleanup,
          update: r.update,
        }
        const rn = r.nodes
        for (let j = 0; j < rn.length; j++) freshNodes.push(rn[j]!)
      }
      nextRecords.push(record)
    }

    // 3. Find matched records already in a stable relative order (LIS of
    //    their previous-array positions). These records don't need DOM
    //    moves — only non-LIS and new records are repositioned. Skipped
    //    entirely when step 2 reused no rows (full-replace / first
    //    render): with no matches there's no LIS to compute and the
    //    placement loop below takes the fragment fast path.
    //
    //    Monotone fast-path: when matched-prev positions are strictly
    //    increasing, every matched record is its own LIS — no DOM
    //    moves needed. Covers the common shapes (append, prepend at
    //    far end, single insert, single remove, no-op) without paying
    //    the O(n log n) LIS scan. Krausest's `remove-one-1k` saves
    //    ~9000 ops on a 1000-row list; `swap-1k` still falls into
    //    the full LIS path because the swap creates an inversion.
    let matchedCount = 0
    if (anyReused) {
      for (let i = 0; i < prev.length; i++) prevPos.set(prev[i]!.key, i)

      let monotone = true
      let lastPrevPos = -1
      for (let i = 0; i < nextRecords.length; i++) {
        const p = prevPos.get(nextRecords[i]!.key)
        if (p !== undefined) {
          matched[2 * matchedCount] = i
          matched[2 * matchedCount + 1] = p
          matchedCount++
          if (monotone && p <= lastPrevPos) monotone = false
          lastPrevPos = p
        }
      }

      if (monotone) {
        for (let j = 0; j < matchedCount; j++) stable.add(matched[2 * j]!)
      } else {
        computeLisMaskInto(matched, matchedCount, lisMaskBuf, lisPred, lisTails)
        for (let j = 0; j < matchedCount; j++) {
          if (lisMaskBuf[j]) stable.add(matched[2 * j]!)
        }
      }
    }

    // 4. Place all records. Fast path: when no records were matched
    //    against the previous render, every record's nodes are freshly
    //    created and unparented — batch them into a DocumentFragment so
    //    the parent sees one mutation instead of N. Covers both the
    //    initial mount (prev empty) and full-replace (prev fully
    //    discarded, all new keys). Custom elements still upgrade and
    //    receive connectedCallback when the fragment lands in the live
    //    tree.
    const parent = anchor.parentNode
    if (parent && matchedCount === 0 && nextRecords.length > 0) {
      const frag = document.createDocumentFragment()
      for (let i = 0; i < nextRecords.length; i++) {
        const nodes = nextRecords[i]!.nodes
        for (let j = 0; j < nodes.length; j++) frag.appendChild(nodes[j]!)
      }
      parent.insertBefore(frag, anchor.nextSibling)
    } else if (parent) {
      // Place records, batching each maximal run of consecutive
      // non-stable records into one DocumentFragment — a bulk insert
      // (append 1000 onto 1000, paste a range) then costs one DOM
      // mutation per run instead of one per row. `cursor` only ever
      // advances to a stable (unmoved, still-parented) node or the
      // anchor, so `cursor.nextSibling` — read AFTER the run's nodes are
      // pulled into the fragment — is the correct insertion point even
      // when a moved node had been the next sibling.
      let cursor: Node = anchor
      let i = 0
      while (i < nextRecords.length) {
        if (stable.has(i)) {
          const nodes = nextRecords[i]!.nodes
          cursor = nodes[nodes.length - 1] ?? cursor
          i++
          continue
        }
        const frag = document.createDocumentFragment()
        let lastRunNode: Node | null = null
        while (i < nextRecords.length && !stable.has(i)) {
          for (const node of nextRecords[i]!.nodes) {
            frag.appendChild(node)
            lastRunNode = node
          }
          i++
        }
        parent.insertBefore(frag, cursor.nextSibling)
        if (lastRunNode) cursor = lastRunNode
      }
    }

    // Swap buffers: `nextRecords` becomes the new prev; the old prev
    // (now stale) becomes the empty buffer for the next render.
    const tmp = prev
    prev = nextRecords
    nextRecords = tmp

    if (freshNodes.length > 0) runEnter(freshNodes)

    // Drop scratch contents so we don't pin references (records, items,
    // keys) between renders. The Maps/Sets themselves are reused.
    // Pre-fix, `nextRecords` was left holding the previous render's
    // record objects (item refs, detached DOM nodes, cleanup/update
    // closures) until the next render's line `nextRecords.length = 0`
    // dropped them. After a `clear()` that emptied the list, no next
    // render arrived — and Krausest's 25_run-clear-memory measured
    // 2.7MB of residual where Solid/Svelte sat at 0.7–0.9MB. The
    // records were already flushed (DOM removed, cleanup called), but
    // the JS objects pinning `item` payloads + the detached <tr>
    // subtrees stayed referenced through the scratch array.
    nextRecords.length = 0
    nextKeys.clear()
    prevByKey.clear()
    prevPos.clear()
    stable.clear()
    freshNodes.length = 0
  })

  return () => {
    active = false
    dispose()
    for (const r of prev) flush(r)
    // Departing rows: hard teardown skips out-animations.
    for (const d of departing) flush(d)
    departing.clear()
    prev = []
    nextRecords.length = 0
    lastItems = null
  }
}

/**
 * Convenience wrapper around `bindFor` for the common case where the
 * items source is a signal. Equivalent to
 * `bindFor({ anchor, items: () => sig(), key, render })`.
 *
 * @param anchor Comment placeholder; rows are inserted after it.
 * @param sig Signal whose value is the current array.
 * @param key Stable, unique key per item identity.
 * @param render Per-row renderer.
 * @returns Disposer.
 *
 * @example
 * ```ts
 * const rows = signal<Row[]>([])
 * bindForSignal(anchor, rows, (r) => r.id, renderRow)
 * ```
 *
 * @see bindFor for the full options form.
 */
export function bindForSignal<T>(
  anchor: Comment,
  sig: Signal<readonly T[]>,
  key: (item: T, index: number) => unknown,
  render: (item: T, index: number) => RenderedItem,
): Disposer {
  return bindFor({ anchor, items: () => sig(), key, render })
}
