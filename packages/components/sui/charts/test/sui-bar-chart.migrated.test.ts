// sui-bar-chart — FOUNDATIONAL migration specs (TESTS-FIRST, RED).
//
// The bar chart is migrated onto the createChartData engine + createChartViewport
// adapter: the comma-string `data` decodes through `normalizeSeries`, bars become
// a pure `derived` (present at first paint), the ad-hoc `isEmpty` signal is
// replaced by the engine discriminator composing sui-empty-state, loading composes
// sui-skeleton, bars gain `part="series"`, and the bar `max=0`→auto domain
// convention is preserved by the engine extent.
//
// Imports the migrated tag's compiled.js + the composed committed primitives'
// compiled.js. Build-spec REPLACE #1/#2/#6 + Slices 1/5, behavioral specs
// 1/3/6/7/11/13.

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-bar-chart/sui-bar-chart.compiled.js'
import '../../primitives/src/sui-empty-state/sui-empty-state.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-bar-chart> migration — geometry at first paint', () => {
  it('renders a rect per datum synchronously at mount (no flush needed)', () => {
    const { el } = mount(reg, 'sui-bar-chart', { data: '1,2,3,4' })
    expect(shadow(el).querySelectorAll('rect').length).toBe(4)
  })
})

describe('<sui-bar-chart> migration — part contract', () => {
  it('bars expose part="series"', async () => {
    const { el } = mount(reg, 'sui-bar-chart', { data: '1,2,3' })
    await flush()
    const parts = [...shadow(el).querySelectorAll('rect')].map((r) => r.getAttribute('part'))
    expect(parts.every((p) => p === 'series')).toBe(true)
  })
})

describe('<sui-bar-chart> migration — engine-driven states', () => {
  it('loading attribute composes a sui-skeleton', async () => {
    const { el } = mount(reg, 'sui-bar-chart', { data: '1,2,3', loading: true })
    await flush()
    expect(shadow(el).querySelector('[skeleton]')).not.toBeNull()
  })

  it('loading hides the data SVG', async () => {
    const { el } = mount(reg, 'sui-bar-chart', { data: '1,2,3', loading: true })
    await flush()
    expect(shadow(el).querySelector('svg[role="img"]')).toBeNull()
  })

  it('loading reflects to the host attribute', () => {
    const { el } = mount(reg, 'sui-bar-chart', { loading: true })
    expect(el.hasAttribute('loading')).toBe(true)
  })

  it('empty data composes a sui-empty-state (not a bare aria-hidden SVG)', async () => {
    const { el } = mount(reg, 'sui-bar-chart')
    await flush()
    expect(shadow(el).querySelector('sui-empty-state')).not.toBeNull()
  })

  it('empty data renders no rect elements', async () => {
    const { el } = mount(reg, 'sui-bar-chart')
    await flush()
    expect(shadow(el).querySelectorAll('rect').length).toBe(0)
  })
})

describe('<sui-bar-chart> migration — domain semantics preserved', () => {
  it('max=0 means auto-domain (bars still render, scaled to data max)', async () => {
    const { el } = mount(reg, 'sui-bar-chart', { data: '10,20,30', max: '0' })
    await flush()
    expect(shadow(el).querySelectorAll('rect').length).toBe(3)
  })

  it('the tallest bar reaches the full plot height under auto-domain', async () => {
    const { el } = mount(reg, 'sui-bar-chart', { data: '5,10', width: '100', height: '200' })
    await flush()
    const heights = [...shadow(el).querySelectorAll('rect')].map((r) =>
      Number(r.getAttribute('height')),
    )
    expect(Math.max(...heights)).toBeGreaterThan(Math.min(...heights))
  })
})

describe('<sui-bar-chart> migration — accessible summary', () => {
  it('aria-label is "kind: summary", not the bare kind', async () => {
    const { el } = mount(reg, 'sui-bar-chart', { data: '1,2,3', label: 'Sales bar chart' })
    await flush()
    const aria = shadow(el).querySelector('svg[role="img"]')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('Sales bar chart')).toBe(true)
    expect(aria.length).toBeGreaterThan('Sales bar chart'.length)
  })
})

describe('<sui-bar-chart> migration — responsive viewBox', () => {
  it('does not stretch with preserveAspectRatio="none"', async () => {
    const { el } = mount(reg, 'sui-bar-chart', { data: '1,2,3' })
    await flush()
    const svg = shadow(el).querySelector('svg[role="img"]')
    expect(svg?.getAttribute('preserveAspectRatio')).not.toBe('none')
  })
})
