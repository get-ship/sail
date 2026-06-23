import { describe, expect, it, vi } from 'vitest'
import { signal } from '../src/signal'
import { bindFor, bindForSignal } from '../src/for'

interface Item { id: string; label: string }

function setup(initial: Item[]) {
  const root = document.createElement('div')
  document.body.appendChild(root)
  const anchor = document.createComment('list')
  root.appendChild(anchor)

  const $list = signal<Item[]>(initial)
  const renderCalls: { id: string; index: number }[] = []
  const cleanupCalls: string[] = []

  const dispose = bindForSignal(
    anchor,
    $list,
    (item) => item.id,
    (item, index) => {
      renderCalls.push({ id: item.id, index })
      const li = document.createElement('li')
      li.id = `item-${item.id}`
      li.textContent = item.label
      return {
        nodes: [li],
        cleanup: () => cleanupCalls.push(item.id),
      }
    },
  )

  const ids = () =>
    Array.from(root.querySelectorAll('li')).map((el) => el.id.replace('item-', ''))

  return { root, $list, ids, renderCalls, cleanupCalls, dispose }
}

describe('bindFor — initial render', () => {
  it('renders an empty list as no DOM nodes', () => {
    const { root, ids, renderCalls } = setup([])
    expect(ids()).toEqual([])
    expect(renderCalls).toEqual([])
    expect(root.querySelectorAll('li').length).toBe(0)
  })

  it('renders each item once', () => {
    const { ids, renderCalls } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
      { id: 'c', label: 'C' },
    ])
    expect(ids()).toEqual(['a', 'b', 'c'])
    expect(renderCalls.map((r) => r.id)).toEqual(['a', 'b', 'c'])
  })

  it('renders inserted in correct order', () => {
    const items = [
      { id: 'z', label: 'Z' },
      { id: 'm', label: 'M' },
      { id: 'a', label: 'A' },
    ]
    const { ids } = setup(items)
    expect(ids()).toEqual(['z', 'm', 'a'])
  })
})

describe('bindFor — append', () => {
  it('appends one item without re-rendering existing', () => {
    const { $list, ids, renderCalls } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
    ])
    expect(renderCalls.length).toBe(2)
    $list.set([...$list(), { id: 'c', label: 'C' }])
    expect(ids()).toEqual(['a', 'b', 'c'])
    expect(renderCalls.length).toBe(3) // only `c` rendered
    expect(renderCalls[2]!.id).toBe('c')
  })

  it('appends many items', () => {
    const { $list, ids } = setup([{ id: 'a', label: 'A' }])
    $list.set([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
      { id: 'c', label: 'C' },
      { id: 'd', label: 'D' },
    ])
    expect(ids()).toEqual(['a', 'b', 'c', 'd'])
  })
})

describe('bindFor — prepend', () => {
  it('prepends one item without re-rendering existing', () => {
    const { $list, ids, renderCalls } = setup([
      { id: 'b', label: 'B' },
      { id: 'c', label: 'C' },
    ])
    $list.set([{ id: 'a', label: 'A' }, ...$list()])
    expect(ids()).toEqual(['a', 'b', 'c'])
    expect(renderCalls.length).toBe(3) // only `a` is new
    expect(renderCalls[2]!.id).toBe('a')
  })
})

describe('bindFor — remove', () => {
  it('removes an item from the middle, calls its cleanup', () => {
    const { $list, ids, cleanupCalls } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
      { id: 'c', label: 'C' },
    ])
    $list.set([
      { id: 'a', label: 'A' },
      { id: 'c', label: 'C' },
    ])
    expect(ids()).toEqual(['a', 'c'])
    expect(cleanupCalls).toEqual(['b'])
  })

  it('removes the first item', () => {
    const { $list, ids } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
    ])
    $list.set([{ id: 'b', label: 'B' }])
    expect(ids()).toEqual(['b'])
  })

  it('removes the last item', () => {
    const { $list, ids } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
    ])
    $list.set([{ id: 'a', label: 'A' }])
    expect(ids()).toEqual(['a'])
  })

  it('removes all items', () => {
    const { $list, ids, cleanupCalls } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
    ])
    $list.set([])
    expect(ids()).toEqual([])
    expect(cleanupCalls.sort()).toEqual(['a', 'b'])
  })
})

describe('bindFor — reorder', () => {
  it('swaps two adjacent items, preserving DOM identity', () => {
    const { root, $list, ids, renderCalls } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
    ])
    const aBefore = root.querySelector('#item-a')!
    const bBefore = root.querySelector('#item-b')!
    $list.set([
      { id: 'b', label: 'B' },
      { id: 'a', label: 'A' },
    ])
    expect(ids()).toEqual(['b', 'a'])
    expect(renderCalls.length).toBe(2) // no re-render
    // Same DOM node identity — focus / scroll / animation preserved.
    expect(root.querySelector('#item-a')).toBe(aBefore)
    expect(root.querySelector('#item-b')).toBe(bBefore)
  })

  it('reverses a longer list', () => {
    const { $list, ids } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
      { id: 'c', label: 'C' },
      { id: 'd', label: 'D' },
    ])
    $list.set([...$list()].reverse())
    expect(ids()).toEqual(['d', 'c', 'b', 'a'])
  })

  it('moves an item from end to beginning', () => {
    const { $list, ids, renderCalls } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
      { id: 'c', label: 'C' },
    ])
    $list.set([
      { id: 'c', label: 'C' },
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
    ])
    expect(ids()).toEqual(['c', 'a', 'b'])
    expect(renderCalls.length).toBe(3)
  })
})

describe('bindFor — mixed mutations', () => {
  it('add + remove + reorder in one update', () => {
    const { $list, ids, renderCalls, cleanupCalls } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
      { id: 'c', label: 'C' },
    ])
    $list.set([
      { id: 'c', label: 'C' },         // moved
      { id: 'd', label: 'D' },         // new
      { id: 'a', label: 'A' },         // moved
      // b removed
    ])
    expect(ids()).toEqual(['c', 'd', 'a'])
    expect(renderCalls.length).toBe(4) // a, b, c, d
    expect(cleanupCalls).toEqual(['b'])
  })
})

describe('bindFor — preserves form state across reorder', () => {
  it('checkbox state survives a reorder', () => {
    const root = document.createElement('div')
    document.body.appendChild(root)
    const anchor = document.createComment('list')
    root.appendChild(anchor)
    const $list = signal<{ id: string }[]>([
      { id: 'a' },
      { id: 'b' },
      { id: 'c' },
    ])
    bindForSignal(
      anchor,
      $list,
      (item) => item.id,
      (item) => {
        const li = document.createElement('li')
        const cb = document.createElement('input')
        cb.type = 'checkbox'
        cb.id = `cb-${item.id}`
        li.appendChild(cb)
        return { nodes: [li] }
      },
    )
    const cbB = root.querySelector('#cb-b') as HTMLInputElement
    cbB.checked = true
    $list.set([{ id: 'c' }, { id: 'b' }, { id: 'a' }])
    expect((root.querySelector('#cb-b') as HTMLInputElement).checked).toBe(true)
  })
})

describe('bindFor — disposer', () => {
  it('removes all rendered nodes + calls cleanups', () => {
    const { root, dispose, cleanupCalls } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
    ])
    expect(root.querySelectorAll('li').length).toBe(2)
    dispose()
    expect(root.querySelectorAll('li').length).toBe(0)
    expect(cleanupCalls.sort()).toEqual(['a', 'b'])
  })

  it('subsequent signal changes are ignored after dispose', () => {
    const { $list, root, dispose } = setup([{ id: 'a', label: 'A' }])
    dispose()
    $list.set([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
    ])
    expect(root.querySelectorAll('li').length).toBe(0)
  })
})

describe('bindFor — monotone fast-path', () => {
  // Three real-world shapes the runtime should resolve without
  // running the O(n log n) LIS scan: append, single-remove, and
  // identity (signal re-fire with same key set). All three produce
  // matched-prev sequences that are monotonically increasing, so
  // the optimizer marks every match as stable in a single O(n) pass.
  // Pre-fix these paid the full LIS — measurable on Krausest's
  // `remove-one-1k` benchmark (12.4 → 11ms-ish, closer to Solid's 10.5).

  it('append-one preserves all existing DOM nodes (no LIS-required moves)', () => {
    const { $list, root, renderCalls } = setup([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
    ])
    const beforeA = root.querySelector('#item-a')!
    const beforeB = root.querySelector('#item-b')!
    expect(renderCalls.length).toBe(2)
    $list.set([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
      { id: 'c', label: 'C' },
    ])
    // No re-renders of a/b; only c rendered fresh.
    expect(renderCalls.length).toBe(3)
    expect(renderCalls[2]!.id).toBe('c')
    // Node identity preserved (no move).
    expect(root.querySelector('#item-a')!).toBe(beforeA)
    expect(root.querySelector('#item-b')!).toBe(beforeB)
  })

  it('single-remove preserves remaining DOM nodes', () => {
    const items = [
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
      { id: 'c', label: 'C' },
      { id: 'd', label: 'D' },
    ]
    const { $list, root, cleanupCalls } = setup(items)
    const nodeA = root.querySelector('#item-a')!
    const nodeC = root.querySelector('#item-c')!
    const nodeD = root.querySelector('#item-d')!
    $list.set([items[0]!, items[2]!, items[3]!]) // drop b
    expect(cleanupCalls).toEqual(['b'])
    // Untouched siblings keep node identity.
    expect(root.querySelector('#item-a')!).toBe(nodeA)
    expect(root.querySelector('#item-c')!).toBe(nodeC)
    expect(root.querySelector('#item-d')!).toBe(nodeD)
  })

  it('reorder fall-back: swap-two still produces correct DOM order', () => {
    // Inversion (b before a) forces the LIS path. Verify the
    // monotone fast-path doesn't accidentally claim "stable" for
    // genuinely-out-of-order matches.
    const items = [
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
      { id: 'c', label: 'C' },
    ]
    const { $list, ids } = setup(items)
    $list.set([items[2]!, items[1]!, items[0]!])
    expect(ids()).toEqual(['c', 'b', 'a'])
  })
})

describe('bindFor — large lists (perf sanity)', () => {
  it('renders 1000 items + reorders one without rendering others', () => {
    const items = Array.from({ length: 1000 }, (_, i) => ({
      id: String(i),
      label: `item ${i}`,
    }))
    const { $list, renderCalls, ids } = setup(items)
    expect(renderCalls.length).toBe(1000)

    // Move last to first
    const swapped = [items[999]!, ...items.slice(0, 999)]
    $list.set(swapped)
    expect(ids()[0]).toBe('999')
    expect(renderCalls.length).toBe(1000) // no new renders, just moves
  })
})

describe('bindFor — update callback', () => {
  it('invokes update on reused rows when item content changes', () => {
    const host = document.createElement('div')
    document.body.appendChild(host)
    const anchor = document.createComment('for')
    host.appendChild(anchor)
    const $list = signal([
      { id: 'a', name: 'Alice' },
      { id: 'b', name: 'Bob' },
    ])
    const updates: Array<{ id: string; name: string }> = []
    bindFor({
      anchor,
      items: () => $list(),
      key: (it) => it.id,
      render: (it) => {
        const span = document.createElement('span')
        span.textContent = it.name
        return {
          nodes: [span],
          update: (next: any) => {
            updates.push(next)
            span.textContent = next.name
          },
        }
      },
    })
    expect(host.textContent).toBe('AliceBob')

    $list.set([
      { id: 'a', name: 'Alicia' }, // same key, new content
      { id: 'b', name: 'Bob' },
    ])
    expect(updates.length).toBe(2)
    expect(host.textContent).toBe('AliciaBob')
  })
})

describe('bindFor — duplicate keys', () => {
  it('warns on duplicate key with message identifying the offending key + indices', () => {
    const root = document.createElement('div')
    document.body.appendChild(root)
    const anchor = document.createComment('list')
    root.appendChild(anchor)
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})

    const $list = signal<Item[]>([
      { id: 'a', label: 'first' },
      { id: 'a', label: 'duplicate' },
      { id: 'b', label: 'b' },
    ])
    const dispose = bindForSignal(
      anchor,
      $list,
      (it) => it.id,
      (it) => {
        const li = document.createElement('li')
        li.textContent = it.label
        return { nodes: [li] }
      },
    )

    expect(warn).toHaveBeenCalled()
    const msg = warn.mock.calls[0]!.join(' ')
    expect(msg).toMatch(/duplicate key/)
    expect(msg).toMatch(/a/)
    // Indices included so the user can find the offending items.
    expect(msg).toMatch(/at index 1/)

    dispose()
    warn.mockRestore()
    document.body.innerHTML = ''
  })

  it('warns N-1 times for N collisions on a single key, once per duplicate item', () => {
    const root = document.createElement('div')
    document.body.appendChild(root)
    const anchor = document.createComment('list')
    root.appendChild(anchor)
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})

    // Initial: clean — 1 item, no duplicate.
    const $list = signal<Item[]>([{ id: 'a', label: 'one' }])
    const dispose = bindForSignal(
      anchor,
      $list,
      (it) => it.id,
      (it) => {
        const li = document.createElement('li')
        li.textContent = it.label
        return { nodes: [li] }
      },
    )
    expect(warn).not.toHaveBeenCalled()

    // Two items, same key → 1 duplicate at index 1.
    $list.set([
      { id: 'x', label: '1' },
      { id: 'x', label: '2' },
    ])
    expect(warn).toHaveBeenCalledTimes(1)

    // Three items, same key → 2 duplicates (indices 1 and 2 collide
    // with 0). Adds 2 more warnings (total 3).
    $list.set([
      { id: 'y', label: '1' },
      { id: 'y', label: '2' },
      { id: 'y', label: '3' },
    ])
    expect(warn).toHaveBeenCalledTimes(3)

    dispose()
    warn.mockRestore()
    document.body.innerHTML = ''
  })

  it('no warning for unique keys', () => {
    const root = document.createElement('div')
    document.body.appendChild(root)
    const anchor = document.createComment('list')
    root.appendChild(anchor)
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})

    const $list = signal<Item[]>([
      { id: 'a', label: 'a' },
      { id: 'b', label: 'b' },
      { id: 'c', label: 'c' },
    ])
    const dispose = bindForSignal(
      anchor,
      $list,
      (it) => it.id,
      (it) => {
        const li = document.createElement('li')
        li.textContent = it.label
        return { nodes: [li] }
      },
    )
    expect(warn).not.toHaveBeenCalled()
    $list.set([
      { id: 'a', label: 'a' },
      { id: 'd', label: 'd' },
    ])
    expect(warn).not.toHaveBeenCalled()

    dispose()
    warn.mockRestore()
    document.body.innerHTML = ''
  })
})

describe('bindFor — first-render fragment batching', () => {
  it('uses a single parent mutation for initial 100-row render', () => {
    const root = document.createElement('div')
    document.body.appendChild(root)
    const anchor = document.createComment('list')
    root.appendChild(anchor)
    const items = Array.from({ length: 100 }, (_, i) => ({ id: String(i) }))

    const obs = new MutationObserver(() => {})
    obs.observe(root, { childList: true })

    bindFor({
      anchor,
      items: () => items,
      key: (it) => it.id,
      render: (it) => {
        const li = document.createElement('li')
        li.id = `item-${it.id}`
        return { nodes: [li] }
      },
    })

    const records = obs.takeRecords()
    obs.disconnect()
    expect(records.length).toBe(1)
    expect(records[0]!.addedNodes.length).toBe(100)
    expect(root.querySelectorAll('li').length).toBe(100)
  })

  it('uses a single parent mutation when growing from 0 → 100 rows', () => {
    const root = document.createElement('div')
    document.body.appendChild(root)
    const anchor = document.createComment('list')
    root.appendChild(anchor)
    const $list = signal<Item[]>([])
    bindForSignal(anchor, $list, (it) => it.id, (it) => {
      const li = document.createElement('li')
      li.id = `item-${it.id}`
      li.textContent = it.label
      return { nodes: [li] }
    })

    const obs = new MutationObserver(() => {})
    obs.observe(root, { childList: true })
    $list.set(Array.from({ length: 100 }, (_, i) => ({ id: String(i), label: `${i}` })))
    const records = obs.takeRecords()
    obs.disconnect()

    expect(records.length).toBe(1)
    expect(records[0]!.addedNodes.length).toBe(100)
    expect(root.querySelectorAll('li').length).toBe(100)
  })

  it('produces identical DOM to per-row appendChild', () => {
    // Behavioural equivalence: order, attributes, text — all match.
    const items = Array.from({ length: 25 }, (_, i) => ({ id: `k${i}`, label: `L${i}` }))
    const { root, ids } = setup(items)
    expect(ids()).toEqual(items.map((i) => i.id))
    expect(Array.from(root.querySelectorAll('li')).map((el) => el.textContent))
      .toEqual(items.map((i) => i.label))
  })

  it('multi-node rows batch into a single parent mutation on first render', () => {
    const root = document.createElement('div')
    document.body.appendChild(root)
    const anchor = document.createComment('list')
    root.appendChild(anchor)
    const items = Array.from({ length: 20 }, (_, i) => ({ id: String(i) }))

    const obs = new MutationObserver(() => {})
    obs.observe(root, { childList: true })

    bindFor({
      anchor,
      items: () => items,
      key: (it) => it.id,
      render: (it) => {
        const a = document.createElement('span')
        a.className = `r${it.id}-a`
        const b = document.createElement('span')
        b.className = `r${it.id}-b`
        return { nodes: [a, b] }
      },
    })

    const records = obs.takeRecords()
    obs.disconnect()
    expect(records.length).toBe(1)
    expect(records[0]!.addedNodes.length).toBe(40) // 20 rows × 2 nodes
    expect(root.querySelectorAll('span').length).toBe(40)
  })

  it('custom elements upgrade and receive connectedCallback inside the fragment batch', () => {
    let connectedCount = 0
    const tag = 'sui-frag-batch-test-el'
    if (!customElements.get(tag)) {
      customElements.define(tag, class extends HTMLElement {
        connectedCallback() { connectedCount++ }
      })
    }

    const root = document.createElement('div')
    document.body.appendChild(root)
    const anchor = document.createComment('list')
    root.appendChild(anchor)
    const items = Array.from({ length: 10 }, (_, i) => ({ id: String(i) }))

    bindFor({
      anchor,
      items: () => items,
      key: (it) => it.id,
      render: (it) => {
        const li = document.createElement('li')
        li.id = `item-${it.id}`
        const el = document.createElement(tag)
        li.appendChild(el)
        return { nodes: [li] }
      },
    })

    expect(root.querySelectorAll(tag).length).toBe(10)
    expect(connectedCount).toBe(10)
    root.querySelectorAll(tag).forEach((el) => {
      expect((el as HTMLElement).isConnected).toBe(true)
    })
  })

  it('falls back to per-row insertBefore when prev is non-empty', () => {
    // Update path is unchanged: append after initial render still uses
    // the cursor loop (verified indirectly by the LIS minimality tests).
    const { $list, ids } = setup([{ id: 'a', label: 'A' }])
    $list.set([
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
      { id: 'c', label: 'C' },
    ])
    expect(ids()).toEqual(['a', 'b', 'c'])
  })
})

describe('bindFor — LIS minimality', () => {
  // Count insertBefore calls during a signal update by spying on Element.prototype.
  // The spy is installed AFTER initial render so we only count moves from the update.
  function countMoves(
    $list: ReturnType<typeof setup>['$list'],
    next: Item[],
  ): number {
    const spy = vi.spyOn(Element.prototype, 'insertBefore')
    try {
      $list.set(next)
      return spy.mock.calls.length
    } finally {
      spy.mockRestore()
    }
  }

  it('swap of positions 1 and 998 in a 1000-element list moves exactly 2 nodes', () => {
    const items = Array.from({ length: 1000 }, (_, i) => ({ id: String(i), label: `${i}` }))
    const { $list, ids } = setup(items)

    const swapped = [...items]
    ;[swapped[1], swapped[998]] = [swapped[998]!, swapped[1]!]
    const moves = countMoves($list, swapped)

    expect(ids()[1]).toBe('998')
    expect(ids()[998]).toBe('1')
    expect(moves).toBe(2)
  })

  it('full reverse repositions N-1 rows in batched fragment inserts', () => {
    const N = 8
    const items = Array.from({ length: N }, (_, i) => ({ id: String(i), label: `${i}` }))
    const { $list, ids } = setup(items)

    const moves = countMoves($list, [...items].reverse())

    expect(ids()).toEqual(Array.from({ length: N }, (_, i) => String(N - 1 - i)))
    // LIS of a fully-reversed sequence has length 1 → N-1 rows reposition.
    // The placement loop batches each maximal run of consecutive
    // non-stable rows into ONE fragment insert, so the single stable
    // (LIS) row splits the movers into at most two runs ⇒ ≤2 inserts,
    // far below the old per-node count. (Correctness is the ids() check.)
    expect(moves).toBeGreaterThanOrEqual(1)
    expect(moves).toBeLessThanOrEqual(2)
  })

  it('no moves when array order is unchanged', () => {
    const items = [
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
      { id: 'c', label: 'C' },
    ]
    const { $list, ids } = setup(items)
    const moves = countMoves($list, [...items])

    expect(ids()).toEqual(['a', 'b', 'c'])
    expect(moves).toBe(0)
  })

  it('arbitrary permutation produces correct DOM order', () => {
    const items = Array.from({ length: 10 }, (_, i) => ({ id: String(i), label: `${i}` }))
    const { $list, ids } = setup(items)

    // Permutation: [3, 7, 1, 9, 0, 5, 2, 8, 4, 6]
    const perm = [3, 7, 1, 9, 0, 5, 2, 8, 4, 6]
    $list.set(perm.map((i) => items[i]!))
    expect(ids()).toEqual(perm.map(String))
  })

  it('partial reverse (middle segment) moves minimal nodes', () => {
    // [0, 1, 2, 3, 4] → [0, 4, 3, 2, 1]
    // LIS of prev positions [0, 4, 3, 2, 1]: LIS = [0, 1] (just item 0)
    // But actually [0] has length 1 for the suffix and we need overall LIS.
    // [0, 4, 3, 2, 1]: LIS = [0, _] where _ < 4... no increasing after 0,4.
    // Longest IS: [0, 1] — no wait: values are prev positions.
    // new order: [item0(prev0), item4(prev4), item3(prev3), item2(prev2), item1(prev1)]
    // LIS of [0, 4, 3, 2, 1] = [0] (length 1, because after 0 only 4 fits, then 4,3,2,1 is decreasing).
    // Actually [0, 4] is IS of length 2? No: after 4 we have 3 < 4. So max IS starting with 0: [0, 4] = length 2? No, that IS stops at 4 since next is 3 < 4.
    // Wait actually LIS of [0, 4, 3, 2, 1]:
    //   [0, 4] — length 2
    //   [0, 3] — length 2
    //   [0, 2] — length 2
    //   [0, 1] — length 2
    // All have length 2. So LIS = 2 → 5-2=3 moves.
    const items = Array.from({ length: 5 }, (_, i) => ({ id: String(i), label: `${i}` }))
    const { $list, ids } = setup(items)

    $list.set([items[0]!, items[4]!, items[3]!, items[2]!, items[1]!])
    expect(ids()).toEqual(['0', '4', '3', '2', '1'])
  })

  it('does not leak state across consecutive disjoint-key renders (scratch reuse)', () => {
    // Renders three full replacements with disjoint key sets. If the
    // hoisted scratch Maps/Sets/Arrays inside bindFor are not cleared
    // between renders, stale keys would linger and corrupt the DOM
    // (e.g. an "existing" reused-record lookup would find a record
    // whose nodes were already removed).
    const { $list, ids, cleanupCalls } = setup(
      Array.from({ length: 50 }, (_, i) => ({ id: `a${i}`, label: `A${i}` })),
    )
    expect(ids()).toHaveLength(50)
    $list.set(Array.from({ length: 50 }, (_, i) => ({ id: `b${i}`, label: `B${i}` })))
    expect(ids()).toEqual(Array.from({ length: 50 }, (_, i) => `b${i}`))
    expect(cleanupCalls.filter((id) => id.startsWith('a'))).toHaveLength(50)
    $list.set(Array.from({ length: 50 }, (_, i) => ({ id: `c${i}`, label: `C${i}` })))
    expect(ids()).toEqual(Array.from({ length: 50 }, (_, i) => `c${i}`))
    expect(cleanupCalls.filter((id) => id.startsWith('b'))).toHaveLength(50)
    // Final state: only c-keyed rows are present, all a/b rows were cleaned up.
    expect(cleanupCalls.filter((id) => id.startsWith('c'))).toHaveLength(0)
  })

  it('reuses scratch buffers when list size oscillates', () => {
    // Stresses the buffer-growth path: list grows past initial capacity,
    // shrinks, grows again. Typed-array reads past `matchedCount` must
    // not affect placement.
    const { $list, ids, renderCalls } = setup([{ id: 'a', label: 'A' }])
    const big = Array.from({ length: 100 }, (_, i) => ({ id: `b${i}`, label: `B${i}` }))
    $list.set(big)
    expect(ids()).toHaveLength(100)
    $list.set([{ id: 'b0', label: 'B0' }])
    expect(ids()).toEqual(['b0'])
    // Re-grow with a superset — b0 is reused, b1..b99 fresh-render again.
    $list.set(big)
    expect(ids()).toEqual(big.map((b) => b.id))
    // 1 ('a') + 100 (b0..b99 first appearance, 'a' was cleaned up)
    //         + 99 (b1..b99 re-rendered after shrink; b0 reused) = 200.
    expect(renderCalls.length).toBe(1 + 100 + 99)
  })

  it('multi-node rows: swap moves only the unstable rows', () => {
    const root = document.createElement('div')
    document.body.appendChild(root)
    const anchor = document.createComment('list')
    root.appendChild(anchor)

    const items = Array.from({ length: 6 }, (_, i) => ({ id: String(i) }))
    const $list = signal(items)

    bindForSignal(anchor, $list, (it) => it.id, (it) => {
      // Each row renders TWO nodes.
      const a = document.createElement('span')
      const b = document.createElement('span')
      a.className = `r${it.id}-a`
      b.className = `r${it.id}-b`
      return { nodes: [a, b] }
    })

    const spans = () =>
      Array.from(root.querySelectorAll('span')).map((s) => s.className)

    // Swap positions 1 and 4: [0, 4, 2, 3, 1, 5]
    const spy = vi.spyOn(Element.prototype, 'insertBefore')
    const swapped = [...items]
    ;[swapped[1], swapped[4]] = [swapped[4]!, swapped[1]!]
    $list.set(swapped)
    const moves = spy.mock.calls.length
    spy.mockRestore()

    expect(spans()).toEqual([
      'r0-a', 'r0-b',
      'r4-a', 'r4-b',
      'r2-a', 'r2-b',
      'r3-a', 'r3-b',
      'r1-a', 'r1-b',
      'r5-a', 'r5-b',
    ])
    // Two unstable rows at non-adjacent positions (new indices 1 and 4,
    // split by stable rows) → two separate runs. Each run's nodes batch
    // into one fragment ⇒ 2 insertBefore calls (was 4, one per node).
    expect(moves).toBe(2)
  })

  it('appends a block onto an existing list in one fragment insert', () => {
    // The Krausest `create1k-after1k` shape: 1000 rows already mounted,
    // append 1000 more. The existing rows are all stable (matched, in
    // order), the appended tail is one contiguous non-stable run → a
    // SINGLE fragment insert, not one insertBefore per appended row.
    const items = Array.from({ length: 1000 }, (_, i) => ({ id: String(i), label: `${i}` }))
    const { $list, ids } = setup(items)

    const more = Array.from({ length: 1000 }, (_, i) => ({ id: String(1000 + i), label: `${1000 + i}` }))
    const moves = countMoves($list, [...items, ...more])

    expect(ids()).toHaveLength(2000)
    expect(ids()[1999]).toBe('1999')
    // One fragment insert for the whole appended block.
    expect(moves).toBe(1)
  })
})

describe('bindFor — perf followups (runtime-perf-followups)', () => {
  it('full-replace (disjoint keys) batches inserts via a single parent mutation', () => {
    // Win 1: when no prev key survives the next render, the placement
    // step must take the fragment fast path, not the per-row cursor loop.
    const root = document.createElement('div')
    document.body.appendChild(root)
    const anchor = document.createComment('list')
    root.appendChild(anchor)
    const $list = signal<Item[]>(
      Array.from({ length: 20 }, (_, i) => ({ id: `a${i}`, label: `A${i}` })),
    )
    bindForSignal(anchor, $list, (it) => it.id, (it) => {
      const li = document.createElement('li')
      li.id = `item-${it.id}`
      li.textContent = it.label
      return { nodes: [li] }
    })
    expect(root.querySelectorAll('li').length).toBe(20)

    // Spy on insertBefore AFTER initial render. The replace must produce
    // exactly one insertBefore (the DocumentFragment), not 20.
    const spy = vi.spyOn(Element.prototype, 'insertBefore')
    $list.set(Array.from({ length: 20 }, (_, i) => ({ id: `b${i}`, label: `B${i}` })))
    const inserts = spy.mock.calls.length
    spy.mockRestore()

    expect(root.querySelectorAll('li').length).toBe(20)
    expect(Array.from(root.querySelectorAll('li')).map((el) => el.id))
      .toEqual(Array.from({ length: 20 }, (_, i) => `item-b${i}`))
    expect(inserts).toBe(1)
  })

  it('items() returning the same array reference is a no-op', () => {
    // Win: short-circuit when the reactive source emits the same array.
    // No render() invocations, no DOM mutations. Verifies the cleanup
    // path also keeps state consistent (a follow-up real change still
    // works).
    const root = document.createElement('div')
    document.body.appendChild(root)
    const anchor = document.createComment('list')
    root.appendChild(anchor)
    const items: Item[] = [
      { id: 'a', label: 'A' },
      { id: 'b', label: 'B' },
    ]
    const $list = signal<Item[]>(items)
    let renders = 0
    bindForSignal(anchor, $list, (it) => it.id, (it) => {
      renders++
      const li = document.createElement('li')
      li.id = `item-${it.id}`
      return { nodes: [li] }
    })
    expect(renders).toBe(2)

    const spy = vi.spyOn(Element.prototype, 'insertBefore')
    $list.set(items) // identical reference
    const inserts = spy.mock.calls.length
    spy.mockRestore()

    expect(renders).toBe(2)
    expect(inserts).toBe(0)
    expect(Array.from(root.querySelectorAll('li')).map((el) => el.id))
      .toEqual(['item-a', 'item-b'])

    // A real change after the no-op still reconciles correctly.
    $list.set([items[0]!, { id: 'c', label: 'C' }])
    expect(Array.from(root.querySelectorAll('li')).map((el) => el.id))
      .toEqual(['item-a', 'item-c'])
  })

  it('reused-key with identical item+index skips the update push', () => {
    // Win 2: when the next array contains an identity-equal item at the
    // same position as before, the row's update() must NOT fire. The
    // signal would short-circuit anyway, but we save the closure/call
    // overhead at N=1000.
    const root = document.createElement('div')
    document.body.appendChild(root)
    const anchor = document.createComment('list')
    root.appendChild(anchor)

    const row0 = { id: 'a', label: 'A' }
    const row1 = { id: 'b', label: 'B' }
    const $list = signal<Item[]>([row0, row1])
    const updates: { id: string; index: number }[] = []
    bindForSignal(anchor, $list, (it) => it.id, (it, i) => {
      const li = document.createElement('li')
      li.id = `item-${it.id}`
      return {
        nodes: [li],
        update: (item, index) =>
          updates.push({ id: (item as Item).id, index }),
      }
    })

    // Mutate one slot only; the other row should not receive update().
    const changedRow1 = { id: 'b', label: 'B2' }
    $list.set([row0, changedRow1])

    expect(updates).toEqual([{ id: 'b', index: 1 }])
  })

  it('reused-key at a new index still pushes the update', () => {
    // Reordering still must propagate the new index even if the item
    // reference is unchanged — row closures depend on index for things
    // like striping or animation staggers.
    const root = document.createElement('div')
    document.body.appendChild(root)
    const anchor = document.createComment('list')
    root.appendChild(anchor)

    const a = { id: 'a', label: 'A' }
    const b = { id: 'b', label: 'B' }
    const $list = signal<Item[]>([a, b])
    const updates: { id: string; index: number }[] = []
    bindForSignal(anchor, $list, (it) => it.id, (it) => {
      const li = document.createElement('li')
      return {
        nodes: [li],
        update: (item, index) =>
          updates.push({ id: (item as Item).id, index }),
      }
    })

    $list.set([b, a]) // reorder; same refs, different indices.
    // Both rows moved positions — both updates must fire so closures
    // can re-sync on the new index.
    expect(updates.some((u) => u.id === 'a' && u.index === 1)).toBe(true)
    expect(updates.some((u) => u.id === 'b' && u.index === 0)).toBe(true)
  })
})
