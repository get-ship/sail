// <sui-band> tests — registration + anatomy, the seam-math cloning contract,
// a11y on clones, reduced motion, the speed→duration derivation, and the
// reflected prop channels.
//
// jsdom measures zero widths, so the default path exercises the documented
// zero-measurement fallback (original + 1 clone — the minimum seamless
// shape). The measurement-driven test patches getBoundingClientRect on the
// band + copy elements and re-triggers layout through the reactive `speed`
// prop, pinning the real formula: copies = 2 x ceil(band / copy), duration =
// halfWidth / rate.
//
// jsdom implements no Web Animations API, so `Element.animate` is recorded by a
// stub: the loop's contract (a linear, infinite, -50% translate whose duration
// is the measured px/s time, reversed by `direction`) is asserted against the
// options the controller passes.
//
// The element registers via its `.compiled.js` (the production light-DOM
// shape); the queries are structural (the root, its track, the copies inside
// it) because the component hand-authors no classes.

import { afterEach, beforeEach, describe, expect, it } from 'vitest'

import '../src/sui-band/sui-band.compiled.js'

interface Mount {
  el: HTMLElement
  host: HTMLElement
}

/** One recorded `el.animate(keyframes, options)` call. */
interface AnimateCall {
  target: Element
  keyframes: Keyframe[]
  options: KeyframeAnimationOptions
}

let animateCalls: AnimateCall[] = []
let cancelled = 0

/** jsdom ships no WAAPI. Record the calls and hand back a minimal Animation. */
beforeEach(() => {
  animateCalls = []
  cancelled = 0
  ;(Element.prototype as unknown as { animate: unknown }).animate = function animate(
    this: Element,
    keyframes: Keyframe[],
    options: KeyframeAnimationOptions,
  ) {
    animateCalls.push({ target: this, keyframes, options })
    return {
      cancel() {
        cancelled++
      },
      play() {},
      finish() {},
    } as unknown as Animation
  }
})

const live: Mount[] = []
afterEach(() => {
  for (const m of live.splice(0)) m.host.remove()
  delete (Element.prototype as unknown as { animate?: unknown }).animate
})

/** Yield a microtask + macrotask turn so the controller's queued first
 *  layout (and any effect re-runs) settle. */
const flush = (): Promise<void> => new Promise((r) => setTimeout(r, 0))

/** Create a <sui-band> with content IN PLACE before it connects — the
 *  light-DOM capture and native shadow slotting both require the children
 *  to exist at connect time. */
function mount(opts: { attrs?: Record<string, string>; html?: string } = {}): Mount {
  const host = document.createElement('div')
  const el = document.createElement('sui-band')
  for (const [k, v] of Object.entries(opts.attrs ?? {})) el.setAttribute(k, v)
  if (opts.html) el.innerHTML = opts.html
  host.appendChild(el)
  document.body.appendChild(host)
  const m: Mount = { el, host }
  live.push(m)
  return m
}

const rootOf = (m: Mount): ParentNode => m.el.shadowRoot ?? m.el
/** The band surface — the component's root element. */
const bandOf = (m: Mount): HTMLElement | null =>
  (rootOf(m) as Element).querySelector('.sui-root')
/** The moving track — the band's only child. */
const trackOf = (m: Mount): HTMLElement | null =>
  (bandOf(m)?.firstElementChild as HTMLElement | null) ?? null
/** Every copy of the content: the original + the clones, in track order. */
const copiesOf = (m: Mount): HTMLElement[] =>
  Array.from((trackOf(m)?.children ?? []) as HTMLCollectionOf<HTMLElement>)
const clonesOf = (m: Mount): HTMLElement[] =>
  Array.from(rootOf(m).querySelectorAll('[data-sui-band-clone]'))

/** The last loop the controller started on the track. */
const lastLoop = (m: Mount): AnimateCall | undefined => {
  const track = trackOf(m)
  return animateCalls.filter((c) => c.target === track).at(-1)
}

/** Fix an element's measured width (jsdom always measures 0). */
function setWidth(el: HTMLElement, width: number): void {
  Object.defineProperty(el, 'getBoundingClientRect', {
    configurable: true,
    value: () => ({
      width,
      height: 24,
      x: 0,
      y: 0,
      top: 0,
      left: 0,
      right: width,
      bottom: 24,
      toJSON: () => ({}),
    }),
  })
}

describe('<sui-band> — registration + anatomy', () => {
  it('registers the custom element and renders band > track > copy > slot', async () => {
    const m = mount({ html: '<span>LOOP ◆</span>' })
    await flush()
    expect(customElements.get('sui-band')).toBeDefined()
    expect(bandOf(m)).not.toBeNull()
    expect(trackOf(m)).not.toBeNull()
    expect(copiesOf(m).length).toBeGreaterThan(0)
    expect(copiesOf(m)[0]!.querySelector('slot')).not.toBeNull()
  })

  it('runs the seamless loop on the track: linear, infinite, -50%', async () => {
    const m = mount({ html: '<span>X</span>' })
    await flush()
    const loop = lastLoop(m)
    expect(loop).toBeDefined()
    expect(loop!.keyframes).toEqual([
      { transform: 'translateX(0)' },
      { transform: 'translateX(-50%)' },
    ])
    expect(loop!.options.easing).toBe('linear')
    expect(loop!.options.iterations).toBe(Infinity)
    expect(loop!.options.direction).toBe('normal')
  })

  it('reverses the same loop for direction="reverse"', async () => {
    const m = mount({ attrs: { direction: 'reverse' }, html: '<span>X</span>' })
    await flush()
    expect(lastLoop(m)!.options.direction).toBe('reverse')
  })
})

describe('<sui-band> — seam-math cloning', () => {
  it('duplicates its content so the -50% loop never shows a seam (zero-measure fallback: original + 1 clone)', async () => {
    const m = mount({ html: '<span>NO RAW CSS ◆</span>' })
    await flush()
    // jsdom measures 0 → the documented minimum seamless shape.
    expect(clonesOf(m).length).toBe(1)
    expect(copiesOf(m).length).toBe(2) // even — the -50% loop lands on a copy boundary
    expect(clonesOf(m)[0]!.textContent).toContain('NO RAW CSS ◆')
  })

  it('grows the copy count from real measurements: copies = 2 x ceil(band / copy)', async () => {
    const m = mount({ html: '<span>MEASURED ◆</span>' })
    await flush()
    setWidth(bandOf(m)!, 1000)
    setWidth(copiesOf(m)[0]!, 300)
    // `speed` is reactive — flipping it re-runs the layout effect against
    // the patched measurements.
    m.el.setAttribute('speed', 'fast')
    await flush()
    // ceil(1000/300) = 4 per half → 8 copies → 7 clones.
    expect(copiesOf(m).length).toBe(8)
    expect(clonesOf(m).length).toBe(7)
    for (const c of clonesOf(m)) expect(c.textContent).toContain('MEASURED ◆')
  })

  it('derives the duration from measured width x speed (honest px/s)', async () => {
    const m = mount({ html: '<span>SPEED ◆</span>' })
    await flush()
    setWidth(bandOf(m)!, 1000)
    setWidth(copiesOf(m)[0]!, 300)
    m.el.setAttribute('speed', 'fast')
    await flush()
    // halfWidth = 4 copies x 300px = 1200px; fast = 160 px/s → 7.5s.
    expect(lastLoop(m)!.options.duration).toBe(7500)
    m.el.setAttribute('speed', 'slow')
    await flush()
    // slow = 40 px/s → 30s.
    expect(lastLoop(m)!.options.duration).toBe(30000)
  })
})

describe('<sui-band> — clones are presentation only', () => {
  it('marks every clone aria-hidden + inert; the original copy stays accessible', async () => {
    const m = mount({ html: '<span>A11Y ◆</span>' })
    await flush()
    const all = copiesOf(m)
    const original = all.find((c) => !c.hasAttribute('data-sui-band-clone'))
    expect(original).toBeDefined()
    expect(original!.getAttribute('aria-hidden')).toBeNull()
    for (const c of clonesOf(m)) {
      expect(c.getAttribute('aria-hidden')).toBe('true')
      expect(c.hasAttribute('inert')).toBe(true)
      // A clone is the same box as the original, or the half-track it
      // measures is not half the track.
      expect(c.className).toBe(original!.className)
    }
  })

  it('strips ids inside clones so the document never carries duplicates', async () => {
    const m = mount({ html: '<span id="brand">SHIP</span>' })
    await flush()
    expect(clonesOf(m).length).toBeGreaterThan(0)
    for (const c of clonesOf(m)) expect(c.querySelector('[id]')).toBeNull()
    // The original keeps its id (whichever root it was slotted through).
    expect(document.getElementById('brand') ?? rootOf(m).querySelector('#brand')).not.toBeNull()
  })
})

describe('<sui-band> — reduced motion', () => {
  it('keeps the content visible, adds no clones, and plays nothing under prefers-reduced-motion', async () => {
    const orig = window.matchMedia
    Object.defineProperty(window, 'matchMedia', {
      configurable: true,
      writable: true,
      value: (query: string) => ({
        matches: query.includes('prefers-reduced-motion'),
        media: query,
        onchange: null,
        addListener: () => {},
        removeListener: () => {},
        addEventListener: () => {},
        removeEventListener: () => {},
        dispatchEvent: () => false,
      }),
    })
    try {
      const m = mount({ html: '<span>STILL HERE ◆</span>' })
      await flush()
      expect(clonesOf(m).length).toBe(0)
      expect(copiesOf(m).length).toBe(1)
      // WAAPI is programmatic — the CSS kill switch cannot stop it, so the
      // controller must never start the loop in the first place.
      expect(lastLoop(m)).toBeUndefined()
      // The original content is still reachable — motion off never means
      // content gone.
      expect((m.el.textContent ?? '') + (rootOf(m).textContent ?? '')).toContain('STILL HERE ◆')
    } finally {
      Object.defineProperty(window, 'matchMedia', {
        configurable: true,
        writable: true,
        value: orig,
      })
    }
  })
})

describe('<sui-band> — reflected prop channels', () => {
  it('reflects tone/direction/angle onto the host — the tone contract and the variant table read them there', async () => {
    const m = mount({
      attrs: { tone: 'primary', direction: 'reverse', angle: 'rise' },
      html: '<span>TONE ◆</span>',
    })
    await flush()
    expect(m.el.getAttribute('tone')).toBe('primary')
    expect(m.el.getAttribute('direction')).toBe('reverse')
    expect(m.el.getAttribute('angle')).toBe('rise')
  })

  it('defaults are the quiet band: no branded tone, forward, flat', async () => {
    const m = mount({ html: '<span>DEFAULT ◆</span>' })
    await flush()
    expect(m.el.getAttribute('tone') ?? 'default').toBe('default')
    expect(m.el.getAttribute('direction') ?? 'forward').toBe('forward')
    expect(m.el.getAttribute('angle') ?? 'none').toBe('none')
  })
})
