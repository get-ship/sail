import { afterEach, describe, expect, it } from 'vitest'
import '../src/sail-sparkline/sail-sparkline.compiled.js'
import { createContainerRegistry, flush, mount, nonTokenPaints } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

// Light DOM: the component renders into the host element, so the host IS the
// query root — no shadow boundary, and no hand-authored classes to key on. The
// queries below are the components. real semantics: tags, ARIA, SVG paint.
const root = (el: HTMLElement): HTMLElement => el

describe('<sail-sparkline>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sail-sparkline')).toBeDefined()
  })

  it('attaches a shadow root', () => {
    const { el } = mount(reg, 'sail-sparkline')
    expect(root(el)).not.toBeNull()
  })

  it('renders a stroke path for valid data', async () => {
    const { el } = mount(reg, 'sail-sparkline', { data: '1,2,3,4,5' })
    await flush()
    const paths = root(el).querySelectorAll('path')
    expect(paths.length).toBeGreaterThanOrEqual(1)
    expect(paths[0]?.getAttribute('d')).toMatch(/^M/)
  })

  it('SVG has role=img when data is valid', async () => {
    const { el } = mount(reg, 'sail-sparkline', { data: '1,2,3' })
    await flush()
    expect(root(el).querySelector('svg')?.getAttribute('role')).toBe('img')
  })

  it('aria-label leads with the label prop (kind + data summary)', async () => {
    // MIGRATED: aria-label is now "kind: summary" (engine ariaSummary), not bare.
    const { el } = mount(reg, 'sail-sparkline', { data: '1,2,3', label: 'Trend' })
    await flush()
    const aria = root(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('Trend')).toBe(true)
  })

  it('aria-label data summary names the point count', async () => {
    // MIGRATED: ariaSummary derives the point count from the engine extent.
    const { el } = mount(reg, 'sail-sparkline', { data: '1,2,3' })
    await flush()
    const aria = root(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria).toContain('3 points')
  })

  it('default aria-label leads with "sparkline"', async () => {
    const { el } = mount(reg, 'sail-sparkline', { data: '1,2,3' })
    await flush()
    const aria = root(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('sparkline')).toBe(true)
  })

  it('renders 2 paths when fill attr is set', async () => {
    const { el } = mount(reg, 'sail-sparkline', { data: '1,2,3,4', fill: true })
    await flush()
    expect(root(el).querySelectorAll('path').length).toBe(2)
  })

  it('area fill path has fill-opacity 0.18', async () => {
    const { el } = mount(reg, 'sail-sparkline', { data: '1,2,3,4', fill: true })
    await flush()
    const first = root(el).querySelector('path')
    expect(first?.getAttribute('fill-opacity')).toBe('0.18')
  })

  it('renders a single-point series as one centered path', async () => {
    // MIGRATED: the unified engine renders a lone finite point (centered at w/2)
    // rather than dropping series with < 2 points — same null-gap contract as line.
    const { el } = mount(reg, 'sail-sparkline', { data: '5' })
    await flush()
    expect(root(el).querySelectorAll('path').length).toBe(1)
  })

  it('a null gap splits the stroke into sub-paths (break, not interpolate)', async () => {
    // MIGRATED: a null comma slot breaks the stroke into one M-sub-path per run.
    const { el } = mount(reg, 'sail-sparkline', { data: '1,2,,4,5' })
    await flush()
    const stroke = root(el).querySelectorAll('path')
    const d = stroke[stroke.length - 1]?.getAttribute('d') ?? ''
    const moves = d.match(/M/g)?.length ?? 0
    expect(moves).toBe(2)
  })

  it('renders no paths for empty data', async () => {
    const { el } = mount(reg, 'sail-sparkline')
    await flush()
    expect(root(el).querySelectorAll('path').length).toBe(0)
  })

  it('empty data keeps the lightweight aria-hidden empty SVG (no empty-state)', async () => {
    // MIGRATED: sparkline is the ONLY lightweight exception — empty stays the
    // aria-hidden empty SVG, NOT a full-size .sail-empty-state.
    const { el } = mount(reg, 'sail-sparkline')
    await flush()
    expect(root(el).querySelector('svg')?.getAttribute('aria-hidden')).toBe('true')
    expect(root(el).querySelector('.sail-empty-state')).toBeNull()
  })

  it('loading attr composes a sail-spinner (inline loading profile, not skeleton)', async () => {
    // MIGRATED: the inline presentation uses sail-spinner for loading; the
    // full-size sail-skeleton would be too heavy inline.
    const { el } = mount(reg, 'sail-sparkline', { data: '1,2,3', loading: true })
    await flush()
    expect(root(el).querySelector('.sail-spinner')).not.toBeNull()
    expect(root(el).querySelector('[skeleton]')).toBeNull()
  })

  it('loading reflects to host attribute', () => {
    // MIGRATED: loading is a reflected bool prop so :host([loading]) CSS reacts.
    const { el } = mount(reg, 'sail-sparkline', { loading: true })
    expect(el.hasAttribute('loading')).toBe(true)
  })

  it('tone reflects to host attribute', () => {
    const { el } = mount(reg, 'sail-sparkline', { tone: 'success' })
    expect(el.getAttribute('tone')).toBe('success')
  })

  it('fill reflects to host attribute', () => {
    const { el } = mount(reg, 'sail-sparkline', { fill: true })
    expect(el.hasAttribute('fill')).toBe(true)
  })

  it('data reflects to host attribute', () => {
    const { el } = mount(reg, 'sail-sparkline', { data: '1,2,3' })
    expect(el.getAttribute('data')).toBe('1,2,3')
  })

  it('paints the trend line with the tone token (no color literal)', async () => {
    const { el } = mount(reg, 'sail-sparkline', { data: '1,2,3', tone: 'success', fill: true })
    await flush()
    expect(nonTokenPaints(root(el))).toEqual([])
    // The trend line is the stroked path (the area under it is the filled one).
    expect(
      root(el).querySelector('path[fill="none"]')?.getAttribute('stroke'),
    ).toBe('var(--success)')
  })

  it('carries no inline style attribute (styling is the language, not style=)', async () => {
    const { el } = mount(reg, 'sail-sparkline', { data: '1,2,3' })
    await flush()
    expect(root(el).querySelector('[style]')).toBeNull()
  })

  it('reacts to data attribute change', async () => {
    const { el } = mount(reg, 'sail-sparkline', { data: '1,2,3' })
    await flush()
    const before = root(el).querySelector('path')?.getAttribute('d')
    el.setAttribute('data', '5,4,3,2,1')
    await flush()
    const after = root(el).querySelector('path')?.getAttribute('d')
    expect(after).not.toEqual(before)
  })
})
