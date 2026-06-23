// Notification reentrance, batch nesting from effects, panic recovery.
// Mirrors crates/sui-signals/tests/reentrance.rs.

import { describe, expect, it, vi } from 'vitest'
import { signal, effect, batch, derived } from '../src/signal'

describe('reentrance', () => {
  it('set inside a subscriber feeds another effect consistently', () => {
    const $a = signal(0)
    const $b = signal(0)
    const log: number[] = []

    effect(() => {
      $b.set($a() * 10)
    })

    effect(() => {
      log.push($b())
    })
    expect(log).toEqual([0]) // initial b read

    $a.set(3)
    // a-effect sets b=30 → b-effect logs 30.
    expect(log).toEqual([0, 30])
  })

  it('effect writing the signal it reads terminates via PartialEq', () => {
    const $s = signal(0)
    let writes = 0
    effect(() => {
      const v = $s()
      if (v < 5) {
        writes += 1
        $s.set(v + 1)
      }
    })
    expect($s()).toBe(5)
    expect(writes).toBe(5)
  })

  it('batch inside an effect coalesces an observer re-runs', () => {
    const $trigger = signal(0)
    const $a = signal(0)
    const $b = signal(0)
    const fn = vi.fn()

    effect(() => {
      $a()
      $b()
      fn()
    })

    effect(() => {
      $trigger()
      batch(() => {
        $a.set($a.__peek() + 1)
        $b.set($b.__peek() + 1)
      })
    })

    const before = fn.mock.calls.length
    $trigger.set(1)
    const after = fn.mock.calls.length
    // Trigger re-ran the writer effect which batched a + b. Observer fires once.
    expect(after - before).toBe(1)
  })

  it('nested batch only flushes at outermost', () => {
    const $s = signal(0)
    const fn = vi.fn()
    effect(() => {
      $s()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)

    batch(() => {
      $s.set(1)
      expect(fn).toHaveBeenCalledTimes(1) // no mid-batch flush
      batch(() => {
        $s.set(2)
        expect(fn).toHaveBeenCalledTimes(1) // no inner-batch flush
      })
      expect(fn).toHaveBeenCalledTimes(1) // still no flush
      $s.set(3)
    })
    expect(fn).toHaveBeenCalledTimes(2)
  })
})

describe('panic recovery', () => {
  it('throwing effect does not leave CURRENT_OBSERVER installed', () => {
    const $s = signal(0)
    const consoleErr = vi.spyOn(console, 'error').mockImplementation(() => {})

    effect(() => {
      $s()
      throw new Error('boom')
    })
    // Effect's eager run threw; effect's try/finally restores observer.

    // Confirm no observer is dangling: a separate effect's tracking works
    // normally and writes to s don't fire the dead effect (its run logs but
    // does nothing on re-run because… well, it WOULD throw again, but the
    // run's try/catch swallows it).
    const $outside = signal(0)
    const fired = vi.fn()
    effect(() => {
      $outside()
      fired()
    })
    expect(fired).toHaveBeenCalledTimes(1)
    consoleErr.mockRestore()
  })

  it('throwing derived recompute does not leave CURRENT_OBSERVER installed', () => {
    const $s = signal(0)
    const $d = derived(() => {
      $s()
      throw new Error('boom')
    })
    // Pull throws.
    expect(() => $d()).toThrow('boom')

    // Subsequent writes to s must not corrupt the reactive graph. A fresh
    // effect should track and re-run normally.
    const fn = vi.fn()
    const $sentinel = signal(0)
    effect(() => {
      $sentinel()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    $sentinel.set(1)
    expect(fn).toHaveBeenCalledTimes(2)

    // Writing s — the derived's invalidate may or may not fire (the
    // subscription is still on s.__subs). What matters: no exception leaks
    // out of the set call to break the test runner.
    expect(() => $s.set(1)).not.toThrow()
  })
})

describe('disposer mid-notification safety', () => {
  it('disposing an effect during its own notification is safe', () => {
    const $s = signal(0)
    const fn = vi.fn()

    let dispose2: (() => void) | null = null
    const e1 = effect(() => {
      const v = $s()
      if (v === 99 && dispose2) dispose2()
    })
    const e2 = effect(() => {
      $s()
      fn()
    })
    dispose2 = e2
    expect(fn).toHaveBeenCalledTimes(1)

    // Trigger the dispose path.
    $s.set(99)
    const afterDispose = fn.mock.calls.length

    // Subsequent writes must not fire e2.
    $s.set(100)
    $s.set(101)
    expect(fn).toHaveBeenCalledTimes(afterDispose)

    e1()
  })
})
