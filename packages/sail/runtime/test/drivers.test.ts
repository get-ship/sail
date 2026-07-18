// Driver tests. The frame loop is motion.dev's, driven by rAF, so these
// use vitest's fake timers to advance it deterministically — no sleeping,
// no flake, and the assertions are about exact progress rather than "did
// SOMETHING happen".

import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'
import { signal, effect } from '../src/signal'
import { createScope, withScope } from '../src/effect-scope'
import { clock, scrollProgress, springValue } from '../src/drivers'
import { timeline } from '../src/timeline'

// Fake timers are installed ONCE for the whole file, not per test.
// motion.dev's frame loop is a module-level singleton that remembers the
// last frame's timestamp and derives `delta` from it. Toggling back to real
// timers between tests rewinds `performance.now()` under it, the next delta
// computes as <= 0, and every subsequent clock silently sits at 0 — which
// looks exactly like "the driver is broken" rather than "the harness is".
beforeAll(() => {
  vi.useFakeTimers()
})
afterAll(() => {
  vi.useRealTimers()
})

/** Advance the rAF-driven frame loop by `ms`, in ~60fps steps. */
function advance(ms: number): void {
  vi.advanceTimersByTime(ms)
}

/** Pretend the OS asked for reduced motion (or didn't). */
function setReducedMotion(reduce: boolean): void {
  vi.stubGlobal('matchMedia', (query: string) => ({
    matches: reduce && query.includes('prefers-reduced-motion'),
    media: query,
    addEventListener() {},
    removeEventListener() {},
  }))
}

describe('clock', () => {
  beforeEach(() => {
    setReducedMotion(false)
  })
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('starts at 0 and advances toward 1 over the duration', () => {
    const p = clock(1000)
    expect(p()).toBe(0)
    advance(500)
    expect(p()).toBeGreaterThan(0.3)
    expect(p()).toBeLessThan(0.7)
    advance(600)
    expect(p()).toBe(1)
  })

  it('stops at 1 and does not overshoot', () => {
    const p = clock(200)
    advance(5000)
    expect(p()).toBe(1)
    expect(p.running()).toBe(false)
  })

  it('pauses and resumes from where it stopped', () => {
    const p = clock(1000)
    advance(400)
    const at = p()
    p.pause()
    advance(2000)
    expect(p()).toBe(at)
    expect(p.running()).toBe(false)
    p.play()
    advance(200)
    expect(p()).toBeGreaterThan(at)
  })

  it('stop() rewinds to the start', () => {
    const p = clock(1000)
    advance(500)
    expect(p()).toBeGreaterThan(0)
    p.stop()
    expect(p()).toBe(0)
    expect(p.running()).toBe(false)
  })

  it('loops forever with loop: true', () => {
    const p = clock(500, { loop: true })
    advance(1400)
    expect(p.running()).toBe(true)
    expect(p()).toBeGreaterThan(0)
    expect(p()).toBeLessThan(1)
    advance(10_000)
    expect(p.running()).toBe(true)
  })

  it('runs a finite iteration count and then holds', () => {
    const p = clock(200, { loop: 3 })
    advance(1000)
    expect(p()).toBe(1)
    expect(p.running()).toBe(false)
  })

  it('runs 1 → 0 with direction: reverse', () => {
    const p = clock(1000, { direction: 'reverse' })
    expect(p()).toBe(1)
    advance(500)
    expect(p()).toBeLessThan(0.7)
    advance(600)
    expect(p()).toBe(0)
  })

  it('ping-pongs with direction: alternate', () => {
    const p = clock(400, { direction: 'alternate', loop: true })
    advance(200)
    const rising = p()
    expect(rising).toBeGreaterThan(0)
    advance(400) // now in the second (descending) iteration
    expect(p()).toBeLessThan(1)
    advance(1600)
    expect(p.running()).toBe(true)
  })

  it('does not start until play() when autoplay is false', () => {
    const p = clock(500, { autoplay: false })
    advance(1000)
    expect(p()).toBe(0)
    expect(p.running()).toBe(false)
    p.play()
    advance(250)
    expect(p()).toBeGreaterThan(0)
  })

  it('refuses a non-positive duration', () => {
    expect(() => clock(0)).toThrow(/positive number/)
    expect(() => clock(-1)).toThrow(/positive number/)
    expect(() => clock(Number.NaN)).toThrow(/positive number/)
  })

  describe('disposal', () => {
    it('stops when the enclosing scope disposes', () => {
      const scope = createScope()
      const p = withScope(scope, () => clock(1000, { loop: true }))
      advance(300)
      expect(p.running()).toBe(true)
      const at = p()

      scope.dispose()
      expect(p.running()).toBe(false)
      advance(5000)
      expect(p()).toBe(at) // frozen — no frame loop still writing
    })

    it('leaves no frame callback behind after disposal', () => {
      const scope = createScope()
      const p = withScope(scope, () => clock(100, { loop: true }))
      advance(50)
      scope.dispose()
      const after = p()
      // A leaked keepAlive callback would keep ticking forever here.
      advance(60_000)
      expect(p()).toBe(after)
    })
  })

  describe('reduced motion', () => {
    it('jumps to the end and never schedules a frame', () => {
      setReducedMotion(true)
      const p = clock(1000)
      expect(p()).toBe(1)
      expect(p.running()).toBe(false)
      advance(5000)
      expect(p()).toBe(1)
    })

    it("jumps to the START of a reverse clock (its settled value is 0)", () => {
      setReducedMotion(true)
      const p = clock(1000, { direction: 'reverse' })
      expect(p()).toBe(0)
      expect(p.running()).toBe(false)
    })

    it('holds at the beginning with reducedMotion: start', () => {
      setReducedMotion(true)
      const p = clock(1000, { reducedMotion: 'start' })
      expect(p()).toBe(0)
      expect(p.running()).toBe(false)
    })

    it('animates anyway with reducedMotion: play', () => {
      setReducedMotion(true)
      const p = clock(1000, { reducedMotion: 'play' })
      expect(p.running()).toBe(true)
      advance(500)
      expect(p()).toBeGreaterThan(0)
      expect(p()).toBeLessThan(1)
    })

    it('still exposes transport controls so a caller can opt in', () => {
      setReducedMotion(true)
      const p = clock(1000)
      p.stop()
      p.play()
      expect(p.running()).toBe(true)
    })
  })

  it('drives a timeline end to end', () => {
    const p = clock(1000)
    const { x } = timeline(p, { x: [[0, '0px'], [0.5, '100px'], [1, '100px']] })
    expect(x()).toBe('0px')
    advance(1100)
    expect(x()).toBe('100px')
  })
})

describe('scrollProgress', () => {
  beforeEach(() => {
    setReducedMotion(false)
  })
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  function setDocumentScroll(scrollY: number, scrollHeight: number, clientHeight: number): void {
    Object.defineProperty(document.documentElement, 'scrollHeight', {
      value: scrollHeight,
      configurable: true,
    })
    Object.defineProperty(document.documentElement, 'clientHeight', {
      value: clientHeight,
      configurable: true,
    })
    Object.defineProperty(window, 'scrollY', { value: scrollY, configurable: true })
  }

  it('reports document scroll as 0..1', () => {
    setDocumentScroll(0, 2000, 1000)
    const p = scrollProgress()
    expect(p()).toBe(0)

    setDocumentScroll(500, 2000, 1000)
    window.dispatchEvent(new Event('scroll'))
    advance(20)
    expect(p()).toBeCloseTo(0.5, 6)

    setDocumentScroll(1000, 2000, 1000)
    window.dispatchEvent(new Event('scroll'))
    advance(20)
    expect(p()).toBe(1)
  })

  it('reports 0 for an unscrollable document rather than NaN', () => {
    setDocumentScroll(0, 800, 800)
    const p = scrollProgress()
    expect(p()).toBe(0)
    expect(Number.isNaN(p())).toBe(false)
  })

  it("measures a target element's travel through the viewport", () => {
    Object.defineProperty(window, 'innerHeight', { value: 1000, configurable: true })
    const el = document.createElement('div')
    document.body.appendChild(el)

    let top = 1000 // leading edge exactly at the viewport bottom → 0
    el.getBoundingClientRect = () =>
      ({ top, left: 0, height: 200, width: 100 }) as DOMRect

    const p = scrollProgress(el)
    expect(p()).toBe(0)

    top = -200 // trailing edge exactly at the viewport top → 1
    window.dispatchEvent(new Event('scroll'))
    advance(20)
    expect(p()).toBe(1)

    top = 400 // partway through
    window.dispatchEvent(new Event('scroll'))
    advance(20)
    expect(p()).toBeCloseTo(0.5, 6)

    el.remove()
  })

  it('coalesces a burst of scroll events into one measurement per frame', () => {
    Object.defineProperty(window, 'innerHeight', { value: 1000, configurable: true })
    const el = document.createElement('div')
    document.body.appendChild(el)
    const measured = vi.fn()
    el.getBoundingClientRect = () => {
      measured()
      return { top: 500, left: 0, height: 200, width: 100 } as DOMRect
    }

    const p = scrollProgress(el)
    measured.mockClear()
    for (let i = 0; i < 50; i++) window.dispatchEvent(new Event('scroll'))
    advance(20)
    expect(measured).toHaveBeenCalledTimes(1)
    expect(p()).toBeGreaterThan(0)
    el.remove()
  })

  it('detaches its listeners when the enclosing scope disposes', () => {
    setDocumentScroll(0, 2000, 1000)
    const scope = createScope()
    const p = withScope(scope, () => scrollProgress())
    scope.dispose()

    setDocumentScroll(1000, 2000, 1000)
    window.dispatchEvent(new Event('scroll'))
    advance(50)
    expect(p()).toBe(0) // never updated — the listener is gone
  })
})

describe('springValue', () => {
  beforeEach(() => {
    setReducedMotion(false)
  })
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('chases its source rather than jumping to it', () => {
    const target = signal(0)
    const y = springValue(target, { stiffness: 200, damping: 20 })
    expect(y()).toBe(0)

    target.set(100)
    // A few frames in, the spring has moved but is nowhere near the target.
    // (The first frame after a retarget seeds the integrator; motion.dev
    // reports the value from the frame AFTER that.)
    advance(50)
    expect(y()).toBeGreaterThan(0)
    expect(y()).toBeLessThan(100)
  })

  it('settles on the target', () => {
    const target = signal(0)
    const y = springValue(target, { stiffness: 200, damping: 20 })
    target.set(100)
    advance(4000)
    expect(y()).toBeCloseTo(100, 1)
  })

  // The "running but invisible" guard: a spring that ticks without moving
  // passes a settle assertion trivially if it starts at the target.
  it('visibly travels — it visits many distinct values on the way', () => {
    const target = signal(0)
    const y = springValue(target, { stiffness: 120, damping: 12 })
    const seen = new Set<number>()
    target.set(200)
    for (let i = 0; i < 120; i++) {
      advance(16)
      seen.add(y())
    }
    expect(seen.size).toBeGreaterThan(20)
    expect(Math.max(...seen)).toBeGreaterThan(50)
  })

  it('overshoots with a bouncy spring — real physics, not a tween', () => {
    const target = signal(0)
    const y = springValue(target, { stiffness: 400, damping: 6 })
    target.set(100)
    let peak = 0
    for (let i = 0; i < 200; i++) {
      advance(16)
      peak = Math.max(peak, y())
    }
    expect(peak).toBeGreaterThan(100)
  })

  it('retargets mid-flight without restarting', () => {
    const target = signal(0)
    const y = springValue(target, { stiffness: 200, damping: 20 })
    target.set(100)
    advance(100)
    const mid = y()
    expect(mid).toBeGreaterThan(0)

    target.set(0)
    advance(2000)
    expect(y()).toBeCloseTo(0, 1)
  })

  it('mirrors the source instantly under reduced motion', () => {
    setReducedMotion(true)
    const target = signal(0)
    const y = springValue(target)
    target.set(100)
    expect(y()).toBe(100) // no frames needed
  })

  it('releases the spring when the enclosing scope disposes', () => {
    const target = signal(0)
    const scope = createScope()
    const y = withScope(scope, () => springValue(target, { stiffness: 200, damping: 20 }))
    target.set(100)
    advance(50)
    const at = y()
    expect(at).toBeGreaterThan(0)

    scope.dispose()
    advance(5000)
    expect(y()).toBe(at) // frozen — the spring is gone
  })

  it('feeds a timeline like any other progress signal', () => {
    const target = signal(0)
    const p = springValue(target, { stiffness: 300, damping: 30 })
    const { size } = timeline(p, { size: [[0, '0px'], [1, '50px']] })
    expect(size()).toBe('0px')
    target.set(1)
    advance(3000)
    expect(Number(String(size()).replace('px', ''))).toBeCloseTo(50, 0)
  })
})

describe('drivers — interchangeability', () => {
  beforeEach(() => {
    setReducedMotion(false)
  })
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('a hand-written signal drives a timeline identically to a clock', () => {
    const tracks = { x: [[0, '0px'], [0.5, '80px'], [1, '0px']] } as const

    const manual = signal(0)
    const fromSignal = timeline(manual, tracks)

    const driven = clock(1000, { autoplay: false })
    const fromClock = timeline(driven, tracks)

    for (const at of [0, 0.25, 0.5, 0.75, 1]) {
      manual.set(at)
      driven.set(at)
      expect(fromClock.x()).toBe(fromSignal.x())
    }
  })

  it('reflects through an effect the way the compiler-emitted binding does', () => {
    // This is the shape the compiler emits for `sx="translate-x-(--tile-x)"`.
    const el = document.createElement('div')
    const p = clock(1000, { autoplay: false })
    const { tileX } = timeline(p, { tileX: [[0, '0px'], [1, '120px']] })
    effect(() => el.style.setProperty('--tile-x', String(tileX())))

    expect(el.style.getPropertyValue('--tile-x')).toBe('0px')
    p.set(0.5)
    expect(el.style.getPropertyValue('--tile-x')).toBe('60px')
    p.set(1)
    expect(el.style.getPropertyValue('--tile-x')).toBe('120px')
  })
})
