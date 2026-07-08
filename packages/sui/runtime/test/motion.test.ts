// motion runtime — focused tests for the prefers-reduced-motion guard and
// the AOT-baked `__sui_motion_attach` seam.
//
// The motion runtime is a thin wrapper the compiler emits around every
// `motion:` directive: it pauses the WAAPI animations on attach, then
// plays/reverses them on the DOM event the trigger names (hover / press /
// focus / focus-visible). The reduced-motion guard consults
// `matchMedia('(prefers-reduced-motion: reduce)')` per event so a
// mid-session preference change is honored.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { __sui_motion_attach } from '../src/motion'
import type { MotionAttachConfig } from '../src/motion'

// Install a stub matchMedia before any test runs — the runtime's motion
// module reads `matchMedia` at handler call time, so it must exist on
// `window`. The per-test override (Object.defineProperty) narrows
// `matches` to drive the reduced-motion branch.
if (typeof window !== 'undefined' && typeof window.matchMedia === 'undefined') {
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    configurable: true,
    value: (_query: string) => ({
      matches: false,
      media: _query,
      onchange: null,
      addListener: () => {},
      removeListener: () => {},
      addEventListener: () => {},
      removeEventListener: () => {},
      dispatchEvent: () => false,
    }),
  })
}

/** A minimal WAAPI-like Animation stub. Tracks play / pause / cancel. */
interface StubAnim {
  playCalls: number
  pauseCalls: number
  cancelCalls: number
  currentTime: number
  play: () => void
  pause: () => void
  cancel: () => void
}
const stubAnim = (): StubAnim => {
  const a: StubAnim = {
    playCalls: 0,
    pauseCalls: 0,
    cancelCalls: 0,
    currentTime: 0,
    play: () => { a.playCalls++ },
    pause: () => { a.pauseCalls++ },
    cancel: () => { a.cancelCalls++ },
  }
  return a
}

/** A single host element wired with a hover-trigger motion. */
function hostWithMotion(): { el: HTMLElement; enter: StubAnim; leave: StubAnim } {
  const enter = stubAnim()
  const leave = stubAnim()
  const el = document.createElement('div')
  document.body.appendChild(el)
  const cfg: MotionAttachConfig = {
    trigger: 'hover',
    enter: [enter as unknown as Animation],
    leave: [leave as unknown as Animation],
  }
  __sui_motion_attach(el, cfg)
  return { el, enter, leave }
}

/** Override window.matchMedia for the duration of one test. */
function setReducedMotion(matches: boolean): void {
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    configurable: true,
    value: (query: string) => ({
      matches,
      media: query,
      onchange: null,
      addListener: () => {},
      removeListener: () => {},
      addEventListener: () => {},
      removeEventListener: () => {},
      dispatchEvent: () => false,
    }),
  })
}

afterEach(() => {
  vi.restoreAllMocks()
  document.body.innerHTML = ''
})

describe('motion runtime — __sui_motion_attach', () => {
  it('pauses the enter animations on attach (initial pose is the rest state)', () => {
    setReducedMotion(false)
    const { enter } = hostWithMotion()
    expect(enter.pauseCalls).toBeGreaterThanOrEqual(1)
  })

  it('plays the enter animation on the trigger event (mouseenter)', () => {
    setReducedMotion(false)
    const { el, enter, leave } = hostWithMotion()
    expect(enter.playCalls).toBe(0)
    el.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true }))
    expect(enter.playCalls).toBe(1)
    expect(leave.playCalls).toBe(0)
  })

  it('reverses on the leave event (cancels enter, plays leave from t=0)', () => {
    setReducedMotion(false)
    const { el, enter, leave } = hostWithMotion()
    el.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true }))
    const enterPlaysAfterEnter = enter.playCalls
    el.dispatchEvent(new MouseEvent('mouseleave', { bubbles: true }))
    expect(enter.cancelCalls).toBeGreaterThanOrEqual(1)
    expect(leave.currentTime).toBe(0)
    expect(leave.playCalls).toBe(1)
    expect(enter.playCalls).toBe(enterPlaysAfterEnter) // unchanged on leave
  })
})

describe('motion runtime — reduced-motion guard (matchMedia honored)', () => {
  beforeEach(() => {
    setReducedMotion(true)
  })

  it('does NOT play the enter animation when prefers-reduced-motion: reduce is set', () => {
    const { el, enter } = hostWithMotion()
    el.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true }))
    expect(enter.playCalls).toBe(0)
  })

  it('does NOT play the leave animation when prefers-reduced-motion: reduce is set', () => {
    const { el, leave } = hostWithMotion()
    el.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true }))
    el.dispatchEvent(new MouseEvent('mouseleave', { bubbles: true }))
    expect(leave.playCalls).toBe(0)
  })

  it('still consults matchMedia per event (preference flip mid-session is honored)', () => {
    // The guard is a function call inside each handler, not a one-time
    // capture — the controller can flip reduced-motion off at runtime
    // and animations resume on the next event.
    const { el, enter } = hostWithMotion()
    el.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true }))
    expect(enter.playCalls).toBe(0)

    setReducedMotion(false)
    el.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true }))
    expect(enter.playCalls).toBe(1)
  })
})

describe('motion runtime — focus-visible trigger', () => {
  beforeEach(() => {
    setReducedMotion(false)
  })

  it('plays the enter animation on focusin when :focus-visible matches', () => {
    // jsdom doesn't implement `:focus-visible`; the controller falls back
    // to the focusSource tracker. We tag the target as focus-visible via
    // a stub matches() to drive the positive path.
    const enter = stubAnim()
    const el = document.createElement('div')
    document.body.appendChild(el)
    vi.spyOn(el, 'matches').mockImplementation((sel: string) => sel === ':focus-visible')
    __sui_motion_attach(el, {
      trigger: 'focus-visible',
      enter: [enter as unknown as Animation],
      leave: [stubAnim() as unknown as Animation],
    })
    el.dispatchEvent(new FocusEvent('focusin', { bubbles: true }))
    expect(enter.playCalls).toBe(1)
  })

  it('does NOT play the enter animation on focusin when :focus-visible does NOT match', () => {
    const enter = stubAnim()
    const el = document.createElement('div')
    document.body.appendChild(el)
    vi.spyOn(el, 'matches').mockImplementation(() => false)
    __sui_motion_attach(el, {
      trigger: 'focus-visible',
      enter: [enter as unknown as Animation],
      leave: [stubAnim() as unknown as Animation],
    })
    el.dispatchEvent(new FocusEvent('focusin', { bubbles: true }))
    expect(enter.playCalls).toBe(0)
  })
})

describe('motion runtime — disposer', () => {
  it('cancels attached animations on disposer call (cleanup teardown)', () => {
    setReducedMotion(false)
    const enter = stubAnim()
    const el = document.createElement('div')
    document.body.appendChild(el)
    const dispose = __sui_motion_attach(el, {
      trigger: 'hover',
      enter: [enter as unknown as Animation],
      leave: [stubAnim() as unknown as Animation],
    })
    dispose()
    expect(enter.cancelCalls).toBeGreaterThanOrEqual(1)
  })
})
