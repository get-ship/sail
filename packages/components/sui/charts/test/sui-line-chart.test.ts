import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-line-chart/sui-line-chart.compiled.js'
import { createContainerRegistry, flush, mount, nonTokenPaints } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

// Light DOM: the component renders into the host element, so the host IS the
// query root — no shadow boundary, and no hand-authored classes to key on. The
// queries below are the component's real semantics: tags, ARIA, SVG paint.
const root = (el: HTMLElement): HTMLElement => el

/** Y-grid reference lines — the only dashed elements in the plot. */
const gridLines = (el: HTMLElement) => root(el).querySelectorAll('svg line[stroke-dasharray]')

describe('<sui-line-chart>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-line-chart')).toBeDefined()
  })

  it('renders into the light DOM (no shadow boundary)', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2] }])
    const { el } = mount(reg, 'sui-line-chart', { series })
    await flush()
    expect(el.shadowRoot).toBeNull()
    expect(el.querySelector('svg[role="img"]')).not.toBeNull()
  })

  it('renders one path per valid series', async () => {
    const series = JSON.stringify([
      { name: 'a', data: [1, 2, 3, 4] },
      { name: 'b', data: [4, 3, 2, 1] },
    ])
    const { el } = mount(reg, 'sui-line-chart', { series })
    await flush()
    expect(root(el).querySelectorAll('path').length).toBe(2)
  })

  it('SVG has role=img', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2] }])
    const { el } = mount(reg, 'sui-line-chart', { series })
    await flush()
    expect(root(el).querySelector('svg')?.getAttribute('role')).toBe('img')
  })

  it('aria-label leads with the label prop (kind + data summary)', async () => {
    // MIGRATED: aria-label is now "kind: summary" (engine ariaSummary), not the
    // bare label — the data is perceivable, not just the chart kind.
    const series = JSON.stringify([{ name: 'a', data: [1, 2] }])
    const { el } = mount(reg, 'sui-line-chart', { series, label: 'Activity' })
    await flush()
    const aria = root(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('Activity')).toBe(true)
  })

  it('default aria-label leads with "line chart"', async () => {
    // MIGRATED: empty data renders a sui-empty-state (no data SVG); a populated
    // default chart leads its aria-label with the "line chart" kind.
    const series = JSON.stringify([{ name: 'a', data: [1, 2] }])
    const { el } = mount(reg, 'sui-line-chart', { series })
    await flush()
    const aria = root(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('line chart')).toBe(true)
  })

  it('renders circles when points attr is set', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sui-line-chart', { series, points: true })
    await flush()
    expect(root(el).querySelectorAll('circle').length).toBe(3)
  })

  it('renders 3 grid lines when grid attr is set', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sui-line-chart', { series, grid: true })
    await flush()
    expect(gridLines(el).length).toBe(3)
  })

  it('no grid lines rendered without grid attr', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sui-line-chart', { series })
    await flush()
    expect(gridLines(el).length).toBe(0)
  })

  it('tolerates bad JSON and renders no paths', async () => {
    const { el } = mount(reg, 'sui-line-chart', { series: 'not-json' })
    await flush()
    expect(root(el).querySelectorAll('path').length).toBe(0)
  })

  it('renders a single-point series as one centered path', async () => {
    // MIGRATED: the unified engine renders a lone finite point (centered at w/2)
    // rather than silently dropping series with < 2 points — consistent with the
    // null-gap contract (a single-finite run is a centered point, not nothing).
    const series = JSON.stringify([{ name: 'a', data: [5] }])
    const { el } = mount(reg, 'sui-line-chart', { series })
    await flush()
    expect(root(el).querySelectorAll('path').length).toBe(1)
  })

  it('points reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-line-chart', { points: true })
    expect(el.hasAttribute('points')).toBe(true)
  })

  it('grid reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-line-chart', { grid: true })
    expect(el.hasAttribute('grid')).toBe(true)
  })

  it('slot for legend is present', async () => {
    const { el } = mount(reg, 'sui-line-chart')
    await flush()
    expect(root(el).querySelector('slot')).not.toBeNull()
  })

  it('paints every mark with a design token (no color literal)', async () => {
    const series = JSON.stringify([
      { name: 'a', data: [1, 2, 3], tone: 'info' },
      { name: 'b', data: [3, 2, 1], tone: 'warning' },
    ])
    const { el } = mount(reg, 'sui-line-chart', { series, grid: true, points: true })
    await flush()
    expect(nonTokenPaints(root(el))).toEqual([])
  })

  it('carries no inline style attribute (styling is the language, not style=)', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sui-line-chart', { series })
    await flush()
    expect(root(el).querySelector('[style]')).toBeNull()
  })

  it('grid lines are hairline dashes in the border token', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sui-line-chart', { series, grid: true })
    await flush()
    const line = gridLines(el)[0]
    expect(line?.getAttribute('stroke')).toBe('var(--border)')
    expect(line?.getAttribute('stroke-width')).toBe('1')
  })

  it('reacts to series attribute change', async () => {
    const series1 = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const series2 = JSON.stringify([
      { name: 'a', data: [1, 2, 3] },
      { name: 'b', data: [3, 2, 1] },
    ])
    const { el } = mount(reg, 'sui-line-chart', { series: series1 })
    await flush()
    expect(root(el).querySelectorAll('path').length).toBe(1)
    el.setAttribute('series', series2)
    await flush()
    expect(root(el).querySelectorAll('path').length).toBe(2)
  })
})
