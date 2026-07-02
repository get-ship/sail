import { describe, expect, it, vi } from 'vitest'
import { signal, derived, effect, batch, untrack, selector } from '../src/signal'

describe('signal', () => {
  it('reads return the current value', () => {
    const $count = signal(42)
    expect($count()).toBe(42)
  })

  it('writes update the value', () => {
    const $count = signal(0)
    $count.set(5)
    expect($count()).toBe(5)
  })

  it('write with function form gets previous value', () => {
    const $count = signal(10)
    $count.set((prev) => prev * 2)
    expect($count()).toBe(20)
  })

  it('Object.is equal writes do not notify', () => {
    const $count = signal(7)
    const fn = vi.fn()
    effect(() => {
      $count()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    $count.set(7)
    expect(fn).toHaveBeenCalledTimes(1)
  })

  it('writes notify subscribers', () => {
    const $count = signal(0)
    const fn = vi.fn()
    effect(() => {
      $count()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    $count.set(1)
    expect(fn).toHaveBeenCalledTimes(2)
    $count.set(2)
    expect(fn).toHaveBeenCalledTimes(3)
  })

  it('peek does not subscribe', () => {
    const $count = signal(0)
    const fn = vi.fn()
    effect(() => {
      $count.__peek()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    $count.set(99)
    expect(fn).toHaveBeenCalledTimes(1)
  })
})

describe('effect', () => {
  it('runs once eagerly', () => {
    const fn = vi.fn()
    effect(fn)
    expect(fn).toHaveBeenCalledTimes(1)
  })

  it('cleanup runs before next re-run', () => {
    const $x = signal(0)
    const cleanup = vi.fn()
    effect(() => {
      $x()
      return cleanup
    })
    expect(cleanup).toHaveBeenCalledTimes(0)
    $x.set(1)
    expect(cleanup).toHaveBeenCalledTimes(1)
    $x.set(2)
    expect(cleanup).toHaveBeenCalledTimes(2)
  })

  it('disposer stops re-runs', () => {
    const $x = signal(0)
    const fn = vi.fn()
    const dispose = effect(() => {
      $x()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    dispose()
    $x.set(1)
    expect(fn).toHaveBeenCalledTimes(1) // unchanged
  })

  it('disposer runs final cleanup', () => {
    const $x = signal(0)
    const cleanup = vi.fn()
    const dispose = effect(() => {
      $x()
      return cleanup
    })
    dispose()
    expect(cleanup).toHaveBeenCalledTimes(1)
  })
})

describe('derived', () => {
  it('memoizes its result', () => {
    const $x = signal(2)
    const compute = vi.fn(() => $x() * 3)
    const $tripled = derived(compute)
    expect($tripled()).toBe(6)
    expect($tripled()).toBe(6) // re-read
    expect(compute).toHaveBeenCalledTimes(1)
  })

  it('recomputes when a dep changes', () => {
    const $x = signal(2)
    const $tripled = derived(() => $x() * 3)
    expect($tripled()).toBe(6)
    $x.set(5)
    expect($tripled()).toBe(15)
  })

  it('downstream effect re-runs when derived changes', () => {
    const $x = signal(1)
    const $double = derived(() => $x() * 2)
    const fn = vi.fn()
    effect(() => {
      $double()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    $x.set(2)
    expect(fn).toHaveBeenCalledTimes(2)
  })

  it('throws on .set()', () => {
    const $x = signal(0)
    const $d = derived(() => $x() + 1)
    expect(() => $d.set(99)).toThrow(/derived/)
  })

  it('skips notify when recomputed value is Object.is-equal', () => {
    const $count = signal(0)
    const $positive = derived(() => $count() > 0)
    const fn = vi.fn()
    effect(() => {
      $positive()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    // 0 → 5 flips false → true; subscriber should fire.
    $count.set(5)
    expect(fn).toHaveBeenCalledTimes(2)
    // 5 → 10 stays true → true; subscriber should NOT fire.
    $count.set(10)
    expect(fn).toHaveBeenCalledTimes(2)
    $count.set(20)
    expect(fn).toHaveBeenCalledTimes(2)
    // 20 → -1 flips true → false; subscriber fires again.
    $count.set(-1)
    expect(fn).toHaveBeenCalledTimes(3)
  })

  it('preserves identity-equal object refs (immutable-ref contract)', () => {
    // Holding the same object ref across a "write" looks unchanged to
    // the derived — callers must produce a fresh ref to signal a change.
    const obj = { a: 1 }
    const $s = signal(obj)
    const $d = derived(() => $s())
    const fn = vi.fn()
    effect(() => {
      $d()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    $s.set(obj)
    expect(fn).toHaveBeenCalledTimes(1)
    $s.set({ a: 1 }) // new ref → does fire
    expect(fn).toHaveBeenCalledTimes(2)
  })

  it('recomputes lazily when no subscribers exist', () => {
    // Without downstream subs, invalidation should stay lazy: the
    // derived recomputes on the next read, not eagerly on dep change.
    const $x = signal(1)
    const compute = vi.fn(() => $x() * 2)
    const $d = derived(compute)
    expect($d()).toBe(2)
    expect(compute).toHaveBeenCalledTimes(1)
    $x.set(2)
    $x.set(3)
    $x.set(4)
    expect(compute).toHaveBeenCalledTimes(1)
    expect($d()).toBe(8)
    expect(compute).toHaveBeenCalledTimes(2)
  })

  it('chained derived also memoizes downstream', () => {
    const $x = signal(0)
    const $isPositive = derived(() => $x() > 0)
    const $label = derived(() => ($isPositive() ? 'yes' : 'no'))
    const computeLabel = vi.fn(() => $label())
    const fn = vi.fn(computeLabel)
    effect(fn)
    expect(fn).toHaveBeenCalledTimes(1)
    $x.set(1)
    expect(fn).toHaveBeenCalledTimes(2)
    $x.set(2)
    // $isPositive stayed true → $label stayed 'yes' → no re-run.
    expect(fn).toHaveBeenCalledTimes(2)
  })
})

describe('batch', () => {
  it('coalesces multiple writes into one notification', () => {
    const $a = signal(0)
    const $b = signal(0)
    const fn = vi.fn()
    effect(() => {
      $a()
      $b()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    batch(() => {
      $a.set(1)
      $b.set(2)
    })
    expect(fn).toHaveBeenCalledTimes(2) // initial + 1 batched
  })

  it('nested batches still coalesce to outermost', () => {
    const $a = signal(0)
    const fn = vi.fn()
    effect(() => {
      $a()
      fn()
    })
    batch(() => {
      $a.set(1)
      batch(() => {
        $a.set(2)
        $a.set(3)
      })
      $a.set(4)
    })
    expect(fn).toHaveBeenCalledTimes(2) // initial + 1 batched
  })
})

describe('untrack', () => {
  it('reads inside untrack do not subscribe', () => {
    const $x = signal(0)
    const $y = signal(0)
    const fn = vi.fn()
    effect(() => {
      $x()
      untrack(() => $y())
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    $y.set(1)
    expect(fn).toHaveBeenCalledTimes(1) // y did not trigger
    $x.set(1)
    expect(fn).toHaveBeenCalledTimes(2) // x did
  })
})

describe('selector', () => {
  it('returns true only for the current value', () => {
    const $sel = signal<string | null>(null)
    const isSelected = selector($sel)
    expect(isSelected('a')).toBe(false)
    $sel.set('a')
    expect(isSelected('a')).toBe(true)
    expect(isSelected('b')).toBe(false)
  })

  it('notifies only the two affected buckets on change (not all subscribers)', () => {
    // The Krausest motivation: with 1000 rows binding to selected,
    // a change between two ids should re-fire just two effects, not
    // 1000. Track each row's fire count to prove the others stay quiet.
    const $sel = signal<number>(-1)
    const isSelected = selector($sel)
    const fires: Record<number, number> = {}
    for (let id = 0; id < 5; id++) {
      fires[id] = 0
      effect(() => {
        isSelected(id) // subscribe
        fires[id]!++
      })
    }
    // Initial run fires each effect once.
    expect(fires).toEqual({ 0: 1, 1: 1, 2: 1, 3: 1, 4: 1 })

    $sel.set(2)
    // Only id=2's effect re-fires (became selected). id=-1 has no
    // subscriber, so no notify there.
    expect(fires).toEqual({ 0: 1, 1: 1, 2: 2, 3: 1, 4: 1 })

    $sel.set(4)
    // Both old (2) and new (4) re-fire; others quiet.
    expect(fires).toEqual({ 0: 1, 1: 1, 2: 3, 3: 1, 4: 2 })

    $sel.set(4)
    // No-op set — Object.is equality, no notifies.
    expect(fires).toEqual({ 0: 1, 1: 1, 2: 3, 3: 1, 4: 2 })
  })

  it('untracks observers on re-run (no stale subscriptions across runs)', () => {
    const $sel = signal<string>('a')
    const $which = signal<'x' | 'y'>('x')
    const isSelected = selector($sel)
    const fires = vi.fn()
    effect(() => {
      // Read the bucket for whichever key $which points to. After
      // $which flips to 'y', the 'x' bucket subscription must clear
      // — otherwise selecting 'x' would re-fire this effect.
      isSelected($which())
      fires()
    })
    expect(fires).toHaveBeenCalledTimes(1)
    $which.set('y') // re-fires (which changed)
    expect(fires).toHaveBeenCalledTimes(2)
    $sel.set('x') // 'x' bucket should have no subscribers left
    expect(fires).toHaveBeenCalledTimes(2)
    $sel.set('y') // 'y' has us, fires
    expect(fires).toHaveBeenCalledTimes(3)
  })

  it('batch groups selector notifies', () => {
    const $sel = signal<number>(-1)
    const isSelected = selector($sel)
    const fires = vi.fn()
    effect(() => {
      isSelected(1)
      isSelected(2)
      fires()
    })
    expect(fires).toHaveBeenCalledTimes(1)
    batch(() => {
      $sel.set(1)
      $sel.set(2)
    })
    // Two changes, one effect run (1 became sel then desel; 2 became
    // sel). End state: 2 is selected. Effect re-runs once.
    expect(fires).toHaveBeenCalledTimes(2)
  })
})

describe('nested-effect owner tree (auto-cleanup)', () => {
  it('disposes a nested effect when its parent re-runs — no leak, no double-fire', () => {
    const outer = signal(0)
    const inner = signal(0)
    const innerRun = vi.fn()

    effect(() => {
      outer() // parent dependency
      effect(() => {
        inner()
        innerRun()
      })
    })

    expect(innerRun).toHaveBeenCalledTimes(1) // the child's initial run
    inner.set(1)
    expect(innerRun).toHaveBeenCalledTimes(2) // the one live child fires

    outer.set(1) // parent re-runs: old child disposed, new child created (its initial run)
    expect(innerRun).toHaveBeenCalledTimes(3)

    inner.set(2) // only the NEW child is alive → +1. (Pre-fix, the orphaned old
    expect(innerRun).toHaveBeenCalledTimes(4) //  child would still fire too → 5.)
  })

  it('disposes nested effects when the parent effect is disposed', () => {
    const inner = signal(0)
    const innerRun = vi.fn()

    const disposeParent = effect(() => {
      effect(() => {
        inner()
        innerRun()
      })
    })

    expect(innerRun).toHaveBeenCalledTimes(1)
    disposeParent() // parent disposal runs its cleanups → the child disposes too
    inner.set(1)
    expect(innerRun).toHaveBeenCalledTimes(1) // child no longer fires
  })
})
