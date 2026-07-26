// Behavioral suite for <sail-thinking-dots>. AOT registration: importing the
// compiled output calls customElements.define() at module load.
//
// Animation gate: the dots are wired through `fx:loop` with the
// `bounce` preset (translateY 0→-100% + opacity 0.4→1, infinite, alternate,
// 150ms stagger step). The `pulse` preset that the migration originally
// picked was a 1.04× scale, sub-pixel on a 7-12px dot — invisible. The
// runtime-dispatched test below stubs `__sui_motion_attach` and asserts
// the bounce preset reaches the runtime, so a regression that swaps back
// to `pulse` (or to a fixed scale) fails the suite.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

interface MotionCall {
  target: Element
  trigger: string
  preset: string
  property: string
  stagger?: number
}

let motionCalls: MotionCall[] = []

// The compiled component imports `__sui_motion_attach` from
// `@ship/sail-runtime/internal` as a named ESM import — a live binding the
// module namespace object exposes read-only, so `require(...).x = fn`
// (the old approach) throws `Attempted to assign to readonly property`.
// `vi.mock` replaces the module at resolution time instead, which is the
// seam ESM actually offers; the real exports pass through untouched except
// for the one function under test, redirected through a mutable ref so each
// test can swap its behavior via `installMotionStub()`.
let motionAttachImpl: ((el: Element, config: MotionAttachStubConfig) => () => void) | null = null

interface MotionAttachStubConfig {
  trigger: string
  motions: Array<{ preset: string; property: string }>
  stagger?: number
}

vi.mock('@ship/sail-runtime/internal', async (importOriginal) => {
  const actual = await importOriginal<Record<string, unknown>>()
  return {
    ...actual,
    __sui_motion_attach: (el: Element, config: MotionAttachStubConfig) => motionAttachImpl!(el, config),
  }
})

import '../src/sail-thinking-dots/sail-thinking-dots.compiled.js'

function installMotionStub(): void {
  motionCalls = []
  motionAttachImpl = (el, config) => {
    for (const m of config.motions) {
      motionCalls.push({
        target: el,
        trigger: config.trigger,
        preset: m.preset,
        property: m.property,
        stagger: config.stagger,
      })
    }
    return () => {}
  }
}

const containers: HTMLElement[] = []
const mk = (): HTMLElement => {
  const c = document.createElement('div')
  document.body.appendChild(c)
  containers.push(c)
  return c
}
afterEach(() => {
  for (const c of containers.splice(0)) c.remove()
  motionCalls = []
})

const mount = (markup: string): HTMLElement => {
  installMotionStub()
  const c = mk()
  c.innerHTML = markup
  return c.querySelector('sail-thinking-dots')!
}
// The live region is the announced status — anchor on the ARIA role, which is
// the contract a screen reader reads. Light DOM: query off the host.
const region = (el: HTMLElement): HTMLElement =>
  el.querySelector<HTMLElement>('[role="status"]')!
// The dots are decorative by contract — aria-hidden is what makes them so.
const dots = (el: HTMLElement): NodeListOf<HTMLElement> =>
  el.querySelectorAll<HTMLElement>('span[aria-hidden="true"]')

describe('<sail-thinking-dots>', () => {
  it('registers with the global custom-element registry', () => {
    expect(customElements.get('sail-thinking-dots')).toBeDefined()
  })

  it('announces a polite live status with a default accessible label', () => {
    const el = mount('<sail-thinking-dots></sail-thinking-dots>')
    const r = region(el)
    expect(r.getAttribute('role')).toBe('status')
    expect(r.getAttribute('aria-live')).toBe('polite')
    expect(r.getAttribute('aria-label')).toBe('Assistant is thinking')
  })

  it('lets the consumer override the announced label', () => {
    const el = mount('<sail-thinking-dots label="Running tool…"></sail-thinking-dots>')
    expect(region(el).getAttribute('aria-label')).toBe('Running tool…')
  })

  it('keeps the three dots decorative (aria-hidden), not announced individually', () => {
    const el = mount('<sail-thinking-dots></sail-thinking-dots>')
    const d = dots(el)
    expect(d.length).toBe(3)
    for (const dot of d) expect(dot.getAttribute('aria-hidden')).toBe('true')
  })

  it('reflects tone + size onto the host so CSS can scale/color the dots', () => {
    const el = mount('<sail-thinking-dots tone="primary" size="lg"></sail-thinking-dots>')
    expect(el.getAttribute('tone')).toBe('primary')
    expect(el.getAttribute('size')).toBe('lg')
  })

  it('attaches the runtime bounce to every dot — translateY 0→-100% on a 7-12px dot is visible motion', () => {
    const el = mount('<sail-thinking-dots></sail-thinking-dots>')
    const d = dots(el)
    expect(d.length).toBe(3)
    // Every dot must reach the runtime with a `bounce` preset, on the
    // `translate-y` axis (the migration lost the original translateY-25%
    // axis when it baked `pulse` as scale). The `bounce` preset's
    // `from: 0, to: 1` is the runtime's contract for the per-axis
    // property; the actual -100% translation is computed from the
    // element's own height in the runtime.
    for (const dot of d) {
      const dotCalls = motionCalls.filter((c) => c.target === dot)
      const bounce = dotCalls.find((c) => c.preset === 'bounce' && c.property === 'translate-y')
      expect(bounce, `dot missing translate-y:bounce`).toBeDefined()
    }
  })

  it('passes the 150ms stagger step to the runtime so the dots phase-offset, not start together', () => {
    const el = mount('<sail-thinking-dots></sail-thinking-dots>')
    // The compiler emits `stagger: 0.15` (the runtime reads seconds;
    // 150ms == 0.15s). At least one of the dot attach calls must carry it.
    const withStagger = motionCalls.filter((c) => c.stagger === 0.15)
    expect(withStagger.length).toBeGreaterThan(0)
  })

  it('mounts and unmounts cleanly across the stream lifecycle', async () => {
    installMotionStub()
    const c = mk()
    c.innerHTML = '<sail-thinking-dots></sail-thinking-dots>'
    await new Promise((r) => setTimeout(r, 0))
    expect(c.querySelector('sail-thinking-dots')).not.toBeNull()
    c.querySelector('sail-thinking-dots')!.remove()
    await new Promise((r) => setTimeout(r, 0))
    expect(c.querySelector('sail-thinking-dots')).toBeNull()
  })
})
