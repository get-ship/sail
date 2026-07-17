// motion runtime — the connection + loop-repeat contract.
//
// Regression guard for the silent-freeze bug: the compiler wires
// `motion:` during DETACHED subtree construction
// (`el = __tpl(...); __sui_motion_attach(el, ...)`), so on first mount the
// element is not yet in the document. A WAAPI animation created on a
// detached element never resolves its play task — it stays `pending` with
// `startTime === null`, frozen at `currentTime` 0. Because Motion attaches
// with `fill: 'both'`, that PINS the element to the first keyframe
// (`opacity: 0`) forever, with no console error and nothing in
// `getAnimations()`. Half the docs site rendered invisible.
//
// The runtime therefore defers animation creation until the element joins
// the document. These tests pin that contract at the `animate()` boundary
// (mocked) so a future refactor that re-attaches synchronously on a
// detached node fails here instead of in production.
//
// Second contract: `motion:loop` must actually loop. A physics preset
// (`spring:bouncy`) is reused for one-shot triggers too, so it carries no
// iteration count of its own — the `loop` trigger supplies the repeat
// (infinite, alternating). Without it the "loop" plays once and settles.

import { afterEach, describe, expect, it, vi } from 'vitest'

const { animateCalls } = vi.hoisted(() => ({
  animateCalls: [] as Array<{ el: unknown; keyframes: unknown; transition: any }>,
}))

vi.mock('motion/mini', () => ({
  animate: (el: unknown, keyframes: unknown, transition: unknown) => {
    animateCalls.push({ el, keyframes, transition })
    return { stop: () => {}, cancel: () => {} }
  },
}))

import { __sui_motion_attach } from '../src/motion'
import type { MotionAttachConfig } from '../src/motion'

// Never reduced — exercise the animating path.
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

const fade: MotionAttachConfig = {
  trigger: 'view',
  motions: [{ property: 'opacity', preset: 'fade', kind: 'tween', from: 0, to: 1, duration: 0.12 }],
}

/** Two rAF cycles — long enough for the runtime's deferred create to fire. */
const flushFrames = (): Promise<void> =>
  new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => resolve())))

afterEach(() => {
  animateCalls.length = 0
  document.body.innerHTML = ''
})

describe('motion runtime — a detached attach defers until the element is connected', () => {
  it('does NOT create the WAAPI animation while the element is detached', () => {
    const el = document.createElement('div') // never appended — detached
    const dispose = __sui_motion_attach(el, fade)
    expect(animateCalls.length).toBe(0) // deferred, not created on a detached node
    // Pinned to its rest pose so it does not flash the final value for a frame.
    expect(el.style.opacity).toBe('0')
    dispose()
  })

  it('creates the animation once the element joins the document', async () => {
    const el = document.createElement('div')
    const dispose = __sui_motion_attach(el, fade)
    expect(animateCalls.length).toBe(0)
    document.body.appendChild(el) // connect
    await flushFrames()
    expect(animateCalls.length).toBe(1)
    expect(animateCalls[0]!.el).toBe(el)
    dispose()
  })

  it('creates synchronously when the element is already connected at attach time', () => {
    const el = document.createElement('div')
    document.body.appendChild(el)
    const dispose = __sui_motion_attach(el, fade)
    expect(animateCalls.length).toBe(1) // no deferral needed
    dispose()
  })

  it('disposing before connection cancels the deferral (no orphaned animation)', async () => {
    const el = document.createElement('div')
    const dispose = __sui_motion_attach(el, fade)
    dispose() // dispose while still detached
    document.body.appendChild(el)
    await flushFrames()
    expect(animateCalls.length).toBe(0)
  })
})

describe('motion runtime — the loop trigger repeats forever', () => {
  it('injects infinite, alternating repeat so a physics preset used as a loop actually loops', () => {
    const el = document.createElement('div')
    document.body.appendChild(el)
    const dispose = __sui_motion_attach(el, {
      trigger: 'loop',
      motions: [{
        property: 'translate-y',
        preset: 'spring:bouncy',
        kind: 'spring',
        stiffness: 320,
        damping: 14,
        mass: 1,
        restSpeed: 0.01,
        restDelta: 0.005,
      }],
    })
    expect(animateCalls.length).toBe(1)
    const t = animateCalls[0]!.transition as { repeat: unknown; repeatType: unknown }
    expect(t.repeat).toBe(Infinity) // loops forever
    expect(t.repeatType).toBe('reverse') // alternates (ping-pong), not a hard restart
    dispose()
  })

  it('an explicit iteration count on the config is not overridden by the loop default', () => {
    const el = document.createElement('div')
    document.body.appendChild(el)
    const dispose = __sui_motion_attach(el, {
      trigger: 'loop',
      motions: [{
        property: 'scale',
        preset: 'pulse',
        kind: 'tween',
        duration: 0.5,
        from: 1,
        to: 1.04,
        iterations: 3,
        direction: 'normal',
      }],
    })
    const t = animateCalls[0]!.transition as { repeat: unknown; repeatType: unknown }
    expect(t.repeat).toBe(3) // author's count wins
    expect(t.repeatType).toBe('loop') // 'normal' → 'loop'
    dispose()
  })
})
