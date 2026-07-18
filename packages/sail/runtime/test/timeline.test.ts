import { describe, expect, it, vi } from 'vitest'
import { signal, effect } from '../src/signal'
import { timeline, SailTimelineError } from '../src/timeline'

describe('timeline — offsets', () => {
  it('interpolates linearly between stops', () => {
    const p = signal(0)
    const { x } = timeline(p, { x: [[0, 0], [1, 100]] })
    expect(x()).toBe(0)
    p.set(0.25)
    expect(x()).toBe(25)
    p.set(0.5)
    expect(x()).toBe(50)
    p.set(1)
    expect(x()).toBe(100)
  })

  it('honours intermediate offsets rather than spreading stops evenly', () => {
    const p = signal(0)
    // Evenly spread, the midpoint would be 50. With the stop at .8 it is 25.
    const { x } = timeline(p, { x: [[0, 0], [0.8, 40], [1, 100]] })
    p.set(0.4)
    expect(x()).toBeCloseTo(20, 10)
    p.set(0.8)
    expect(x()).toBeCloseTo(40, 10)
    p.set(0.9)
    expect(x()).toBeCloseTo(70, 10)
  })

  it('clamps outside the first and last offsets', () => {
    const p = signal(0)
    const { x } = timeline(p, { x: [[0.2, 10], [0.8, 90]] })
    p.set(0)
    expect(x()).toBe(10)
    p.set(-5)
    expect(x()).toBe(10)
    p.set(1)
    expect(x()).toBe(90)
    p.set(99)
    expect(x()).toBe(90)
  })

  it('treats a single-stop track as a constant', () => {
    const p = signal(0)
    const { x } = timeline(p, { x: [[0.5, 7]] })
    p.set(0)
    expect(x()).toBe(7)
    p.set(1)
    expect(x()).toBe(7)
  })
})

describe('timeline — holds', () => {
  it('holds a repeated value across the whole segment', () => {
    const p = signal(0)
    const { x } = timeline(p, {
      x: [[0, 0], [0.34, 120], [0.44, 120], [1, 0]],
    })
    p.set(0.34)
    expect(x()).toBeCloseTo(120, 10)
    for (const at of [0.36, 0.38, 0.4, 0.42, 0.44]) {
      p.set(at)
      expect(x()).toBeCloseTo(120, 10)
    }
    // …and moves again immediately after the hold ends.
    p.set(0.5)
    expect(x()).toBeLessThan(120)
  })

  it('notifies nothing while a hold is in progress', () => {
    const p = signal(0.34)
    const { x } = timeline(p, { x: [[0, 0], [0.34, 120], [0.44, 120], [1, 0]] })

    const seen = vi.fn()
    effect(() => seen(x()))
    expect(seen).toHaveBeenCalledTimes(1)

    // Every one of these recomputes to 120 — Object.is-equal, so the
    // downstream effect (a custom-property write, in production) is skipped.
    for (const at of [0.36, 0.39, 0.41, 0.44]) p.set(at)
    expect(seen).toHaveBeenCalledTimes(1)

    p.set(0.5)
    expect(seen).toHaveBeenCalledTimes(2)
  })

  it('rejects a repeated OFFSET, which is the typo for a hold', () => {
    const p = signal(0)
    expect(() =>
      timeline(p, { x: [[0, 0], [0.5, 1], [0.5, 2], [1, 3]] }),
    ).toThrow(/offsets must ascend/)
  })
})

describe('timeline — easing', () => {
  it('defaults to linear', () => {
    const p = signal(0.5)
    const { x } = timeline(p, { x: [[0, 0], [1, 100]] })
    expect(x()).toBeCloseTo(50, 10)
  })

  it('applies a timeline-wide default easing to every segment', () => {
    const p = signal(0.5)
    const eased = timeline(p, { x: [[0, 0], [1, 100]] }, { easing: [0.6, 0, 0.2, 1] })
    // cubic-bezier(0.6, 0, 0.2, 1) at t=.5 is its own midpoint, 0.5 —
    // symmetric — so probe an asymmetric point instead.
    p.set(0.25)
    expect(eased.x()).toBeLessThan(25)
    p.set(0.75)
    expect(eased.x()).toBeGreaterThan(75)
  })

  it('applies a per-segment easing only to the segment it starts', () => {
    const p = signal(0)
    const { x } = timeline(p, {
      // Segment 0 (0 → .5) linear; segment 1 (.5 → 1) heavily eased.
      x: [[0, 0], [0.5, 100, { easing: [0.9, 0, 1, 0.1] }], [1, 200]],
    })
    p.set(0.25)
    expect(x()).toBeCloseTo(50, 10) // untouched by the later segment's easing
    p.set(0.75)
    expect(x()).toBeLessThan(140) // the ease holds it back mid-segment
    p.set(1)
    expect(x()).toBeCloseTo(200, 10)
  })

  it("ignores the last stop's easing (it starts no segment)", () => {
    const p = signal(0.5)
    const a = timeline(p, { x: [[0, 0], [1, 100]] })
    const b = timeline(p, { x: [[0, 0], [1, 100, { easing: [0.9, 0, 1, 0.1] }]] })
    expect(a.x()).toBeCloseTo(b.x() as number, 10)
  })

  it('accepts named easings and raw functions', () => {
    const p = signal(0.5)
    const named = timeline(p, { x: [[0, 0], [1, 100]] }, { easing: 'ease-in' })
    expect(named.x()).toBeLessThan(50)

    const fn = timeline(p, { x: [[0, 0], [1, 100]] }, { easing: (t) => t * t })
    expect(fn.x()).toBeCloseTo(25, 10)
  })

  it('refuses an unknown easing name', () => {
    const p = signal(0)
    expect(() =>
      // @ts-expect-error — the point of the test is the runtime refusal.
      timeline(p, { x: [[0, 0], [1, 1]] }, { easing: 'ease-in-oot' }),
    ).toThrow(/unknown easing/)
  })
})

describe('timeline — units', () => {
  it('interpolates px and keeps the unit', () => {
    const p = signal(0)
    const { y } = timeline(p, { y: [[0, '40px'], [1, '0px']] })
    expect(y()).toBe('40px')
    p.set(0.5)
    expect(y()).toBe('20px')
  })

  it('interpolates percentages, degrees, and negative values', () => {
    const p = signal(0.5)
    const t = timeline(p, {
      pct: [[0, '0%'], [1, '-12%']],
      deg: [[0, '0deg'], [1, '15deg']],
    })
    expect(t.pct()).toBe('-6%')
    expect(t.deg()).toBe('7.5deg')
  })

  it('keeps numbers as numbers', () => {
    const p = signal(0.5)
    const { s } = timeline(p, { s: [[0, 1], [1, 1.75]] })
    expect(s()).toBe(1.375)
    expect(typeof s()).toBe('number')
  })

  it('refuses mixed units in one track', () => {
    const p = signal(0)
    expect(() => timeline(p, { x: [[0, '0px'], [1, '50%']] })).toThrow(
      /does not match the track's/,
    )
  })

  it('refuses a number mixed with a unit string', () => {
    const p = signal(0)
    expect(() => timeline(p, { x: [[0, 0], [1, '50px']] })).toThrow(
      /does not match the track's/,
    )
  })

  it('refuses a multi-part value rather than blending it', () => {
    const p = signal(0)
    expect(() => timeline(p, { shadow: [[0, '0 0 4px red'], [1, '0 0 8px blue']] })).toThrow(
      /not an interpolable value/,
    )
  })
})

describe('timeline — validation', () => {
  const p = signal(0)

  it('refuses a non-function progress', () => {
    // @ts-expect-error — runtime refusal is the contract under test.
    expect(() => timeline(0.5, { x: [[0, 0], [1, 1]] })).toThrow(SailTimelineError)
  })

  it('refuses an empty track', () => {
    expect(() => timeline(p, { x: [] })).toThrow(/at least one stop/)
  })

  it('refuses offsets outside 0..1', () => {
    expect(() => timeline(p, { x: [[0, 0], [1.5, 1]] })).toThrow(/offset must be a number in 0\.\.1/)
    expect(() => timeline(p, { x: [[-0.5, 0], [1, 1]] })).toThrow(
      /offset must be a number in 0\.\.1/,
    )
  })

  it('refuses a malformed stop', () => {
    // @ts-expect-error — runtime refusal is the contract under test.
    expect(() => timeline(p, { x: [[0]] })).toThrow(/expected \[offset, value\]/)
  })

  it('names the offending track and stop', () => {
    expect(() => timeline(p, { tileScale: [[0, 0], [0.2, '5px']] })).toThrow(
      /timeline track 'tileScale' stop 1/,
    )
  })
})

describe('timeline — laziness', () => {
  it('does not evaluate a track nobody reads', () => {
    const reads = vi.fn()
    const p = (() => {
      reads()
      return 0.5
    }) as unknown as () => number

    timeline(p, { x: [[0, 0], [1, 1]] })
    // Construction validates the DATA; it never samples the progress.
    expect(reads).not.toHaveBeenCalled()
  })

  it('samples once per read regardless of track count', () => {
    let reads = 0
    const value = signal(0.5)
    const p = () => {
      reads++
      return value()
    }
    const t = timeline(p, { a: [[0, 0], [1, 1]], b: [[0, 0], [1, 2]], c: [[0, 0], [1, 3]] })
    reads = 0
    t.a()
    expect(reads).toBe(1)
    // Memoized — a second read of an unchanged progress re-samples nothing.
    t.a()
    expect(reads).toBe(1)
  })
})
