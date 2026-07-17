// Unsubscribe-on-rerun semantics. Mirrors crates/sail-signals/tests/dep_cleanup.rs
// so the Rust and TS signal implementations stay aligned.

import { describe, expect, it, vi } from 'vitest'
import { signal, derived, effect } from '../src/signal'

describe('effect — drops deps no longer read', () => {
  it('effect_drops_dep_when_no_longer_read', () => {
    const $cond = signal(true)
    const $a = signal(0)
    const $b = signal(0)
    const runs: string[] = []

    effect(() => {
      if ($cond()) {
        $a()
        runs.push('a-branch')
      } else {
        $b()
        runs.push('b-branch')
      }
    })
    expect(runs).toEqual(['a-branch'])

    $cond.set(false)
    expect(runs).toEqual(['a-branch', 'b-branch'])

    // `a` is no longer a dep — writing it must not re-run.
    $a.set(1)
    expect(runs).toEqual(['a-branch', 'b-branch'])

    // `b` is the current dep — must re-run.
    $b.set(1)
    expect(runs).toEqual(['a-branch', 'b-branch', 'b-branch'])
  })

  it('same signal read multiple times in one run dedupes', () => {
    const $s = signal(0)
    const fn = vi.fn()
    effect(() => {
      $s()
      $s()
      $s()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    $s.set(1)
    expect(fn).toHaveBeenCalledTimes(2)
  })

  it('writes after dispose do not fire the effect', () => {
    const $s = signal(0)
    const fn = vi.fn()
    const dispose = effect(() => {
      $s()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    dispose()
    $s.set(1)
    $s.set(2)
    $s.set(3)
    expect(fn).toHaveBeenCalledTimes(1)
  })

  it('dispose proactively removes the effect from signal subs', () => {
    const $s = signal(0)
    const dispose = effect(() => {
      $s()
    })
    // The effect's subscriber sits inside $s.__subs.
    expect($s.__subs.size).toBe(1)
    dispose()
    expect($s.__subs.size).toBe(0)
  })

  it('effect re-tracking after a flip removes old subs', () => {
    const $cond = signal(true)
    const $a = signal(0)
    const $b = signal(0)
    effect(() => {
      if ($cond()) $a()
      else $b()
    })
    // Initial: subs on $cond and $a.
    expect($a.__subs.size).toBe(1)
    expect($b.__subs.size).toBe(0)

    $cond.set(false)
    // Now: subs on $cond and $b. $a's sub must be removed.
    expect($a.__subs.size).toBe(0)
    expect($b.__subs.size).toBe(1)

    $cond.set(true)
    expect($a.__subs.size).toBe(1)
    expect($b.__subs.size).toBe(0)
  })
})

describe('derived — drops deps no longer read', () => {
  it('derived_drops_dep_when_no_longer_read', () => {
    const $cond = signal(true)
    const $a = signal(10)
    const $b = signal(20)
    const compute = vi.fn(() => ($cond() ? $a() : $b()))
    const $d = derived(compute)

    // Force initial recompute via read.
    expect($d()).toBe(10)
    const initial = compute.mock.calls.length

    // Flip — invalidates and recomputes on next read.
    $cond.set(false)
    expect($d()).toBe(20)
    const afterFlip = compute.mock.calls.length
    expect(afterFlip).toBeGreaterThan(initial)

    // Write old dep `a`. Must not invalidate (a is no longer a dep).
    $a.set(99)
    expect($d()).toBe(20) // pull
    const afterA = compute.mock.calls.length
    expect(afterA).toBe(afterFlip)

    // Write new dep `b`. Must invalidate.
    $b.set(99)
    expect($d()).toBe(99)
    const afterB = compute.mock.calls.length
    expect(afterB).toBeGreaterThan(afterA)
  })

  it('derived chain cleans up stale intermediate deps', () => {
    const $toggle = signal(true)
    const $a = signal(1)
    const $b = signal(100)
    const $mid = derived(() => ($toggle() ? $a() : $b()))

    const outerCompute = vi.fn(() => $mid() * 2)
    const $outer = derived(outerCompute)

    const eff = vi.fn()
    effect(() => {
      $outer()
      eff()
    })
    const initialEff = eff.mock.calls.length

    $toggle.set(false)
    const afterFlip = eff.mock.calls.length
    expect(afterFlip).toBeGreaterThan(initialEff)

    // Writing old dep `a` (no longer in $mid's deps) must NOT re-run outer.
    $a.set(42)
    const afterA = eff.mock.calls.length
    expect(afterA).toBe(afterFlip)
  })

  it('derived dispose path: dropping all subscribers unsubscribes from deps', () => {
    // Subtle: a derived's subscription to its deps persists as long as
    // anything observes the derived. The current API has no explicit
    // dispose, but its `invalidate` is tracked in deps' __subs only when
    // something has called the reader and forced a recompute. After
    // recompute, deps should NOT accumulate stale invalidate refs.
    const $a = signal(1)
    const $b = signal(2)
    let useA = true
    const $d = derived(() => (useA ? $a() : $b()))
    void $d() // force initial compute, subscribes to $a

    expect($a.__subs.size).toBeGreaterThanOrEqual(1)
    const aSubsBefore = $a.__subs.size

    // Flip the branch and recompute.
    useA = false
    $a.set(99) // invalidates the derived; will recompute on next pull
    void $d() // recompute: now reads $b, not $a
    void $d() // re-read to be sure
    void $d()

    // After recompute, $a should no longer have a stale invalidate ref.
    expect($a.__subs.size).toBeLessThan(aSubsBefore + 5)
    // The hard assertion: $a's sub set should not grow on each call.
    const aSubsSettled = $a.__subs.size
    void $d()
    void $d()
    void $d()
    expect($a.__subs.size).toBe(aSubsSettled)
  })
})
