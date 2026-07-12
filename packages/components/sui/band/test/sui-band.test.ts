// <sui-band> tests — registration + anatomy, the seam-math cloning contract,
// a11y on clones, reduced motion, the speed→duration derivation, and the
// CSS attribute channels.
//
// jsdom measures zero widths, so the default path exercises the documented
// zero-measurement fallback (original + 1 clone — the minimum seamless
// shape). The measurement-driven test patches getBoundingClientRect on the
// band + copy elements and re-triggers layout through the reactive `speed`
// prop, pinning the real formula: copies = 2 x ceil(band / copy), duration =
// halfWidth / rate.
//
// The element registers via its `.compiled.js` (the production light-DOM
// shape); the queries are root-agnostic (shadow root when a shadow variant
// is registered, host subtree otherwise) so the suite pins the CONTRACT,
// not the DOM mode.

import { afterEach, describe, expect, it } from 'vitest'

import '../src/sui-band/sui-band.compiled.js'

interface Mount {
  el: HTMLElement
  host: HTMLElement
}

const live: Mount[] = []
afterEach(() => {
  for (const m of live.splice(0)) m.host.remove()
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
const bandOf = (m: Mount): HTMLElement | null => rootOf(m).querySelector('.sui-band')
const trackOf = (m: Mount): HTMLElement | null => rootOf(m).querySelector('.track')
const copiesOf = (m: Mount): HTMLElement[] => Array.from(rootOf(m).querySelectorAll('.copy'))
const clonesOf = (m: Mount): HTMLElement[] =>
  Array.from(rootOf(m).querySelectorAll('[data-sui-band-clone]'))

/** The component's stylesheet, wherever the registered mode injected it. */
const stylesOf = (m: Mount): string =>
  m.el.shadowRoot?.querySelector('style')?.textContent ??
  document.getElementById('ship-set-style-sui-band')?.textContent ??
  ''

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
    expect(rootOf(m).querySelector('.copy slot')).not.toBeNull()
  })

  it('carries the loop keyframes, the reverse channel, and the reduced-motion guard in its stylesheet', async () => {
    const m = mount({ html: '<span>X</span>' })
    await flush()
    const css = stylesOf(m)
    expect(css).toContain('translateX(-50%)')
    expect(css).toContain("[direction='reverse'] .track")
    expect(css).toContain('prefers-reduced-motion')
    // Token discipline: the surface reads custom properties, never raw hex.
    expect(css).not.toMatch(/#[0-9a-fA-F]{3,8}\b/)
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
    expect(trackOf(m)!.style.getPropertyValue('--sui-band-duration')).toBe('7.5s')
    m.el.setAttribute('speed', 'slow')
    await flush()
    // slow = 40 px/s → 30s.
    expect(trackOf(m)!.style.getPropertyValue('--sui-band-duration')).toBe('30s')
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
  it('keeps the content visible but adds no clones under prefers-reduced-motion', async () => {
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

describe('<sui-band> — CSS attribute channels', () => {
  it('mirrors tone/direction/angle onto the band root for the attribute-keyed skin + tone contract', async () => {
    const m = mount({
      attrs: { tone: 'primary', direction: 'reverse', angle: 'rise' },
      html: '<span>TONE ◆</span>',
    })
    await flush()
    const band = bandOf(m)!
    expect(band.getAttribute('tone')).toBe('primary')
    expect(band.getAttribute('direction')).toBe('reverse')
    expect(band.getAttribute('angle')).toBe('rise')
    // The host carries the authored attributes too — the global [tone]
    // channel may match either element.
    expect(m.el.getAttribute('tone')).toBe('primary')
  })

  it('defaults are the quiet band: no branded tone, forward, flat', async () => {
    const m = mount({ html: '<span>DEFAULT ◆</span>' })
    await flush()
    const band = bandOf(m)!
    expect(band.getAttribute('tone') ?? 'default').toBe('default')
    expect(band.getAttribute('direction') ?? 'forward').toBe('forward')
    expect(band.getAttribute('angle') ?? 'none').toBe('none')
  })
})
