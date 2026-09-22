// motion runtime — the reveal/scroll trigger contract.
//
// `style.reveal` must play when the element enters the viewport — never
// stay pinned at opacity 0 while on screen. ViewTimeline + entry/cover
// ranges fail that contract inside nested overflow (app chrome, docs
// rail): progress stays negative for the entire in-view lifetime.
// Reveal therefore uses IntersectionObserver play-once as the primary
// path. `scroll` still prefers ScrollTimeline when present.
//
// These tests pin the attach boundary (mocked animate / IO):
//   1. reveal NEVER attaches a ViewTimeline (nested-scroll trap).
//   2. reveal pauses at creation, plays on first intersection, one-shot.
//   3. No IntersectionObserver → settle final pose + console.warn.
//   4. scroll still uses ScrollTimeline when available.
//   5. loop/enter never touch the timeline / IO seams.

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
      attachTimeline: vi.fn((opts: { timeline?: unknown; observe: (a: unknown) => () => void }) =>
        opts.timeline ? () => {} : opts.observe(c),
      ),
    }
    controlsMade.push(c)
    return c
  },
}))

import { __sail_motion_attach } from '../src/motion'
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
  const dispose = __sail_motion_attach(el, config)
  return { el, dispose }
}

describe('fx:reveal — IntersectionObserver play-once (primary path)', () => {
  it('does NOT attach a ViewTimeline even when the platform has one', () => {
    // Nested overflow makes ViewTimeline entry/cover pin opacity at 0 while
    // on screen. Reveal must not take that path.
    globals.ViewTimeline = FakeViewTimeline
    globals.IntersectionObserver = FakeIntersectionObserver
    const { dispose } = attachConnected(viewFade)
    expect(controlsMade.length).toBe(1)
    expect(controlsMade[0]!.attachTimeline).not.toHaveBeenCalled()
    dispose()
  })

  it('pauses at creation and plays on the first intersection, then stops observing', () => {
    globals.ViewTimeline = FakeViewTimeline // present — still IO
    globals.IntersectionObserver = FakeIntersectionObserver
    const { el, dispose } = attachConnected(viewFade)
    const c = controlsMade[0]!
    expect(c.pause).toHaveBeenCalledTimes(1)
    expect(c.play).not.toHaveBeenCalled()
    expect(ioInstances.length).toBe(1)
    expect(ioInstances[0]!.el).toBe(el)
    ioInstances[0]!.fire(false)
    expect(c.play).not.toHaveBeenCalled()
    ioInstances[0]!.fire(true)
    expect(c.play).toHaveBeenCalledTimes(1)
    expect(ioInstances[0]!.disconnected).toBe(true)
    dispose()
  })

  it('disposal before the element is ever revealed disconnects the observer', () => {
    globals.IntersectionObserver = FakeIntersectionObserver
    const { dispose } = attachConnected(viewFade)
    dispose()
    expect(ioInstances[0]!.disconnected).toBe(true)
  })

  it('no IntersectionObserver: settles at the final pose, LOUDLY', () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    const { dispose } = attachConnected(viewFade)
    const c = controlsMade[0]!
    expect(c.complete).toHaveBeenCalledTimes(1)
    expect(warn).toHaveBeenCalledTimes(1)
    expect(String(warn.mock.calls[0]![0])).toContain('style.reveal')
    dispose()
  })
})

describe('fx:scroll — ScrollTimeline when available', () => {
  it('hands every created animation a ScrollTimeline on the document scroller', () => {
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
