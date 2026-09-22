// reactivity.test.ts — the signal graph's behavioral contract, exhaustively.
//
// This is the failure-mode taxonomy for SAIL's fine-grained reactivity, written
// as the DESIRED end-state (not the current state). Tests that are red today
// are the proof-attempts the scheduler fix must turn green — per the honesty
// policy, a red here is a real, un-papered gap, not a TODO.
//
// Organized by failure mode:
//   1. glitch-freedom + exactly-once     (diamonds: the headline defect)
//   2. recompute minimality              (no exponential blow-up on stacked diamonds)
//   3. fan-out + batch                   (one write → one notification cycle)
//   4. equal-value suppression           (no churn when nothing actually changed)
//   5. dynamic / conditional dependencies (deps re-tracked each run; no stale fire)
//   6. laziness + caching                (derived computes on demand, memoizes)
//   7. effect lifecycle                  (cleanup ordering, dispose, nesting)
//   8. untrack                           (reads escape tracking)
//   9. writes during reaction            (effect-writes-signal propagate, terminate)
//  10. selector                          (fine-grained membership; only 2 buckets fire)

import { describe, expect, it, vi } from 'vitest'
import { signal, derived, effect, batch, untrack, selector } from '../src/signal'
import type { Derived } from '../src/signal'

/** A derived that counts how many times its body actually ran. */
function counted<T>(fn: () => T): { d: Derived<T>; runs: () => number } {
  let n = 0
  const d = derived(() => {
    n++
    return fn()
  })
  return { d, runs: () => n }
}

// ── 1. Glitch-freedom + exactly-once ─────────────────────────────────────────
describe('reactivity/glitch-freedom', () => {
  //        a
  //       / \
  //      b   c        b = a+1 , c = a*10
  //       \ /
  //        d          d = b + c   (both paths reconverge)
  function diamond() {
    const a = signal(1)
    const b = derived(() => a() + 1)
    const c = derived(() => a() * 10)
    const d = derived(() => b() + c())
    const seen: number[] = []
    effect(() => seen.push(d())) // initial a=1 → b=2,c=10,d=12
    return { a, b, c, d, seen }
  }

  it('a diamond write updates the dependent exactly once, no stale intermediate', () => {
    const { a, seen } = diamond()
    a.set(2) // b=3, c=20, d=23
    expect(seen).toEqual([12, 23]) // NOT [12, 13, 23]
  })

  it('a batched diamond write is glitch-free and runs the effect once', () => {
    const { a, seen } = diamond()
    batch(() => a.set(2))
    expect(seen).toEqual([12, 23])
  })

  it('two writes in a batch settle to one consistent observation', () => {
    const x = signal(1)
    const y = signal(1)
    const sum = derived(() => x() + y())
    const seen: number[] = []
    effect(() => seen.push(sum()))
    batch(() => {
      x.set(10)
      y.set(20)
    })
    expect(seen).toEqual([2, 30]) // never an intermediate 21 or 11
  })

  it('an effect reading both arms of a diamond never observes a half-updated graph', () => {
    const a = signal(0)
    const b = derived(() => a())
    const c = derived(() => a())
    const pairs: Array<[number, number]> = []
    effect(() => pairs.push([b(), c()]))
    a.set(1)
    // b and c must always agree — [0,0] then [1,1], never [1,0].
    expect(pairs).toEqual([
      [0, 0],
      [1, 1],
    ])
  })
})

// ── 2. Recompute minimality (no exponential blow-up) ─────────────────────────
describe('reactivity/recompute-minimality', () => {
  it('the join of a diamond recomputes once per logical change, not once per path', () => {
    const a = signal(0)
    const b = derived(() => a())
    const c = derived(() => a())
    const join = counted(() => b() + c())
    effect(() => join.d()) // make it live; initial compute = 1
    expect(join.runs()).toBe(1)
    a.set(1)
    expect(join.runs()).toBe(2) // exactly one recompute, not two
  })

  it('stacked diamonds recompute linearly, not exponentially', () => {
    // Three stacked diamonds. Glitchy eager propagation recomputes the final
    // node 2^3 = 8 times per source write; a correct scheduler recomputes once.
    const a = signal(0)
    const layer = (input: Derived<number> | (() => number)) => {
      const b = derived(() => (input as () => number)())
      const c = derived(() => (input as () => number)())
      return derived(() => b() + c())
    }
    const out1 = layer(() => a())
    const out2 = layer(out1)
    const final = counted(() => out2())
    effect(() => final.d()) // initial = 1
    expect(final.runs()).toBe(1)
    a.set(1)
    expect(final.runs()).toBe(2) // one recompute — NOT 1 + 2^k
  })
})

// ── 3. Fan-out + batch ───────────────────────────────────────────────────────
describe('reactivity/fan-out', () => {
  it('one write re-runs each dependent effect exactly once', () => {
    const s = signal(0)
    const a: number[] = []
    const b: number[] = []
    effect(() => a.push(s()))
    effect(() => b.push(s()))
    s.set(1)
    expect(a).toEqual([0, 1])
    expect(b).toEqual([0, 1])
  })

  it('nested batches flush only at the outermost boundary', () => {
    const s = signal(0)
    const seen: number[] = []
    effect(() => seen.push(s()))
    batch(() => {
      s.set(1)
      batch(() => {
        s.set(2)
      })
      s.set(3)
    })
    expect(seen).toEqual([0, 3]) // single flush, final value only
  })
})

// ── 4. Equal-value suppression ───────────────────────────────────────────────
describe('reactivity/equal-value-suppression', () => {
  it('an equal-value signal write notifies nobody', () => {
    const s = signal(7)
    const seen: number[] = []
    effect(() => seen.push(s()))
    s.set(7)
    expect(seen).toEqual([7])
  })

  it('a derived that recomputes to an equal value does not re-run downstream', () => {
    const a = signal(2)
    const parity = derived(() => a() % 2) // 0 for even
    const seen: number[] = []
    effect(() => seen.push(parity()))
    a.set(4) // still even → parity unchanged → effect must NOT re-run
    expect(seen).toEqual([0])
    a.set(5) // now odd → parity flips → effect runs
    expect(seen).toEqual([0, 1])
  })
})

// ── 5. Dynamic / conditional dependencies ────────────────────────────────────
describe('reactivity/dynamic-deps', () => {
  it('re-tracks dependencies each run; a no-longer-read signal stops firing it', () => {
    const cond = signal(true)
    const a = signal('a1')
    const b = signal('b1')
    const probe = counted(() => (cond() ? a() : b()))
    effect(() => probe.d())
    expect(probe.runs()).toBe(1) // reads cond + a

    a.set('a2')
    expect(probe.runs()).toBe(2) // depends on a → recompute
    b.set('b2')
    expect(probe.runs()).toBe(2) // does NOT depend on b → no recompute

    cond.set(false)
    expect(probe.runs()).toBe(3) // now reads cond + b
    a.set('a3')
    expect(probe.runs()).toBe(3) // a dropped → no recompute
    b.set('b3')
    expect(probe.runs()).toBe(4) // b now tracked → recompute
  })
})

// ── 6. Laziness + caching ────────────────────────────────────────────────────
describe('reactivity/laziness-and-caching', () => {
  it('a derived with no subscribers does not compute until first read', () => {
    let ran = false
    const d = derived(() => {
      ran = true
      return 1
    })
    expect(ran).toBe(false) // not computed yet
    expect(d()).toBe(1)
    expect(ran).toBe(true)
  })

  it('a derived memoizes — repeated reads without a dep change compute once', () => {
    const a = signal(1)
    const d = counted(() => a() * 2)
    expect(d.d()).toBe(2)
    expect(d.d()).toBe(2)
    expect(d.d()).toBe(2)
    expect(d.runs()).toBe(1)
    a.set(3)
    expect(d.d()).toBe(6)
    expect(d.runs()).toBe(2)
  })
})

// ── 7. Effect lifecycle ──────────────────────────────────────────────────────
describe('reactivity/effect-lifecycle', () => {
  it('cleanup runs before each re-run and once on dispose', () => {
    const s = signal(0)
    const runs: number[] = []
    const cleanups: number[] = []
    const dispose = effect(() => {
      const v = s()
      runs.push(v)
      return () => cleanups.push(v)
    })
    s.set(1) // cleanup(0), run(1)
    dispose() // cleanup(1)
    s.set(2) // disposed → nothing
    expect(runs).toEqual([0, 1])
    expect(cleanups).toEqual([0, 1])
  })

  it('disposing an effect detaches it from its sources', () => {
    const s = signal(0)
    const seen: number[] = []
    const dispose = effect(() => seen.push(s()))
    dispose()
    s.set(1)
    s.set(2)
    expect(seen).toEqual([0]) // only the initial run
  })

  it('a manually-disposed inner effect does not leak across outer re-runs', () => {
    const outer = signal(0)
    const inner = signal(0)
    const innerSeen: number[] = []
    const dispose = effect(() => {
      outer()
      const stop = effect(() => innerSeen.push(inner()))
      return () => stop() // tear down the inner on every outer re-run
    })
    // initial: inner effect ran once → [0]
    inner.set(1) // current inner effect → [0,1]
    outer.set(1) // outer re-runs: old inner disposed, fresh inner created → [0,1,1]
    inner.set(2) // only ONE live inner effect should fire → [0,1,1,2]
    dispose()
    inner.set(3) // all disposed
    expect(innerSeen).toEqual([0, 1, 1, 2])
  })
})

// ── 8. untrack ───────────────────────────────────────────────────────────────
describe('reactivity/untrack', () => {
  it('reads inside untrack do not subscribe the observer', () => {
    const tracked = signal(0)
    const peeked = signal(0)
    const seen: string[] = []
    effect(() => {
      seen.push(`${tracked()}-${untrack(() => peeked())}`)
    })
    expect(seen).toEqual(['0-0'])
    peeked.set(5) // untracked → no re-run
    expect(seen).toEqual(['0-0'])
    tracked.set(1) // tracked → re-run, reads latest peeked
    expect(seen).toEqual(['0-0', '1-5'])
  })
})

// ── 9. Writes during a reaction ──────────────────────────────────────────────
describe('reactivity/writes-during-reaction', () => {
  it('an effect that writes another signal propagates to its readers', () => {
    const a = signal(1)
    const b = signal(0)
    effect(() => b.set(a() * 2)) // b mirrors a*2
    const seen: number[] = []
    effect(() => seen.push(b()))
    expect(seen).toEqual([2])
    a.set(5)
    expect(seen).toEqual([2, 10])
  })

  it('a self-bounded write loop terminates at its fixpoint', () => {
    const a = signal(0)
    effect(() => {
      if (a() < 3) a.set(a() + 1)
    })
    expect(a()).toBe(3)
  })
})

// ── 10. Selector ─────────────────────────────────────────────────────────────
describe('reactivity/selector', () => {
  it('changing the source re-runs only the old and new buckets', () => {
    const selected = signal<number | null>(null)
    const isSel = selector(selected)
    const runs: Record<number, number> = { 1: 0, 2: 0, 3: 0 }
    for (const id of [1, 2, 3]) {
      effect(() => {
        isSel(id)
        runs[id]++
      })
    }
    expect(runs).toEqual({ 1: 1, 2: 1, 3: 1 }) // initial runs

    selected.set(1) // only bucket 1 flips membership
    expect(runs).toEqual({ 1: 2, 2: 1, 3: 1 })

    selected.set(2) // bucket 1 (off) + bucket 2 (on) flip; 3 untouched
    expect(runs).toEqual({ 1: 3, 2: 2, 3: 1 })
  })
})

// ── 11. Deep linear chains ───────────────────────────────────────────────────
describe('reactivity/deep-chains', () => {
  it('a linear chain settles each node exactly once, in order', () => {
    const a = signal(1)
    const b = counted(() => a() + 1)
    const c = counted(() => b.d() + 1)
    const d = counted(() => c.d() + 1)
    const seen: number[] = []
    effect(() => seen.push(d.d())) // 1→2→3→4
    expect(seen).toEqual([4])
    expect([b.runs(), c.runs(), d.runs()]).toEqual([1, 1, 1])
    a.set(10) // 10→11→12→13
    expect(seen).toEqual([4, 13])
    expect([b.runs(), c.runs(), d.runs()]).toEqual([2, 2, 2])
  })
})

// ── 12. Partial suppression through a diamond ─────────────────────────────────
describe('reactivity/partial-suppression', () => {
  it('an unchanging diamond arm is suppressed while the live arm propagates once', () => {
    const a = signal(2)
    const constArm = counted(() => a() * 0) // always 0 — never changes value
    const liveArm = counted(() => a() * 10)
    const join = counted(() => constArm.d() + liveArm.d())
    const seen: number[] = []
    effect(() => seen.push(join.d())) // 0 + 20 = 20
    expect(seen).toEqual([20])
    a.set(5) // constArm still 0, liveArm 50 → join 50
    expect(seen).toEqual([20, 50])
    // constArm re-evaluated (input changed) but produced an equal value so it
    // did not propagate; join recomputed once because liveArm changed.
    expect(constArm.runs()).toBe(2)
    expect(liveArm.runs()).toBe(2)
    expect(join.runs()).toBe(2)
  })
})

// ── 13. A signal and a derived of it, read by one effect ──────────────────────
describe('reactivity/signal-and-derived', () => {
  it('an effect reading both a signal and a derived of it sees a consistent pair, once', () => {
    const a = signal(1)
    const double = derived(() => a() * 2)
    const pairs: Array<[number, number]> = []
    effect(() => pairs.push([a(), double()]))
    a.set(5)
    expect(pairs).toEqual([
      [1, 2],
      [5, 10],
    ]) // never the glitch pair [5, 2]
  })
})

// ── 14. Effects on intermediate and leaf nodes ────────────────────────────────
describe('reactivity/multiple-effects', () => {
  it('effects on an intermediate and a leaf each run once per write', () => {
    const a = signal(0)
    const b = derived(() => a() + 1)
    const d = derived(() => b() + 1)
    const bSeen: number[] = []
    const dSeen: number[] = []
    effect(() => bSeen.push(b()))
    effect(() => dSeen.push(d()))
    a.set(10)
    expect(bSeen).toEqual([1, 11])
    expect(dSeen).toEqual([2, 12])
  })
})

// ── 15. Disposal during a flush ───────────────────────────────────────────────
describe('reactivity/dispose-during-flush', () => {
  it('an effect disposed by another effect in the same cycle does not run', () => {
    const a = signal(0)
    const bRan: number[] = []
    let disposeB: (() => void) | null = null
    // Disposer subscribes FIRST so it is processed before B in the same flush.
    effect(() => {
      if (a() > 0 && disposeB) disposeB()
    })
    disposeB = effect(() => bRan.push(a()))
    a.set(1)
    expect(bRan).toEqual([0]) // B was torn down before it could re-run
  })
})

// ── 16. Batch value semantics ─────────────────────────────────────────────────
describe('reactivity/batch-semantics', () => {
  it('a read inside a batch sees the written value immediately; effects defer', () => {
    const s = signal(1)
    const seen: number[] = []
    effect(() => seen.push(s()))
    batch(() => {
      s.set(2)
      expect(s()).toBe(2) // value is synchronous
      s.set(3)
    })
    expect(seen).toEqual([1, 3]) // effect saw only the final value, once
  })
})

// ── 17. A throwing derived is contained ───────────────────────────────────────
describe('reactivity/error-containment', () => {
  it('a throwing derived surfaces at the reader and does not wedge the graph', () => {
    const a = signal(1)
    const bad = derived(() => {
      if (a() === 0) throw new Error('boom')
      return a() * 2
    })
    expect(bad()).toBe(2)
    a.set(0)
    expect(() => bad()).toThrow('boom')
    a.set(3) // graph still works after the throw
    expect(bad()).toBe(6)
  })
})

// ── 18. Runaway-loop guard ────────────────────────────────────────────────────
describe('reactivity/runaway-guard', () => {
  it('an unbounded self-scheduling effect is aborted, not hung', () => {
    const spy = vi.spyOn(console, 'error').mockImplementation(() => {})
    const a = signal(0)
    // No fixpoint — writes forever. The flush guard must abort + log.
    effect(() => {
      a.set(a() + 1)
    })
    expect(spy).toHaveBeenCalled()
    const logged = spy.mock.calls.flat().map(String).join(' ')
    expect(logged).toMatch(/aborting flush|scheduling itself|infinite/i)
    spy.mockRestore()
  })
})
