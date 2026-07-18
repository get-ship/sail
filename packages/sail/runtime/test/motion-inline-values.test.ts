// motion runtime — inline values and reactive (signal-valued) configs.
//
// The compiler resolves inline duration/easing/from-to and hands the runtime
// already-numeric config. Two things the runtime still owns, and this file
// pins both:
//
//  1. UNITS. A magnitude may arrive as a bare number (the channel's native
//     unit) or as a unit-bearing string (`'-12%'`, `'120px'`). The second is
//     what made the motion.dev rows archetype expressible at all — the
//     retired `slip` preset's own doc recorded "% is not expressible".
//
//  2. REACTIVITY. When any config field holds a signal the compiler emits a
//     THUNK instead of an object. The runtime re-reads it inside an effect
//     and re-creates the animation on change, cancelling the old one. The
//     disposer must stop both the effect and the live animations.

import { afterEach, describe, expect, it, vi } from 'vitest'

const { animateCalls } = vi.hoisted(() => ({
  animateCalls: [] as Array<{ el: unknown; keyframes: any; transition: any }>,
}))

const { cancelCount } = vi.hoisted(() => ({ cancelCount: { n: 0 } }))

vi.mock('motion/mini', () => ({
  animate: (el: unknown, keyframes: unknown, transition: unknown) => {
    animateCalls.push({ el, keyframes, transition })
    return {
      stop: () => {},
      cancel: () => {
        cancelCount.n += 1
      },
    }
  },
}))

import { __sui_motion_attach } from '../src/motion'
import type { MotionAttachConfig } from '../src/motion'
import { signal } from '../src/signal'

// Never reduced — exercise the animating path. (Reduced motion has its own
// test at the bottom.)
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  configurable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    addEventListener: () => {},
    removeEventListener: () => {},
    addListener: () => {},
    removeListener: () => {},
    onchange: null,
    dispatchEvent: () => false,
  }),
})

function connected(): HTMLElement {
  const el = document.createElement('div')
  document.body.appendChild(el)
  return el
}

afterEach(() => {
  animateCalls.length = 0
  cancelCount.n = 0
  document.body.innerHTML = ''
})

describe('inline magnitudes and units', () => {
  it('appends the channel native unit to a bare number', () => {
    const el = connected()
    const cfg: MotionAttachConfig = {
      trigger: 'loop',
      motions: [{ property: 'translate-x', preset: 'inline', kind: 'tween', from: 0, to: 120 }],
    }
    __sui_motion_attach(el, cfg)
    expect(animateCalls[0]!.keyframes.translate).toEqual(['0px 0px', '120px 0px'])
  })

  it('preserves a unit-bearing magnitude verbatim', () => {
    // THE rows archetype: translate(-12%). No preset could ever say this.
    const el = connected()
    const cfg: MotionAttachConfig = {
      trigger: 'loop',
      motions: [{ property: 'translate-x', preset: 'inline', kind: 'tween', from: 0, to: '-12%' }],
    }
    __sui_motion_attach(el, cfg)
    expect(animateCalls[0]!.keyframes.translate).toEqual(['0px 0px', '-12% 0px'])
  })

  it('treats a numeric string and a bare number identically', () => {
    const el = connected()
    __sui_motion_attach(el, {
      trigger: 'loop',
      motions: [{ property: 'translate-x', preset: 'inline', kind: 'tween', from: 0, to: '120' }],
    })
    expect(animateCalls[0]!.keyframes.translate).toEqual(['0px 0px', '120px 0px'])
  })

  it('gives rotate its degrees and honours an explicit angle unit', () => {
    const el = connected()
    __sui_motion_attach(el, {
      trigger: 'loop',
      motions: [{ property: 'rotate', preset: 'inline', kind: 'tween', from: 0, to: 15 }],
    })
    expect(animateCalls[0]!.keyframes.rotate).toEqual(['0deg', '15deg'])

    animateCalls.length = 0
    __sui_motion_attach(connected(), {
      trigger: 'loop',
      motions: [{ property: 'rotate', preset: 'inline', kind: 'tween', from: 0, to: '0.25turn' }],
    })
    expect(animateCalls[0]!.keyframes.rotate).toEqual(['0deg', '0.25turn'])
  })

  it('keeps scale and opacity unitless even if a string arrives', () => {
    const el = connected()
    __sui_motion_attach(el, {
      trigger: 'loop',
      motions: [
        { property: 'scale', preset: 'inline', kind: 'tween', from: 1, to: '1.75' },
        { property: 'opacity', preset: 'inline', kind: 'tween', from: 1, to: 0 },
      ],
    })
    expect(animateCalls[0]!.keyframes.scale).toEqual([1, 1.75])
    expect(animateCalls[1]!.keyframes.opacity).toEqual([1, 0])
  })

  it('merges both translate axes while preserving each unit', () => {
    const el = connected()
    __sui_motion_attach(el, {
      trigger: 'loop',
      motions: [
        { property: 'translate-x', preset: 'inline', kind: 'tween', from: 0, to: '-12%' },
        { property: 'translate-y', preset: 'inline', kind: 'tween', from: 0, to: 16 },
      ],
    })
    // ONE translate track — WAAPI has exactly one `translate` per element.
    expect(animateCalls).toHaveLength(1)
    expect(animateCalls[0]!.keyframes.translate).toEqual(['0px 0px', '-12% 16px'])
  })
})

describe('inline timing', () => {
  it('passes an inline duration and easing straight to the transition', () => {
    const el = connected()
    __sui_motion_attach(el, {
      trigger: 'loop',
      motions: [
        {
          property: 'scale',
          preset: 'inline',
          kind: 'tween',
          from: 1,
          to: 1.75,
          duration: 3.6,
          easing: 'cubic-bezier(0.6, 0, 0.2, 1)',
        },
      ],
    })
    expect(animateCalls[0]!.transition.duration).toBe(3.6)
    expect(animateCalls[0]!.transition.ease).toBe('cubic-bezier(0.6, 0, 0.2, 1)')
  })

  it('composes a config delay with the sibling stagger step', () => {
    // They are different things: stagger orchestrates BETWEEN elements, delay
    // is this animation's own offset. A staggered list whose members each
    // wait 200ms should do both.
    const parent = document.createElement('div')
    document.body.appendChild(parent)
    const a = document.createElement('div')
    const b = document.createElement('div')
    parent.append(a, b)
    const cfg = (): MotionAttachConfig => ({
      trigger: 'enter',
      stagger: 0.45,
      motions: [{ property: 'opacity', preset: 'inline', kind: 'tween', from: 0, to: 1, delay: 0.2 }],
    })
    __sui_motion_attach(a, cfg())
    __sui_motion_attach(b, cfg())
    expect(animateCalls[0]!.transition.delay).toBeCloseTo(0.2, 5)
    expect(animateCalls[1]!.transition.delay).toBeCloseTo(0.65, 5)
  })

  it('applies a delay with no stagger present', () => {
    const el = connected()
    __sui_motion_attach(el, {
      trigger: 'loop',
      motions: [{ property: 'opacity', preset: 'inline', kind: 'tween', from: 0, to: 1, delay: 0.2 }],
    })
    expect(animateCalls[0]!.transition.delay).toBeCloseTo(0.2, 5)
  })
})

describe('signal-valued config fields', () => {
  it('updates the animation when a timing signal changes', () => {
    const el = connected()
    const speed = signal(1000)
    __sui_motion_attach(el, () => ({
      trigger: 'loop',
      motions: [
        {
          property: 'scale',
          preset: 'inline',
          kind: 'tween',
          from: 1,
          to: 2,
          duration: speed() / 1000,
        },
      ],
    }))
    expect(animateCalls).toHaveLength(1)
    expect(animateCalls[0]!.transition.duration).toBe(1)

    speed.set(2500)
    expect(animateCalls).toHaveLength(2)
    expect(animateCalls[1]!.transition.duration).toBe(2.5)
    // The superseded animation is cancelled, not left running alongside.
    expect(cancelCount.n).toBe(1)
  })

  it('updates the animation when a keyframe signal changes', () => {
    const el = connected()
    const dist = signal(120)
    __sui_motion_attach(el, () => ({
      trigger: 'loop',
      motions: [{ property: 'translate-x', preset: 'inline', kind: 'tween', from: 0, to: dist() }],
    }))
    expect(animateCalls[0]!.keyframes.translate).toEqual(['0px 0px', '120px 0px'])

    dist.set(40)
    expect(animateCalls).toHaveLength(2)
    expect(animateCalls[1]!.keyframes.translate).toEqual(['0px 0px', '40px 0px'])
  })

  it('does not re-create when an unrelated signal changes', () => {
    const el = connected()
    const used = signal(1)
    const unused = signal(99)
    __sui_motion_attach(el, () => ({
      trigger: 'loop',
      motions: [{ property: 'scale', preset: 'inline', kind: 'tween', from: 1, to: used() }],
    }))
    expect(animateCalls).toHaveLength(1)
    unused.set(100)
    expect(animateCalls).toHaveLength(1)
  })

  it('cleans up on dispose: the effect stops and the animation is cancelled', () => {
    const el = connected()
    const dist = signal(10)
    const dispose = __sui_motion_attach(el, () => ({
      trigger: 'loop',
      motions: [{ property: 'translate-x', preset: 'inline', kind: 'tween', from: 0, to: dist() }],
    }))
    expect(animateCalls).toHaveLength(1)

    dispose()
    expect(cancelCount.n).toBe(1)

    // The effect is stopped: a later signal change must NOT resurrect the
    // animation against a torn-down node.
    dist.set(999)
    expect(animateCalls).toHaveLength(1)
  })

  it('a literal (non-thunk) config still attaches with no effect involved', () => {
    const el = connected()
    const dispose = __sui_motion_attach(el, {
      trigger: 'loop',
      motions: [{ property: 'scale', preset: 'inline', kind: 'tween', from: 1, to: 2 }],
    })
    expect(animateCalls).toHaveLength(1)
    dispose()
    expect(cancelCount.n).toBe(1)
  })
})

describe('reduced motion still wins over inline values', () => {
  it('zeroes an inline-duration loop and creates no animation', async () => {
    vi.resetModules()
    vi.doMock('motion/mini', () => ({
      animate: (el: unknown, keyframes: unknown, transition: unknown) => {
        animateCalls.push({ el, keyframes, transition })
        return { stop: () => {}, cancel: () => {} }
      },
    }))
    Object.defineProperty(window, 'matchMedia', {
      writable: true,
      configurable: true,
      value: (query: string) => ({
        matches: true, // prefers-reduced-motion: reduce
        media: query,
        addEventListener: () => {},
        removeEventListener: () => {},
        addListener: () => {},
        removeListener: () => {},
        onchange: null,
        dispatchEvent: () => false,
      }),
    })
    const mod = await import('../src/motion')
    const el = connected()
    mod.__sui_motion_attach(el, {
      trigger: 'loop',
      motions: [
        {
          property: 'scale',
          preset: 'inline',
          kind: 'tween',
          from: 1,
          to: 1.75,
          duration: 3.6,
          easing: 'cubic-bezier(0.6, 0, 0.2, 1)',
        },
      ],
    })
    // A 3600ms inline duration is not special-cased anywhere: reduced motion
    // is enforced GLOBALLY in the runtime, which is exactly why retiring the
    // per-token CSS overrides costs nothing.
    expect(animateCalls).toHaveLength(0)
    expect(el.style.scale).toBe('1')
  })

  it('zeroes a REACTIVE inline-duration loop too', async () => {
    vi.resetModules()
    vi.doMock('motion/mini', () => ({
      animate: (el: unknown, keyframes: unknown, transition: unknown) => {
        animateCalls.push({ el, keyframes, transition })
        return { stop: () => {}, cancel: () => {} }
      },
    }))
    Object.defineProperty(window, 'matchMedia', {
      writable: true,
      configurable: true,
      value: (query: string) => ({
        matches: true,
        media: query,
        addEventListener: () => {},
        removeEventListener: () => {},
        addListener: () => {},
        removeListener: () => {},
        onchange: null,
        dispatchEvent: () => false,
      }),
    })
    const mod = await import('../src/motion')
    const sig = await import('../src/signal')
    const el = connected()
    const speed = sig.signal(3600)
    mod.__sui_motion_attach(el, () => ({
      trigger: 'loop',
      motions: [
        {
          property: 'scale',
          preset: 'inline',
          kind: 'tween',
          from: 1,
          to: 1.75,
          duration: speed() / 1000,
        },
      ],
    }))
    expect(animateCalls).toHaveLength(0)
    // And a signal change under reduced motion must not start one either.
    speed.set(500)
    expect(animateCalls).toHaveLength(0)
  })
})
