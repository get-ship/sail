// motion runtime — the view/scroll trigger contract.
//
// Regression guard for the second silent death of `motion:view` (the first
// was #69's `x`/`y` keyframe defect). The motion.dev migration rewrote the
// view/scroll branch as a bare `createAnimations()` — identical to `enter`
// — so every `motion:view` reveal played ONCE at mount, below the fold,
// invisibly, and the element sat at its final pose forever. No error, no
// warning: a finished animation is indistinguishable from a correct one.
//
// The contract these tests pin, at the `animate()`/`attachTimeline`
// boundary (mocked):
//   1. A view/scroll animation is NEVER left time-driven. The runtime must
//      hand it a progress timeline (`ViewTimeline`/`ScrollTimeline`) via
//      the driver's `attachTimeline` seam the moment it is created.
//   2. `view` scopes the reveal to the element's ENTRY into the viewport
//      (`entry 0% → entry 100%`), so a card is fully arrived once fully
//      visible.
//   3. No timeline support → `view` degrades to play-once-on-first-reveal
//      via IntersectionObserver (paused until intersection), NOT to
//      play-at-mount and NOT to invisible-forever.
//   4. No timeline AND no IntersectionObserver → settle at the final pose
//      and console.warn. Silent no-ops are the disease; the degraded path
//      is loud by policy.
//   5. `loop`/`enter` never touch the timeline seam (no regression of the
//      working triggers).

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

interface MockControl {
  stop: () => void
  cancel: () => void
  pause: ReturnType<typeof vi.fn>
  play: ReturnType<typeof vi.fn>
  complete: ReturnType<typeof vi.fn>
  attachTimeline: ReturnType<typeof vi.fn>
}

const { animateCalls, controlsMade } = vi.hoisted(() => ({
  animateCalls: [] as Array<{ el: unknown; keyframes: unknown; transition: unknown }>,
  controlsMade: [] as MockControl[],
}))

vi.mock('motion/mini', () => ({
  animate: (el: unknown, keyframes: unknown, transition: unknown) => {
    animateCalls.push({ el, keyframes, transition })
    const c: MockControl = {
      stop: () => {},
      cancel: () => {},
      pause: vi.fn(),
      play: vi.fn(),
      complete: vi.fn(),
      // Mirrors motion-dom's NativeAnimation.attachTimeline: with a
      // timeline, re-home and return a detacher; without one, hand the
      // animation to the caller's `observe` fallback.
      attachTimeline: vi.fn((opts: { timeline?: unknown; observe: (a: unknown) => () => void }) =>
        opts.timeline ? () => {} : opts.observe(c),
      ),
    }
    controlsMade.push(c)
    return c
  },
}))

import { __sui_motion_attach } from '../src/motion'
import type { MotionAttachConfig } from '../src/motion'

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  configurable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => {},
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => false,
  }),
})

const viewFade: MotionAttachConfig = {
  trigger: 'reveal',
  motions: [{ property: 'opacity', preset: 'fade', kind: 'tween', from: 0, to: 1, duration: 0.12 }],
}

const globals = globalThis as {
  ViewTimeline?: unknown
  ScrollTimeline?: unknown
  IntersectionObserver?: unknown
}

class FakeViewTimeline {
  subject: Element
  constructor(opts: { subject: Element }) { this.subject = opts.subject }
}
class FakeScrollTimeline {
  source: Element | null
  constructor(opts: { source: Element | null }) { this.source = opts.source }
}

/** The IO stub records observers so a test can synthesize an intersection. */
let ioInstances: Array<{
  el: Element | null
  fire: (isIntersecting: boolean) => void
  disconnected: boolean
}> = []
class FakeIntersectionObserver {
  private cb: (entries: Array<{ isIntersecting: boolean }>) => void
  private record: { el: Element | null; fire: (i: boolean) => void; disconnected: boolean }
  constructor(cb: (entries: Array<{ isIntersecting: boolean }>) => void) {
    this.cb = cb
    this.record = { el: null, fire: (i: boolean) => this.cb([{ isIntersecting: i }]), disconnected: false }
    ioInstances.push(this.record)
  }
  observe(el: Element): void { this.record.el = el }
  disconnect(): void { this.record.disconnected = true }
}

beforeEach(() => {
  ioInstances = []
})

afterEach(() => {
  animateCalls.length = 0
  controlsMade.length = 0
  delete globals.ViewTimeline
  delete globals.ScrollTimeline
  delete globals.IntersectionObserver
  document.body.innerHTML = ''
  vi.restoreAllMocks()
})

function attachConnected(config: MotionAttachConfig): { el: HTMLElement; dispose: () => void } {
  const el = document.createElement('div')
  document.body.appendChild(el)
  const dispose = __sui_motion_attach(el, config)
  return { el, dispose }
}

describe('motion:reveal — re-timed onto a ViewTimeline when the platform has one', () => {
  it('hands every created animation a ViewTimeline scoped to the entry range', () => {
    globals.ViewTimeline = FakeViewTimeline
    const { el, dispose } = attachConnected(viewFade)
    expect(controlsMade.length).toBe(1)
    const call = controlsMade[0]!.attachTimeline.mock.calls[0]![0] as {
      timeline: FakeViewTimeline
      rangeStart?: string
      rangeEnd?: string
    }
    expect(call.timeline).toBeInstanceOf(FakeViewTimeline)
    expect(call.timeline.subject).toBe(el) // the element's OWN viewport passage
    expect(call.rangeStart).toBe('entry 0%')
    expect(call.rangeEnd).toBe('entry 100%')
    dispose()
  })

  it('motion:scroll gets a ScrollTimeline on the document scroller, unranged', () => {
    globals.ScrollTimeline = FakeScrollTimeline
    const { dispose } = attachConnected({ ...viewFade, trigger: 'scroll' })
    const call = controlsMade[0]!.attachTimeline.mock.calls[0]![0] as {
      timeline: FakeScrollTimeline
      rangeStart?: string
    }
    expect(call.timeline).toBeInstanceOf(FakeScrollTimeline)
    expect(call.rangeStart).toBeUndefined()
    dispose()
  })
})

describe('motion:reveal — no timeline support degrades to play-on-first-reveal', () => {
  it('pauses at creation and plays on the first intersection, then stops observing', () => {
    globals.IntersectionObserver = FakeIntersectionObserver
    const { el, dispose } = attachConnected(viewFade)
    const c = controlsMade[0]!
    expect(c.pause).toHaveBeenCalledTimes(1) // held at the rest pose, not playing at mount
    expect(c.play).not.toHaveBeenCalled()
    expect(ioInstances.length).toBe(1)
    expect(ioInstances[0]!.el).toBe(el)
    ioInstances[0]!.fire(false) // scrolled near, not in — still held
    expect(c.play).not.toHaveBeenCalled()
    ioInstances[0]!.fire(true) // enters the viewport → the reveal plays
    expect(c.play).toHaveBeenCalledTimes(1)
    expect(ioInstances[0]!.disconnected).toBe(true) // one-shot
    dispose()
  })

  it('disposal before the element is ever revealed disconnects the observer', () => {
    globals.IntersectionObserver = FakeIntersectionObserver
    const { dispose } = attachConnected(viewFade)
    dispose()
    expect(ioInstances[0]!.disconnected).toBe(true)
  })

  it('no timeline AND no IntersectionObserver: settles at the final pose, LOUDLY', () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    const { dispose } = attachConnected(viewFade)
    const c = controlsMade[0]!
    expect(c.complete).toHaveBeenCalledTimes(1) // visible, not invisible-forever
    expect(warn).toHaveBeenCalledTimes(1)
    expect(String(warn.mock.calls[0]![0])).toContain('motion:reveal')
    dispose()
  })
})

describe('the working triggers never touch the timeline seam', () => {
  it('loop and enter do not attach a timeline', () => {
    globals.ViewTimeline = FakeViewTimeline
    globals.ScrollTimeline = FakeScrollTimeline
    for (const trigger of ['loop', 'enter']) {
      const { dispose } = attachConnected({ ...viewFade, trigger })
      expect(controlsMade.at(-1)!.attachTimeline).not.toHaveBeenCalled()
      dispose()
    }
  })
})
