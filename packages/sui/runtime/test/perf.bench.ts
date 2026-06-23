// Performance microbenchmarks for the SUI runtime.
//
// Run with `bun run --filter @ship/sui-runtime bench`. Results land in stdout.
// Numbers are baselines — what matters is the ratio across changes,
// not the absolute µs/op (which varies by machine).
//
// Covers:
//   1. Signal write throughput (1 effect / N sequential writes)
//   2. Derived recompute chain (3-level)
//   3. bindFor reconciliation on a 1000-item list (insert / remove / shuffle)
//
// Mount cost lives in a separate bench because it touches the DOM and
// has to spin up jsdom — kept here together for one-pass measurement.

import { bench, describe } from 'vitest'
import { signal, derived, effect, batch } from '../src/signal'
import { bindForSignal } from '../src/for'

describe('signal', () => {
  bench('1 effect / 1 signal: 1000 writes', () => {
    const s = signal(0)
    const _d = effect(() => {
      s()
    })
    for (let i = 0; i < 1000; i++) s.set(i)
  })

  bench('1 effect / 1 signal: 1000 batched writes', () => {
    const s = signal(0)
    const _d = effect(() => {
      s()
    })
    batch(() => {
      for (let i = 0; i < 1000; i++) s.set(i)
    })
  })

  bench('1 effect / 10 signals: 100 writes each (1k total)', () => {
    const sigs = Array.from({ length: 10 }, () => signal(0))
    const _d = effect(() => {
      for (const s of sigs) s()
    })
    for (let i = 0; i < 100; i++) {
      for (const s of sigs) s.set(i)
    }
  })

  bench('100 effects / 1 signal: 100 writes (fanout)', () => {
    const s = signal(0)
    const disposers: Array<() => void> = []
    for (let i = 0; i < 100; i++) {
      disposers.push(effect(() => s()))
    }
    for (let i = 0; i < 100; i++) s.set(i)
    for (const d of disposers) d()
  })
})

describe('derived', () => {
  bench('3-level derived chain: 1000 writes', () => {
    const a = signal(0)
    const b = derived(() => a() + 1)
    const c = derived(() => b() * 2)
    const d = derived(() => c() - 3)
    const _e = effect(() => {
      d()
    })
    for (let i = 0; i < 1000; i++) a.set(i)
  })

  bench('conditional derived (toggling deps): 1000 flips', () => {
    const cond = signal(true)
    const a = signal(1)
    const b = signal(2)
    const d = derived(() => (cond() ? a() : b()))
    const _e = effect(() => {
      d()
    })
    for (let i = 0; i < 1000; i++) {
      cond.set(i % 2 === 0)
    }
  })
})

describe('bindFor reconciliation', () => {
  function rowList(n: number): Array<{ id: string; label: string }> {
    return Array.from({ length: n }, (_, i) => ({
      id: `row-${i}`,
      label: `Row ${i}`,
    }))
  }

  function setup(n: number) {
    const root = document.createElement('div')
    document.body.appendChild(root)
    const anchor = document.createComment('list')
    root.appendChild(anchor)
    const $list = signal(rowList(n))
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
    return { root, $list, dispose }
  }

  bench('initial render: 1000 items', () => {
    const { dispose, root } = setup(1000)
    dispose()
    root.remove()
  })

  bench('append 100 to 1000', () => {
    const { $list, dispose, root } = setup(1000)
    const next = [...$list.__peek(), ...rowList(100).map((r) => ({ ...r, id: `extra-${r.id}` }))]
    $list.set(next)
    dispose()
    root.remove()
  })

  bench('prepend 100 to 1000 (worst case)', () => {
    const { $list, dispose, root } = setup(1000)
    const newRows = rowList(100).map((r) => ({ ...r, id: `pre-${r.id}` }))
    $list.set([...newRows, ...$list.__peek()])
    dispose()
    root.remove()
  })

  bench('shuffle 1000 (reverse)', () => {
    const { $list, dispose, root } = setup(1000)
    $list.set([...$list.__peek()].reverse())
    dispose()
    root.remove()
  })

  bench('remove every other from 1000', () => {
    const { $list, dispose, root } = setup(1000)
    $list.set($list.__peek().filter((_, i) => i % 2 === 0))
    dispose()
    root.remove()
  })
})
