import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-area-chart/sui-area-chart.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-area-chart>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-area-chart')).toBeDefined()
  })

  it('attaches a shadow root', () => {
    const { el } = mount(reg, 'sui-area-chart')
    expect(shadow(el)).not.toBeNull()
  })

  it('SVG has role=img for non-empty data', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2] }])
    const { el } = mount(reg, 'sui-area-chart', { series })
    await flush()
    expect(shadow(el).querySelector('svg')?.getAttribute('role')).toBe('img')
  })

  it('aria-label leads with the label prop (kind + data summary)', async () => {
    // MIGRATED: aria-label is now "kind: summary" (engine ariaSummary), not the
    // bare label — the data is perceivable, not just the chart kind.
    const series = JSON.stringify([{ name: 'a', data: [1, 2] }])
    const { el } = mount(reg, 'sui-area-chart', { series, label: 'Revenue' })
    await flush()
    const aria = shadow(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('Revenue')).toBe(true)
  })

  it('aria-label data summary names the series + point count', async () => {
    // MIGRATED: ariaSummary derives "N series, M points, range lo–hi" from the
    // engine extent — the underlying numbers are described, not just the kind.
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sui-area-chart', { series })
    await flush()
    const aria = shadow(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria).toContain('1 series')
    expect(aria).toContain('3 points')
  })

  it('default aria-label leads with "area chart"', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2] }])
    const { el } = mount(reg, 'sui-area-chart', { series })
    await flush()
    const aria = shadow(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('area chart')).toBe(true)
  })

  it('renders 2 paths per series (area fill + stroke line)', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 3, 2, 4] }])
    const { el } = mount(reg, 'sui-area-chart', { series })
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(2)
  })

  it('renders 4 paths for 2 series', async () => {
    const series = JSON.stringify([
      { name: 'a', data: [1, 2, 3] },
      { name: 'b', data: [3, 2, 1] },
    ])
    const { el } = mount(reg, 'sui-area-chart', { series })
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(4)
  })

  it('first path (area fill) has fill-opacity 0.3', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sui-area-chart', { series })
    await flush()
    const firstPath = shadow(el).querySelector('path')
    expect(firstPath?.getAttribute('fill-opacity')).toBe('0.3')
  })

  it('stroke path exposes part="series"', async () => {
    // MIGRATED: series paths carry part="series" so consumers theme without forking.
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sui-area-chart', { series })
    await flush()
    expect(shadow(el).querySelector('path[part="series"]')).not.toBeNull()
  })

  it('renders circles when points attr is set', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sui-area-chart', { series, points: true })
    await flush()
    expect(shadow(el).querySelectorAll('circle').length).toBe(3)
  })

  it('renders 3 grid lines when grid attr is set', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sui-area-chart', { series, grid: true })
    await flush()
    expect(shadow(el).querySelectorAll('line.grid-line').length).toBe(3)
  })

  it('grid lines expose part="grid"', async () => {
    // MIGRATED: grid lines carry part="grid" (the new themeable seam).
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sui-area-chart', { series, grid: true })
    await flush()
    expect(shadow(el).querySelector('line[part="grid"]')).not.toBeNull()
  })

  it('no grid lines rendered without grid attr', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sui-area-chart', { series })
    await flush()
    expect(shadow(el).querySelectorAll('line.grid-line').length).toBe(0)
  })

  it('renders a single-point series as one centered area', async () => {
    // MIGRATED: the unified engine renders a lone finite point (centered at w/2)
    // rather than silently dropping series with < 2 points — consistent with the
    // null-gap contract (a single-finite run is a centered point, not nothing).
    const series = JSON.stringify([{ name: 'a', data: [5] }])
    const { el } = mount(reg, 'sui-area-chart', { series })
    await flush()
    expect(shadow(el).querySelectorAll('path[part="series"]').length).toBe(1)
  })

  it('a null gap splits the stroke into sub-paths (break, not interpolate)', async () => {
    // MIGRATED: a null point breaks the stroke into one M-sub-path per contiguous
    // finite run rather than interpolating or zeroing the gap.
    const series = JSON.stringify([{ name: 'a', data: [1, 2, null, 4, 5] }])
    const { el } = mount(reg, 'sui-area-chart', { series })
    await flush()
    const stroke = shadow(el).querySelector('path[part="series"]')?.getAttribute('d') ?? ''
    const moves = stroke.match(/M/g)?.length ?? 0
    expect(moves).toBe(2)
  })

  it('tolerates bad JSON and renders an empty state (no spurious paths)', async () => {
    // MIGRATED: bad data routes through the engine isEmpty discriminator to
    // .sui-empty-state — no silent zero-path SVG.
    const { el } = mount(reg, 'sui-area-chart', { series: 'not-json' })
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(0)
    expect(shadow(el).querySelector('.sui-empty-state')).not.toBeNull()
  })

  it('empty data composes a .sui-empty-state (no bare data SVG)', async () => {
    // MIGRATED: empty data routes to .sui-empty-state per the full-size
    // presentation profile, replacing the silent zero-path render.
    const { el } = mount(reg, 'sui-area-chart')
    await flush()
    expect(shadow(el).querySelector('.sui-empty-state')).not.toBeNull()
    expect(shadow(el).querySelector('svg[role="img"]')).toBeNull()
  })

  it('loading attr composes a sui-skeleton (full-size loading profile)', async () => {
    // MIGRATED: the engine isLoading discriminator drives the loading branch.
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sui-area-chart', { series, loading: true })
    await flush()
    expect(shadow(el).querySelector('[skeleton]')).not.toBeNull()
    expect(shadow(el).querySelector('svg[role="img"]')).toBeNull()
  })

  it('loading reflects to host attribute', () => {
    // MIGRATED: loading is a reflected bool prop so :host([loading]) CSS reacts.
    const { el } = mount(reg, 'sui-area-chart', { loading: true })
    expect(el.hasAttribute('loading')).toBe(true)
  })

  it('points reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-area-chart', { points: true })
    expect(el.hasAttribute('points')).toBe(true)
  })

  it('grid reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-area-chart', { grid: true })
    expect(el.hasAttribute('grid')).toBe(true)
  })

  it('series reflects to host attribute', () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2] }])
    const { el } = mount(reg, 'sui-area-chart', { series })
    expect(el.getAttribute('series')).toBe(series)
  })

  it('slot for legend is present in shadow DOM', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2] }])
    const { el } = mount(reg, 'sui-area-chart', { series })
    await flush()
    expect(shadow(el).querySelector('slot')).not.toBeNull()
  })

  it('exposes a described-data table referenced by aria-describedby', async () => {
    // MIGRATED: the visually-hidden ariaTable backs aria-describedby, a sibling of
    // the role="img" SVG in the SAME shadow root (not pruned, not cross-root).
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sui-area-chart', { series })
    await flush()
    const svg = shadow(el).querySelector('svg[role="img"]')
    const id = svg?.getAttribute('aria-describedby')
    expect(id).toBeTruthy()
    const described = shadow(el).getElementById(id!)
    expect(described).not.toBeNull()
    expect(described?.querySelector('table')).not.toBeNull()
    // The table must NOT be a descendant of the role="img" SVG (a11y-tree pruning).
    expect(svg?.contains(described!)).toBe(false)
  })

  it('shadow styles include prefers-reduced-motion rule', () => {
    const { el } = mount(reg, 'sui-area-chart')
    const styleEl = shadow(el).querySelector('style')
    expect(styleEl?.textContent).toContain('prefers-reduced-motion')
  })

  it('shadow <style> carries no color literal (token-clean)', () => {
    // MIGRATED: token purity — no raw hex/rgb/oklch/color-mix in the style block.
    const { el } = mount(reg, 'sui-area-chart')
    const css = shadow(el).querySelector('style')?.textContent ?? ''
    expect(css).not.toMatch(/#[0-9a-fA-F]{3,8}\b/)
    expect(css).not.toMatch(/\brgba?\(/)
    expect(css).not.toMatch(/\boklch\(/)
    expect(css).not.toContain('color-mix')
  })

  it('reacts to series attribute change', async () => {
    const series1 = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const series2 = JSON.stringify([
      { name: 'a', data: [1, 2, 3] },
      { name: 'b', data: [3, 2, 1] },
    ])
    const { el } = mount(reg, 'sui-area-chart', { series: series1 })
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(2)
    el.setAttribute('series', series2)
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(4)
  })
})
