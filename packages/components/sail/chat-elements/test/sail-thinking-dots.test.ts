// Behavioral suite for <sail-thinking-dots>. AOT registration: importing the
// compiled output calls customElements.define() at module load.
//
// Animation gate: the dots use `style.loop` with explicit translate-y px
// travel (not the opacity-bearing `bounce` preset — opacity 0→1 pins rest
// at 0 and reads as "not animating"). The runtime-dispatched test below
// stubs `__sail_motion_attach` and asserts a translate-y loop with a real
// negative `to` reaches the runtime, so a regression that swaps back to
// `pulse`/scale-only (invisible on an 8px dot) fails the suite.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

interface MotionCall {
  target: Element
  trigger: string
  preset: string
  property: string
  from?: number | string
  to?: number | string
  stagger?: number
}

let motionCalls: MotionCall[] = []

// The compiled component imports `__sail_motion_attach` from
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
  motions: Array<{
    preset: string
    property: string
    from?: number | string
    to?: number | string
  }>
  stagger?: number
}

vi.mock('@ship/sail-runtime/internal', async (importOriginal) => {
  const actual = await importOriginal<Record<string, unknown>>()
  return {
    ...actual,
    __sail_motion_attach: (el: Element, config: MotionAttachStubConfig) => motionAttachImpl!(el, config),
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
        from: m.from,
        to: m.to,
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

  it('attaches a visible translate-y loop to every dot (explicit px travel, not opacity)', () => {
    const el = mount('<sail-thinking-dots></sail-thinking-dots>')
    const d = dots(el)
    expect(d.length).toBe(3)
    // Explicit px `to` (e.g. -10px) — not scale pulse, not opacity-only.
    for (const dot of d) {
      const dotCalls = motionCalls.filter((c) => c.target === dot)
      const hop = dotCalls.find((c) => c.property === 'translate-y' && c.trigger === 'loop')
      expect(hop, `dot missing translate-y loop`).toBeDefined()
      const to = String(hop!.to ?? '')
      expect(to.includes('-'), `dot translate-y to must be negative px travel, got ${to}`).toBe(true)
      const opacity = dotCalls.find((c) => c.property === 'opacity')
      expect(opacity, 'opacity channel must not ride the dots (pins rest at 0)').toBeUndefined()
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
