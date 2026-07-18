// The gauntlet: recreate motion.dev's tile choreography with `timeline()`.
//
// The tile is the hard case on purpose. Seven stops, three channels sharing
// one clock, a translate that settles at .34 and HOLDS through .44, and a
// scale that pops .54 → .64 while everything else stays put. Hand-rolled
// `Math.min/Math.max(v - a) / b` progress math gets this *nearly* right,
// which is worse than getting it wrong — the tile drifts a few pixels off
// its hold and nothing tells you.
//
// The ground truth is a verbatim capture off the live page (see
// `fixtures/motion-capture/README.md`). The ORACLE this test compares
// against is written here, independently: its own cubic-bezier solver
// (bisection, not motion's Newton–Raphson) walking the JSON keyframes
// directly. Where the two agree across 400 sample points, the interpolation
// is right for reasons neither implementation shares.
//
// The browser half of the proof — the same capture replayed through real
// WAAPI in headless Chromium — is `e2e/timeline-tile.spec.ts`.

import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { signal } from '../src/signal'
import { timeline } from '../src/timeline'

// ── The capture ──────────────────────────────────────────────────────────────

interface CapturedKeyframe {
  offset: number
  easing: string
  transform: string
}
interface CapturedAnimation {
  target: string
  duration: number
  keyframes: CapturedKeyframe[]
}

const CAPTURE: CapturedAnimation[] = JSON.parse(
  readFileSync(
    resolve(
      dirname(fileURLToPath(import.meta.url)),
      './fixtures/motion-capture/waapi-animations.json',
    ),
    'utf8',
  ),
)

const TILE = CAPTURE.find((a) => a.target === 'span.home-diag-tile')!

/** `cubic-bezier(0.6, 0, 0.2, 1)` → `[0.6, 0, 0.2, 1]`. */
function parseBezier(easing: string): [number, number, number, number] {
  const m = /^cubic-bezier\(([^)]+)\)$/.exec(easing)
  if (!m) throw new Error(`unexpected easing in capture: ${easing}`)
  const n = m[1]!.split(',').map((s) => Number(s.trim()))
  return [n[0]!, n[1]!, n[2]!, n[3]!]
}

/** `translate(120px) rotate(15deg) scale(1.75)` → the three channels. */
function parseTransform(t: string): { x: number; rotate: number; scale: number } {
  const num = (fn: string): number => {
    const m = new RegExp(`${fn}\\(([-\\d.]+)`).exec(t)
    if (!m) throw new Error(`no ${fn}() in captured transform: ${t}`)
    return Number(m[1])
  }
  return { x: num('translate'), rotate: num('rotate'), scale: num('scale') }
}

const STOPS = TILE.keyframes.map((kf) => ({
  offset: kf.offset,
  easing: parseBezier(kf.easing),
  ...parseTransform(kf.transform),
}))

// ── The independent oracle ───────────────────────────────────────────────────

/**
 * Solve a unit cubic bezier by bisection — deliberately a different method
 * from the Newton–Raphson solver `timeline()` uses via motion.dev, so
 * agreement is evidence rather than a shared bug.
 */
function bezier(x1: number, y1: number, x2: number, y2: number): (x: number) => number {
  const curve = (a: number, b: number, t: number): number => {
    const u = 1 - t
    return 3 * u * u * t * a + 3 * u * t * t * b + t * t * t
  }
  return (x) => {
    if (x <= 0) return 0
    if (x >= 1) return 1
    let lo = 0
    let hi = 1
    for (let i = 0; i < 80; i++) {
      const mid = (lo + hi) / 2
      if (curve(x1, x2, mid) < x) lo = mid
      else hi = mid
    }
    return curve(y1, y2, (lo + hi) / 2)
  }
}

/** Sample the captured keyframes directly, walking segments by hand. */
function oracle(channel: 'x' | 'rotate' | 'scale', p: number): number {
  if (p <= STOPS[0]!.offset) return STOPS[0]![channel]
  const last = STOPS[STOPS.length - 1]!
  if (p >= last.offset) return last[channel]
  for (let i = 0; i < STOPS.length - 1; i++) {
    const a = STOPS[i]!
    const b = STOPS[i + 1]!
    if (p < a.offset || p > b.offset) continue
    const span = b.offset - a.offset
    const local = span === 0 ? 0 : (p - a.offset) / span
    const eased = bezier(...a.easing)(local)
    return a[channel] + (b[channel] - a[channel]) * eased
  }
  throw new Error(`unreachable: no segment for ${p}`)
}

// ── The recreation ───────────────────────────────────────────────────────────

const progress = signal(0)

// This is the whole choreography. Keyframes as data, straight off the
// capture — no arithmetic, no offset bookkeeping, no `el.style`.
const tile = timeline(
  progress,
  {
    tileX: STOPS.map((s) => [s.offset, `${s.x}px`] as const),
    tileRotate: STOPS.map((s) => [s.offset, `${s.rotate}deg`] as const),
    tileScale: STOPS.map((s) => [s.offset, s.scale] as const),
  },
  { easing: [0.6, 0, 0.2, 1] },
)

const px = (v: string | number): number => Number(String(v).replace(/[a-z%]+$/i, ''))

describe('acceptance — motion.dev tile, 7-stop compound choreography', () => {
  it('reads its shape from the capture, not from this test', () => {
    expect(TILE.duration).toBe(3600)
    expect(STOPS.map((s) => s.offset)).toEqual([0, 0.12, 0.34, 0.44, 0.54, 0.64, 1])
    expect(STOPS.every((s) => s.easing.join() === '0.6,0,0.2,1')).toBe(true)
  })

  // Tolerance is not a fudge factor — it is the measured gap between the two
  // bezier solvers. motion.dev's `cubicBezier` (Newton–Raphson over a sampled
  // table) differs from the oracle's 80-round bisection by at most
  // 3.66e-4 in eased PROGRESS, measured across 100k points on this exact
  // curve. Scaled onto each channel's span that is 0.044px of travel,
  // 0.0055deg of rotation, and 2.8e-4 of scale — three orders of magnitude
  // below anything a display can resolve. The bounds below are those numbers
  // rounded up, so a REAL interpolation bug (a missed offset, a mis-assigned
  // easing, an off-by-one segment) blows through them immediately.
  const SOLVER_EPS = 3.66e-4

  it('matches the oracle on every channel across 400 sample points', () => {
    for (let i = 0; i <= 400; i++) {
      const p = i / 400
      progress.set(p)
      expect(Math.abs(px(tile.tileX()) - oracle('x', p))).toBeLessThan(120 * SOLVER_EPS)
      expect(Math.abs(px(tile.tileRotate()) - oracle('rotate', p))).toBeLessThan(15 * SOLVER_EPS)
      expect(Math.abs((tile.tileScale() as number) - oracle('scale', p))).toBeLessThan(
        0.75 * SOLVER_EPS,
      )
    }
  })

  it('lands exactly on every captured stop', () => {
    for (const s of STOPS) {
      progress.set(s.offset)
      expect(px(tile.tileX())).toBeCloseTo(s.x, 9)
      expect(px(tile.tileRotate())).toBeCloseTo(s.rotate, 9)
      expect(tile.tileScale() as number).toBeCloseTo(s.scale, 9)
    }
  })

  it('settles the translate at .34 and HOLDS it through .44', () => {
    for (const p of [0.34, 0.36, 0.38, 0.4, 0.42, 0.44]) {
      progress.set(p)
      expect(tile.tileX()).toBe('120px')
      expect(tile.tileRotate()).toBe('15deg')
    }
    // The hold is a hold, not a plateau the easing happens to flatten:
    // just before it the tile is still travelling.
    progress.set(0.33)
    expect(px(tile.tileX())).toBeLessThan(120)
    expect(px(tile.tileX())).toBeGreaterThan(0)
  })

  it('pops the scale .54 → .64 while translate and rotate stay parked', () => {
    progress.set(0.44)
    expect(tile.tileScale()).toBe(1)
    progress.set(0.54)
    expect(tile.tileScale()).toBe(1.75)
    progress.set(0.64)
    expect(tile.tileScale()).toBe(1)

    // Throughout the pop, the other two channels do not move.
    for (const p of [0.46, 0.5, 0.54, 0.58, 0.62]) {
      progress.set(p)
      expect(tile.tileX()).toBe('120px')
      expect(tile.tileRotate()).toBe('15deg')
    }
  })

  it('holds the opening beat through .12 and returns home at 1', () => {
    for (const p of [0, 0.05, 0.12]) {
      progress.set(p)
      expect(tile.tileX()).toBe('0px')
      expect(tile.tileScale()).toBe(1)
    }
    progress.set(1)
    expect(tile.tileX()).toBe('0px')
    expect(tile.tileRotate()).toBe('0deg')
    expect(tile.tileScale()).toBe(1)
  })

  // The "running but invisible" guard. An animation that technically ticks
  // but never moves anything passes every structural assertion above if the
  // values are all equal; these assert the motion has real amplitude.
  it('actually moves — every channel spans its full captured range', () => {
    const seen = { x: new Set<number>(), rotate: new Set<number>(), scale: new Set<number>() }
    // 400 steps so the grid lands exactly on every captured stop
    // (.12, .34, .44, .54, .64 are all integer multiples of 1/400).
    for (let i = 0; i <= 400; i++) {
      progress.set(i / 400)
      seen.x.add(px(tile.tileX()))
      seen.rotate.add(px(tile.tileRotate()))
      seen.scale.add(tile.tileScale() as number)
    }
    expect(Math.min(...seen.x)).toBe(0)
    expect(Math.max(...seen.x)).toBe(120)
    expect(Math.max(...seen.rotate)).toBe(15)
    expect(Math.max(...seen.scale)).toBe(1.75)
    // Not a two-state flicker: a real curve visits many distinct values.
    expect(seen.x.size).toBeGreaterThan(100)
    expect(seen.scale.size).toBeGreaterThan(40)
  })
})

describe('acceptance — the rest of the capture recreates too', () => {
  it('expresses the row exit/return (translate + opacity, 6 stops)', () => {
    const row = CAPTURE.find((a) => a.target === 'span.home-diag-row')!
    const p = signal(0)
    const t = timeline(
      p,
      {
        rowX: row.keyframes.map(
          (kf) => [kf.offset, /translate\(([-\d.]+)/.exec(kf.transform)![1] + 'px'] as const,
        ),
        rowO: row.keyframes.map(
          (kf) => [kf.offset, Number((kf as unknown as { opacity: string }).opacity)] as const,
        ),
      },
      { easing: [0.4, 0, 0.2, 1] },
    )
    p.set(0.3)
    expect(t.rowO()).toBe(1)
    p.set(0.55)
    expect(t.rowO()).toBe(0)
    p.set(0.7)
    expect(t.rowO()).toBe(0) // held out
    p.set(0.85)
    expect(t.rowO()).toBe(1) // and back
  })

  it('expresses the bar fill (scaleX sweep, 4 stops)', () => {
    const bar = CAPTURE.find((a) => a.target === 'span.home-diag-bar-fill')!
    const p = signal(0)
    const { barScale } = timeline(
      p,
      {
        barScale: bar.keyframes.map(
          (kf) => [kf.offset, Number(/scaleX\(([-\d.]+)/.exec(kf.transform)![1])] as const,
        ),
      },
      { easing: [0.4, 0, 0.4, 1] },
    )
    p.set(0)
    expect(barScale()).toBe(0)
    p.set(0.45)
    expect(barScale()).toBe(1)
    p.set(0.9)
    expect(barScale()).toBe(0)
    p.set(1)
    expect(barScale()).toBe(0)
  })
})
