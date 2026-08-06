import { afterEach, describe, expect, it } from 'vitest'
import '../src/sail-area-chart/sail-area-chart.compiled.js'
import { createContainerRegistry, flush, mount, nonTokenPaints } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

// Light DOM: the component renders into the host element, so the host IS the
// query root — no shadow boundary, and no hand-authored classes to key on. The
// queries below are the component's real semantics: tags, ARIA, SVG paint.
const root = (el: HTMLElement): HTMLElement => el

/** The stroke line of a series — the only paths drawn with `fill="none"`. */
const strokes = (el: HTMLElement) => root(el).querySelectorAll('svg path[fill="none"]')
/** The area under a series — the only paths that carry a fill opacity. */
const fills = (el: HTMLElement) => root(el).querySelectorAll('svg path[fill-opacity]')
/** Y-grid reference lines — the only dashed elements in the plot. */
const gridLines = (el: HTMLElement) => root(el).querySelectorAll('svg line[stroke-dasharray]')

describe('<sail-area-chart>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sail-area-chart')).toBeDefined()
  })

  it('renders into the light DOM (no shadow boundary)', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2] }])
    const { el } = mount(reg, 'sail-area-chart', { series })
    await flush()
    expect(el.shadowRoot).toBeNull()
    expect(el.querySelector('svg[role="img"]')).not.toBeNull()
  })

  it('SVG has role=img for non-empty data', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2] }])
    const { el } = mount(reg, 'sail-area-chart', { series })
    await flush()
    expect(root(el).querySelector('svg')?.getAttribute('role')).toBe('img')
  })

  it('aria-label leads with the label prop (kind + data summary)', async () => {
    // MIGRATED: aria-label is now "kind: summary" (engine ariaSummary), not the
    // bare label — the data is perceivable, not just the chart kind.
    const series = JSON.stringify([{ name: 'a', data: [1, 2] }])
    const { el } = mount(reg, 'sail-area-chart', { series, label: 'Revenue' })
    await flush()
    const aria = root(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('Revenue')).toBe(true)
  })

  it('aria-label data summary names the series + point count', async () => {
    // MIGRATED: ariaSummary derives "N series, M points, range lo–hi" from the
    // engine extent — the underlying numbers are described, not just the kind.
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sail-area-chart', { series })
    await flush()
    const aria = root(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria).toContain('1 series')
    expect(aria).toContain('3 points')
  })

  it('default aria-label leads with "area chart"', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2] }])
    const { el } = mount(reg, 'sail-area-chart', { series })
    await flush()
    const aria = root(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('area chart')).toBe(true)
  })

  it('renders 2 paths per series (area fill + stroke line)', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 3, 2, 4] }])
    const { el } = mount(reg, 'sail-area-chart', { series })
    await flush()
    expect(root(el).querySelectorAll('path').length).toBe(2)
  })

  it('renders 4 paths for 2 series', async () => {
    const series = JSON.stringify([
      { name: 'a', data: [1, 2, 3] },
      { name: 'b', data: [3, 2, 1] },
    ])
    const { el } = mount(reg, 'sail-area-chart', { series })
    await flush()
    expect(root(el).querySelectorAll('path').length).toBe(4)
  })

  it('first path (area fill) has fill-opacity 0.3', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sail-area-chart', { series })
    await flush()
    expect(fills(el)[0]?.getAttribute('fill-opacity')).toBe('0.3')
  })

  it('the series stroke is painted with a tone token, never a color literal', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3], tone: 'success' }])
    const { el } = mount(reg, 'sail-area-chart', { series })
    await flush()
    expect(strokes(el)[0]?.getAttribute('stroke')).toBe('var(--success)')
  })

  it('renders circles when points attr is set', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sail-area-chart', { series, points: true })
    await flush()
    expect(root(el).querySelectorAll('circle').length).toBe(3)
  })

  it('renders 3 grid lines when grid attr is set', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sail-area-chart', { series, grid: true })
    await flush()
    expect(gridLines(el).length).toBe(3)
  })

  it('grid lines are hairline dashes in the border token', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sail-area-chart', { series, grid: true })
    await flush()
    const line = gridLines(el)[0]
    expect(line?.getAttribute('stroke')).toBe('var(--border)')
    expect(line?.getAttribute('stroke-width')).toBe('1')
  })

  it('no grid lines rendered without grid attr', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sail-area-chart', { series })
    await flush()
    expect(gridLines(el).length).toBe(0)
  })

  it('renders a single-point series as one centered area', async () => {
    // MIGRATED: the unified engine renders a lone finite point (centered at w/2)
    // rather than silently dropping series with < 2 points — consistent with the
    // null-gap contract (a single-finite run is a centered point, not nothing).
    const series = JSON.stringify([{ name: 'a', data: [5] }])
    const { el } = mount(reg, 'sail-area-chart', { series })
    await flush()
    expect(strokes(el).length).toBe(1)
  })

  it('a null gap splits the stroke into sub-paths (break, not interpolate)', async () => {
    // MIGRATED: a null point breaks the stroke into one M-sub-path per contiguous
    // finite run rather than interpolating or zeroing the gap.
    const series = JSON.stringify([{ name: 'a', data: [1, 2, null, 4, 5] }])
    const { el } = mount(reg, 'sail-area-chart', { series })
    await flush()
    const stroke = strokes(el)[0]?.getAttribute('d') ?? ''
    const moves = stroke.match(/M/g)?.length ?? 0
    expect(moves).toBe(2)
  })

  it('tolerates bad JSON and renders an empty state (no spurious paths)', async () => {
    // MIGRATED: bad data routes through the engine isEmpty discriminator to
    // .sail-empty-state — no silent zero-path SVG.
    const { el } = mount(reg, 'sail-area-chart', { series: 'not-json' })
    await flush()
    expect(root(el).querySelectorAll('path').length).toBe(0)
    expect(root(el).querySelector('.sail-empty-state')).not.toBeNull()
  })

  it('empty data composes a .sail-empty-state (no bare data SVG)', async () => {
    // MIGRATED: empty data routes to .sail-empty-state per the full-size
    // presentation profile, replacing the silent zero-path render.
    const { el } = mount(reg, 'sail-area-chart')
    await flush()
    expect(root(el).querySelector('.sail-empty-state')).not.toBeNull()
    expect(root(el).querySelector('svg[role="img"]')).toBeNull()
  })

  it('loading attr composes a sail-skeleton (full-size loading profile)', async () => {
    // MIGRATED: the engine isLoading discriminator drives the loading branch.
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sail-area-chart', { series, loading: true })
    await flush()
    expect(root(el).querySelector('.sail-skeleton')).not.toBeNull()
    expect(root(el).querySelector('svg[role="img"]')).toBeNull()
  })

  it('loading reflects to host attribute', () => {
    // MIGRATED: loading is a reflected bool prop so :host([loading]) CSS reacts.
    const { el } = mount(reg, 'sail-area-chart', { loading: true })
    expect(el.hasAttribute('loading')).toBe(true)
  })

  it('points reflects to host attribute', () => {
    const { el } = mount(reg, 'sail-area-chart', { points: true })
    expect(el.hasAttribute('points')).toBe(true)
  })

  it('grid reflects to host attribute', () => {
    const { el } = mount(reg, 'sail-area-chart', { grid: true })
    expect(el.hasAttribute('grid')).toBe(true)
  })

  it('series reflects to host attribute', () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2] }])
    const { el } = mount(reg, 'sail-area-chart', { series })
    expect(el.getAttribute('series')).toBe(series)
  })

  it('slot for legend is present', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2] }])
    const { el } = mount(reg, 'sail-area-chart', { series })
    await flush()
    expect(root(el).querySelector('slot')).not.toBeNull()
  })

  it('exposes a described-data table referenced by aria-describedby', async () => {
    // The visually-hidden ariaTable backs aria-describedby: a SIBLING of the
    // role="img" SVG (inside it, the a11y tree would prune it away).
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sail-area-chart', { series })
    await flush()
    const svg = root(el).querySelector('svg[role="img"]')
    const id = svg?.getAttribute('aria-describedby')
    expect(id).toBeTruthy()
    const described = root(el).querySelector(`#${id!}`)
    expect(described).not.toBeNull()
    expect(described?.querySelector('table')).not.toBeNull()
    // The table must NOT be a descendant of the role="img" SVG (a11y-tree pruning).
    expect(svg?.contains(described!)).toBe(false)
  })

  it('paints every mark with a design token (no color literal)', async () => {
    const series = JSON.stringify([
      { name: 'a', data: [1, 2, 3], tone: 'info' },
      { name: 'b', data: [3, 2, 1], tone: 'warning' },
    ])
    const { el } = mount(reg, 'sail-area-chart', { series, grid: true, points: true })
    await flush()
    expect(nonTokenPaints(root(el))).toEqual([])
  })

  it('carries no inline style attribute (styling is the language, not style=)', async () => {
    const series = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const { el } = mount(reg, 'sail-area-chart', { series })
    await flush()
    expect(root(el).querySelector('[style]')).toBeNull()
  })

  it('reacts to series attribute change', async () => {
    const series1 = JSON.stringify([{ name: 'a', data: [1, 2, 3] }])
    const series2 = JSON.stringify([
      { name: 'a', data: [1, 2, 3] },
      { name: 'b', data: [3, 2, 1] },
    ])
    const { el } = mount(reg, 'sail-area-chart', { series: series1 })
    await flush()
    expect(root(el).querySelectorAll('path').length).toBe(2)
    el.setAttribute('series', series2)
    await flush()
    expect(root(el).querySelectorAll('path').length).toBe(4)
  })
})
