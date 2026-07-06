// sui-line-chart — FOUNDATIONAL migration specs (TESTS-FIRST, RED).
//
// The line chart is migrated onto the createChartData engine + createChartViewport
// adapter: geometry becomes a pure `derived` (present at first paint, no flush),
// loading/empty branch on the engine discriminators composing the primitives
// pack, series paths gain `part="series"` and grid lines `part="grid"`, and the
// viewBox tracks the measured viewport (no `preserveAspectRatio="none"`).
//
// Imports the migrated tag's compiled.js + the composed committed primitives'
// compiled.js (sui-skeleton / sui-empty-state). Build-spec REPLACE #1/#3/#6 +
// Slices 1/4/5, behavioral specs 1/5/6/7/11/13.

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-line-chart/sui-line-chart.compiled.js'
import '../../primitives/src/sui-empty-state/sui-empty-state.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

const TWO_SERIES = JSON.stringify([
  { name: 'a', data: [1, 2, 3, 4] },
  { name: 'b', data: [4, 3, 2, 1] },
])

describe('<sui-line-chart> migration — geometry at first paint', () => {
  it('renders series paths synchronously at mount (no flush needed)', () => {
    const { el } = mount(reg, 'sui-line-chart', { series: TWO_SERIES })
    // The derived geometry is available at render time — the effect-into-signal
    // first-paint gap is gone, so paths exist before any microtask flush.
    expect(shadow(el).querySelectorAll('path').length).toBe(2)
  })
})

describe('<sui-line-chart> migration — part contract', () => {
  it('series paths expose part="series"', async () => {
    const { el } = mount(reg, 'sui-line-chart', { series: TWO_SERIES })
    await flush()
    const parts = [...shadow(el).querySelectorAll('path')].map((p) => p.getAttribute('part'))
    expect(parts.every((p) => p === 'series')).toBe(true)
  })

  it('grid lines expose part="grid"', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sui-line-chart', { series, grid: true })
    await flush()
    const gridParts = [...shadow(el).querySelectorAll('line.grid-line')].map((l) =>
      l.getAttribute('part'),
    )
    expect(gridParts.every((p) => p === 'grid')).toBe(true)
  })
})

describe('<sui-line-chart> migration — engine-driven states', () => {
  it('loading attribute composes a sui-skeleton', async () => {
    const { el } = mount(reg, 'sui-line-chart', { series: TWO_SERIES, loading: true })
    await flush()
    expect(shadow(el).querySelector('[skeleton]')).not.toBeNull()
  })

  it('loading hides the data SVG', async () => {
    const { el } = mount(reg, 'sui-line-chart', { series: TWO_SERIES, loading: true })
    await flush()
    expect(shadow(el).querySelector('svg[role="img"]')).toBeNull()
  })

  it('loading reflects to the host attribute', () => {
    const { el } = mount(reg, 'sui-line-chart', { loading: true })
    expect(el.hasAttribute('loading')).toBe(true)
  })

  it('empty (no series) composes a sui-empty-state', async () => {
    const { el } = mount(reg, 'sui-line-chart')
    await flush()
    expect(shadow(el).querySelector('sui-empty-state')).not.toBeNull()
  })

  it('empty data renders no series paths', async () => {
    const { el } = mount(reg, 'sui-line-chart')
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(0)
  })

  it('clearing loading reveals the data SVG', async () => {
    const { el } = mount(reg, 'sui-line-chart', { series: TWO_SERIES, loading: true })
    await flush()
    el.removeAttribute('loading')
    await flush()
    expect(shadow(el).querySelector('svg[role="img"]')).not.toBeNull()
  })
})

describe('<sui-line-chart> migration — accessible summary', () => {
  it('aria-label is "kind: summary", not the bare kind', async () => {
    const { el } = mount(reg, 'sui-line-chart', { series: TWO_SERIES, label: 'Activity line chart' })
    await flush()
    const aria = shadow(el).querySelector('svg[role="img"]')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('Activity line chart')).toBe(true)
    // The summary suffix names the underlying data (series/points/range).
    expect(aria.length).toBeGreaterThan('Activity line chart'.length)
  })
})

describe('<sui-line-chart> migration — responsive viewBox', () => {
  it('does not stretch with preserveAspectRatio="none"', async () => {
    const { el } = mount(reg, 'sui-line-chart', { series: TWO_SERIES })
    await flush()
    const svg = shadow(el).querySelector('svg[role="img"]')
    expect(svg?.getAttribute('preserveAspectRatio')).not.toBe('none')
  })
})
