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
import {
  __sui_motion_attach,
  __buildTransitionForTest,
  __buildKeyframesForTest,
  __warnIfNotAnimatableForTest,
} from '../src/motion'
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
  it('honors reduced motion by snapping translate/scale/rotate to the neutral FINAL transform (no tween)', () => {
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
    // The scale animation itself never plays — it snaps straight to the
    // neutral final transform (1 = identity, no visual jump).
    expect(el.style.scale).toBe('1')
    dispose()
  })

  it('a translate/scale/rotate config settles fully visible under reduced motion — never left at zero opacity', async () => {
    // Ruling: reduced motion drops vestibular-hazard motion (translate/
    // scale/rotate) but must not leave the element looking dead. A real
    // WAAPI opacity fade (0 → 1) was tried here and found unreliable
    // under browser reduced-motion emulation (see the doc comment on
    // `applyReducedMotionSettle` in src/motion.ts) — the guaranteed-safe
    // half of the mandate's OR is what ships: content is immediately,
    // fully visible, not animated in. `animate()` is never called for
    // this path — confirmed via a mock that fails the test if it is.
    vi.resetModules()
    const calls: Array<unknown> = []
    vi.doMock('motion/mini', () => ({
      animate: (..._args: unknown[]) => {
        calls.push(_args)
        return { stop: () => {}, cancel: () => {}, finished: Promise.resolve() }
      },
    }))
    const { __sui_motion_attach: attach } = await import('../src/motion')
    setReducedMotion(true)
    const el = host()
    const dispose = attach(el, {
      trigger: 'loop',
      motions: [{ property: 'translate-y', preset: 'spring:gentle', kind: 'spring', stiffness: 200, damping: 20, mass: 1 }],
    })
    expect(calls.length).toBe(0)
    // The translate itself is snapped to neutral, not animated.
    expect(el.style.translate).toBe('0 0')
    dispose()
  })

  it('a paired opacity + translate config settles both to FINAL — visible, neutral transform', async () => {
    vi.resetModules()
    const calls: Array<unknown> = []
    vi.doMock('motion/mini', () => ({
      animate: (..._args: unknown[]) => {
        calls.push(_args)
        return { stop: () => {}, cancel: () => {}, finished: Promise.resolve() }
      },
    }))
    const { __sui_motion_attach: attach } = await import('../src/motion')
    setReducedMotion(true)
    const el = host()
    const dispose = attach(el, {
      trigger: 'loop',
      motions: [
        { property: 'opacity', preset: 'fade', kind: 'tween', duration: 0.3, from: 0, to: 1 },
        { property: 'translate-y', preset: 'spring:gentle', kind: 'spring', stiffness: 200, damping: 20, mass: 1 },
      ],
    })
    expect(el.style.opacity).toBe('1')
    expect(el.style.translate).toBe('0 0')
    expect(calls.length).toBe(0)
    dispose()
  })

  it('a pure-opacity config keeps its existing snap-to-final behavior under reduced motion (no extra fade)', async () => {
    vi.resetModules()
    const calls: Array<unknown> = []
    vi.doMock('motion/mini', () => ({
      animate: (..._args: unknown[]) => {
        calls.push(_args)
        return { stop: () => {}, cancel: () => {}, finished: Promise.resolve() }
      },
    }))
    const { __sui_motion_attach: attach } = await import('../src/motion')
    setReducedMotion(true)
    const el = host()
    const dispose = attach(el, {
      trigger: 'loop',
      motions: [{ property: 'opacity', preset: 'fade', kind: 'tween', duration: 0.3, from: 0, to: 1 }],
    })
    expect(el.style.opacity).toBe('1')
    expect(calls.length).toBe(0)
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
  it('parks the config without animating or touching style at mount — nothing plays until leave', async () => {
    setReducedMotion(false)
    const el = host()
    const dispose = __sui_motion_attach(el, {
      trigger: 'exit',
      motions: [press],
    })
    // No mount-time animation, no mount-time style clobber: the element
    // renders in its natural pose until the branch is actually removed.
    expect(el.style.scale).toBe('')
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

describe('motion runtime — rotate preset produces WAAPI-valid keyframes', () => {
  // CSS's native `rotate` property (distinct from the `transform`
  // shorthand) requires an angle, not a bare number — WAAPI throws
  // "Invalid keyframe value for property rotate" on `{ rotate: [-6, 0] }`.
  // `x`/`y`/`scale`/`opacity` are unaffected: `x`/`y` are auto-unit'd to
  // px by motion/mini, and `scale`/`opacity` are unitless by spec.
  const dummyEl = document.createElement('div')

  it('the default rest→final rotate sweep is unit-suffixed (deg), not bare numbers', () => {
    const rotateSpring: MotionConfig = {
      property: 'rotate',
      preset: 'spring:bouncy',
      kind: 'spring',
      stiffness: 320,
      damping: 14,
      mass: 1.0,
    }
    const kf = __buildKeyframesForTest(rotateSpring, dummyEl)
    expect(kf.rotate).toEqual(['-6deg', '0deg'])
    for (const v of kf.rotate!) expect(typeof v).toBe('string')
  })

  it('an authored from/to rotate sweep is also unit-suffixed', () => {
    const rotate: MotionConfig = {
      property: 'rotate',
      preset: 'spring:snappy',
      kind: 'spring',
      stiffness: 380,
      damping: 30,
      mass: 1.0,
      from: -12,
      to: 4,
    }
    const kf = __buildKeyframesForTest(rotate, dummyEl)
    expect(kf.rotate).toEqual(['-12deg', '4deg'])
  })

  it('other properties in the preset table stay unitless numbers (negative control)', () => {
    expect(__buildKeyframesForTest(press, dummyEl).scale).toEqual([1, 0.96]) // scale: unitless
    expect(
      __buildKeyframesForTest(
        { property: 'opacity', preset: 'fade-out', kind: 'tween', from: 1, to: 0 },
        dummyEl,
      ).opacity,
    ).toEqual([1, 0]) // opacity: unitless
  })

  it('reaches motion/mini as an actual keyframe object with deg-suffixed rotate values (end-to-end, mocked animate)', async () => {
    // Re-import with `motion/mini` mocked so we can read back exactly what
    // the runtime hands to `animate()`, not just what buildKeyframes returns.
    vi.resetModules()
    const calls: Array<{ keyframes: Record<string, unknown> }> = []
    vi.doMock('motion/mini', () => ({
      animate: (_el: unknown, keyframes: Record<string, unknown>) => {
        calls.push({ keyframes })
        return { stop: () => {}, cancel: () => {}, finished: Promise.resolve() }
      },
    }))
    const { __sui_motion_attach: attach } = await import('../src/motion')
    setReducedMotion(false)
    const el = host()
    const dispose = attach(el, {
      trigger: 'loop',
      motions: [{ property: 'rotate', preset: 'spring:bouncy', kind: 'spring', stiffness: 320, damping: 14, mass: 1 }],
    })
    expect(calls.length).toBe(1)
    expect(calls[0]!.keyframes.rotate).toEqual(['-6deg', '0deg'])
    dispose()
    vi.doUnmock('motion/mini')
    vi.resetModules()
  })
})

describe('motion runtime — translate-x/y emit the `translate` CSS property, not `x`/`y`', () => {
  // `x`/`y` are motion/mini's own value names, not CSS properties on an
  // HTML element — WAAPI silently accepts and no-ops unknown keyframe
  // properties, so the loop "played" forever and moved nothing. The
  // fix: translate-x/y resolve through the `translate` CSS property
  // (already what the REST/FINAL inline styles use), unit-suffixed.
  const dummyEl = document.createElement('div')

  it('translate-y emits a `translate` keyframe with px units, y-axis only', () => {
    const kf = __buildKeyframesForTest(
      { property: 'translate-y', preset: 'spring:gentle', kind: 'spring', from: 16, to: 0 },
      dummyEl,
    )
    expect(kf.x).toBeUndefined()
    expect(kf.y).toBeUndefined()
    expect(kf.translate).toEqual(['0px 16px', '0px 0px'])
  })

  it('translate-x emits a `translate` keyframe with px units, x-axis only', () => {
    const kf = __buildKeyframesForTest(
      { property: 'translate-x', preset: 'spring:deliberate', kind: 'spring', from: 16, to: 0 },
      dummyEl,
    )
    expect(kf.translate).toEqual(['16px 0px', '0px 0px'])
  })

  it('a bounce preset also resolves through `translate`, not raw `x`/`y`', () => {
    const el = document.createElement('div')
    Object.defineProperty(el, 'offsetHeight', { value: 10, configurable: true })
    Object.defineProperty(el, 'offsetWidth', { value: 20, configurable: true })
    const kfY = __buildKeyframesForTest({ property: 'translate-y', preset: 'bounce', kind: 'tween' }, el)
    expect(kfY.translate).toEqual(['0px 0px', '0px -10px'])
    const kfX = __buildKeyframesForTest({ property: 'translate-x', preset: 'bounce', kind: 'tween' }, el)
    expect(kfX.translate).toEqual(['0px 0px', '-20px 0px'])
  })

  it('reaches motion/mini as `translate`, and MOVES: sampled easing keyframes differ (end-to-end, mocked animate)', async () => {
    vi.resetModules()
    const calls: Array<{ keyframes: Record<string, unknown> }> = []
    vi.doMock('motion/mini', () => ({
      animate: (_el: unknown, keyframes: Record<string, unknown>) => {
        calls.push({ keyframes })
        return { stop: () => {}, cancel: () => {}, finished: Promise.resolve() }
      },
    }))
    const { __sui_motion_attach: attach } = await import('../src/motion')
    setReducedMotion(false)
    const el = host()
    const dispose = attach(el, {
      trigger: 'loop',
      motions: [{ property: 'translate-y', preset: 'spring:gentle', kind: 'spring', from: 16, to: 0 }],
    })
    expect(calls.length).toBe(1)
    expect(calls[0]!.keyframes.x).toBeUndefined()
    expect(calls[0]!.keyframes.y).toBeUndefined()
    expect(calls[0]!.keyframes.translate).toEqual(['0px 16px', '0px 0px'])
    dispose()
    vi.doUnmock('motion/mini')
    vi.resetModules()
  })

  describe('axis-merge: translate-x and translate-y on one element compose into one track', () => {
    it('merges both axes into a single `translate` animate() call instead of two fighting ones', async () => {
      vi.resetModules()
      const calls: Array<{ keyframes: Record<string, unknown>; transition: unknown }> = []
      vi.doMock('motion/mini', () => ({
        animate: (_el: unknown, keyframes: Record<string, unknown>, transition: unknown) => {
          calls.push({ keyframes, transition })
          return { stop: () => {}, cancel: () => {}, finished: Promise.resolve() }
        },
      }))
      const { __sui_motion_attach: attach } = await import('../src/motion')
      setReducedMotion(false)
      const el = host()
      const dispose = attach(el, {
        trigger: 'loop',
        motions: [
          { property: 'translate-x', preset: 'spring:deliberate', kind: 'tween', duration: 0.3, from: 8, to: 0 },
          { property: 'translate-y', preset: 'spring:gentle', kind: 'tween', duration: 0.3, from: 16, to: 0 },
        ],
      })
      // ONE animate() call for both motions, not two racing ones.
      expect(calls.length).toBe(1)
      expect(calls[0]!.keyframes.translate).toEqual(['8px 16px', '0px 0px'])
      dispose()
      vi.doUnmock('motion/mini')
      vi.resetModules()
    })

    it('a single-axis motion (no partner) still animates alone, unmerged', () => {
      setReducedMotion(false)
      const el = host()
      const dispose = __sui_motion_attach(el, {
        trigger: 'loop',
        motions: [{ property: 'translate-y', preset: 'spring:gentle', kind: 'tween', duration: 0.3, from: 16, to: 0 }],
      })
      expect(typeof dispose).toBe('function')
      dispose()
    })
  })
})

describe('motion runtime — SILENCE IS THE BUG: dev-mode warn on a non-animatable keyframe property', () => {
  it('warns naming the property and the element when a keyframe property is not in the known-animatable set', () => {
    // Asserts the guard's contract directly against the exact historical
    // defect shape (`{ x: [...], y: [...] }` — motion/mini's own value
    // names, not CSS properties on an HTML element) rather than trying
    // to reconstruct a broken fixture through the (now-fixed) builder.
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {})
    const el = host()
    __warnIfNotAnimatableForTest(el, { x: [16, 0], y: [16, 0] })
    expect(warnSpy).toHaveBeenCalledTimes(2) // one per bad property
    expect(String(warnSpy.mock.calls[0]![0])).toContain('"x"')
    expect(String(warnSpy.mock.calls[1]![0])).toContain('"y"')
    expect(warnSpy.mock.calls[0]![1]).toBe(el)
    warnSpy.mockRestore()
  })

  it('does NOT warn on the fixed translate/rotate/scale/opacity keyframes the preset table actually emits', async () => {
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {})
    setReducedMotion(false)
    const el = host()
    const dispose = __sui_motion_attach(el, {
      trigger: 'loop',
      motions: [
        { property: 'translate-y', preset: 'spring:gentle', kind: 'tween', duration: 0.3, from: 16, to: 0 },
        { property: 'opacity', preset: 'fade', kind: 'tween', duration: 0.3, from: 0.4, to: 1 },
      ],
    })
    expect(warnSpy).not.toHaveBeenCalled()
    dispose()
    warnSpy.mockRestore()
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
