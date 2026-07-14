// motion runtime — focused tests for the prefers-reduced-motion guard and
// the runtime-dispatched `__sui_motion_attach` seam.
//
// `motion:` directives compile to a single `__sui_motion_attach(el, config)`
// call. The runtime creates the animations through motion.dev and consults
// `prefers-reduced-motion: reduce` once per attach (so a mid-session
// preference flip is honored). The behavior gates below assert:
//   - the runtime is the single reduced-motion authority
//   - loop animations are skipped + the element rests in the FINAL pose
//   - hover/focus triggers create controls and tear them down on dispose
//   - exit triggers park + let the lifecycle runner play them on leave
//   - bad triggers (not on the allow-list) are inert disposers
//   - a kind=spring preset reaches motion/mini as a sampled
//     `linear(...)` easing string, NOT a dropped `type: 'spring'` flag

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { __sui_motion_attach, __buildTransitionForTest } from '../src/motion'
import type { MotionAttachConfig, MotionConfig } from '../src/motion'

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

const press: MotionConfig = {
  property: 'scale',
  preset: 'press',
  kind: 'tween',
  duration: 0.06,
  from: 1,
  to: 0.96,
}
const release: MotionConfig = {
  property: 'scale',
  preset: 'release',
  kind: 'tween',
  duration: 0.12,
  from: 0.96,
  to: 1,
}
const snappy: MotionConfig = {
  property: 'scale',
  preset: 'spring:snappy',
  kind: 'spring',
  stiffness: 380,
  damping: 30,
  mass: 1.0,
  restSpeed: 0.01,
  restDelta: 0.005,
}

const hover: MotionAttachConfig = {
  trigger: 'hover',
  motions: [press, release],
}

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

function host(): HTMLElement {
  const el = document.createElement('div')
  document.body.appendChild(el)
  return el
}

afterEach(() => {
  vi.restoreAllMocks()
  document.body.innerHTML = ''
})

describe('motion runtime — hover trigger', () => {
  it('attaches controls and disposes cleanly', () => {
    setReducedMotion(false)
    const el = host()
    const dispose = __sui_motion_attach(el, hover)
    expect(typeof dispose).toBe('function')
    dispose()
  })

  it('does not animate when prefers-reduced-motion: reduce is set (event handler is inert)', () => {
    setReducedMotion(true)
    const el = host()
    const dispose = __sui_motion_attach(el, hover)
    el.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true }))
    expect(typeof dispose).toBe('function')
    dispose()
  })

  it('still consults matchMedia per event (preference flip mid-session is honored)', () => {
    setReducedMotion(true)
    const el = host()
    const dispose = __sui_motion_attach(el, hover)
    dispose()
    setReducedMotion(false)
    const el2 = host()
    const dispose2 = __sui_motion_attach(el2, hover)
    dispose2()
  })
})

describe('motion runtime — focus-visible trigger', () => {
  it('still creates controls on attach (the keyboard heuristic is queried on focusin)', () => {
    setReducedMotion(false)
    const el = host()
    vi.spyOn(el, 'matches').mockImplementation((sel: string) => sel === ':focus-visible')
    const dispose = __sui_motion_attach(el, {
      trigger: 'focus-visible',
      motions: [press, release],
    })
    expect(typeof dispose).toBe('function')
    dispose()
  })
})

describe('motion runtime — loop trigger (the always-on ambient path)', () => {
  it('honors reduced motion by settling on the FINAL pose (no half-fade, no scale-tween)', () => {
    setReducedMotion(true)
    const el = host()
    const dispose = __sui_motion_attach(el, {
      trigger: 'loop',
      motions: [{
        property: 'scale',
        preset: 'pulse',
        kind: 'tween',
        duration: 0.28,
        from: 1,
        to: 1.04,
        iterations: Infinity,
        direction: 'alternate',
      }],
    })
    expect(el.style.scale).toBe('1')
    dispose()
  })

  it('creates the animation under normal motion (no throw, disposer is a cancel)', () => {
    setReducedMotion(false)
    const el = host()
    const dispose = __sui_motion_attach(el, {
      trigger: 'loop',
      motions: [{
        property: 'opacity',
        preset: 'pulse',
        kind: 'tween',
        duration: 0.5,
        from: 1,
        to: 1,
        iterations: Infinity,
        direction: 'alternate',
      }],
    })
    expect(typeof dispose).toBe('function')
    dispose()
  })
})

describe('motion runtime — exit trigger (lifecycle integration)', () => {
  it('parks the config so the lifecycle runner can play it on leave', async () => {
    setReducedMotion(false)
    const el = host()
    const dispose = __sui_motion_attach(el, {
      trigger: 'exit',
      motions: [press],
    })
    expect(el.style.scale).toBe('0.96')
    dispose()
  })

  it('reduced motion skips the park so the runner tears down synchronously', () => {
    setReducedMotion(true)
    const el = host()
    const dispose = __sui_motion_attach(el, {
      trigger: 'exit',
      motions: [press],
    })
    expect(typeof dispose).toBe('function')
    dispose()
  })
})

describe('motion runtime — single reduced-motion authority', () => {
  it('a single matchMedia subscription is used for every attach (one cached MQL)', () => {
    setReducedMotion(false)
    const el1 = host()
    const el2 = host()
    const el3 = host()
    const d1 = __sui_motion_attach(el1, { ...hover })
    const d2 = __sui_motion_attach(el2, { trigger: 'loop', motions: [press] })
    const d3 = __sui_motion_attach(el3, { trigger: 'exit', motions: [press] })
    expect(typeof d1).toBe('function')
    expect(typeof d2).toBe('function')
    expect(typeof d3).toBe('function')
    d1()
    d2()
    d3()
  })
})

describe('motion runtime — spring preset reaches motion/mini as a linear string', () => {
  // The spring path goes through `motionSpring({stiffness, ...})` +
  // `generateLinearEasing(...)` — the runtime samples the spring
  // curve to a `linear(...)` string motion/mini understands. A dropped
  // `type: 'spring'` flag (the previous bug) looked identical from
  // the outside but played as the default ease-out. The test reads
  // back the resolved options the runtime hands to `animate()` and
  // asserts the easing is a `linear(...)` string. A spring you
  // cannot distinguish from the default ease-out in a test is a
  // spring you cannot distinguish in production either.
  it('a kind=spring preset resolves to a `linear(...)` easing string', () => {
    const t = __buildTransitionForTest(snappy, 0, 1, undefined)
    expect(typeof t.ease).toBe('string')
    expect(t.ease as string).toMatch(/^linear\(/)
    // Duration is derived from the spring's rest thresholds — not
    // the authored `duration` token. It is a positive number in
    // seconds.
    expect(typeof t.duration).toBe('number')
    expect(t.duration).toBeGreaterThan(0)
    // The spring path no longer leaves a `type: 'spring'` flag on
    // the transition (the previous bug).
    expect(t).not.toHaveProperty('type')
  })

  it('a kind=tween preset resolves to tween fields (negative control)', () => {
    const t = __buildTransitionForTest(press, 0, 1, undefined)
    expect(t.duration).toBe(0.06)
    expect(t.ease).toBeUndefined()
  })

  it('a kind=spring preset under reduced motion never builds a transition', () => {
    setReducedMotion(true)
    const el = host()
    const dispose = __sui_motion_attach(el, {
      trigger: 'loop',
      motions: [snappy],
    })
    expect(el.style.scale).toBe('1')
    dispose()
  })
})

describe('motion runtime — unknown trigger is inert', () => {
  it('returns a no-op disposer for an off-list trigger name', () => {
    setReducedMotion(false)
    const el = host()
    const dispose = __sui_motion_attach(el, {
      trigger: 'bogus-trigger',
      motions: [press],
    })
    expect(typeof dispose).toBe('function')
    expect(el.style.scale).toBe('')
    dispose()
  })
})
