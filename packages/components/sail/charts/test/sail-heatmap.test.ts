import { afterEach, describe, expect, it } from 'vitest'
import '../src/sail-heatmap/sail-heatmap.compiled.js'
import { createContainerRegistry, flush, mount, nonTokenPaints } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

// Light DOM: the component renders into the host element, so the host IS the
// query root — no shadow boundary, and no hand-authored classes to key on. The
// queries below are the component's real semantics: tags, ARIA, SVG paint.
const root = (el: HTMLElement): HTMLElement => el

describe('<sail-heatmap>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sail-heatmap')).toBeDefined()
  })

  it('renders into the light DOM (no shadow boundary)', async () => {
    const { el } = mount(reg, 'sail-heatmap', { rows: 'a', cols: 'x', data: '0.5' })
    await flush()
    expect(el.shadowRoot).toBeNull()
    expect(el.querySelector('svg[role="img"]')).not.toBeNull()
  })

  it('empty data composes a .sail-empty-state (full-size presentation profile)', async () => {
    // MIGRATED: empty data routes through the engine isEmpty discriminator to
    // .sail-empty-state, replacing the silent no-rect render.
    const { el } = mount(reg, 'sail-heatmap', { rows: 'a,b', cols: 'x,y' })
    await flush()
    expect(root(el).querySelector('.sail-empty-state')).not.toBeNull()
    expect(root(el).querySelector('svg[role="img"]')).toBeNull()
  })

  it('empty data renders no rect elements', async () => {
    const { el } = mount(reg, 'sail-heatmap', { rows: 'a,b', cols: 'x,y' })
    await flush()
    expect(root(el).querySelectorAll('rect').length).toBe(0)
  })

  it('1D data renders rows × cols rects', async () => {
    const { el } = mount(reg, 'sail-heatmap', {
      rows: 'a,b',
      cols: 'x,y,z',
      data: '0.1,0.5,0.9,0.2,0.6,1.0',
    })
    await flush()
    expect(root(el).querySelectorAll('rect').length).toBe(6)
  })

  it('2D JSON data renders rows × cols rects', async () => {
    const { el } = mount(reg, 'sail-heatmap', {
      rows: 'a,b',
      cols: 'x,y',
      data: '[[0.5,0.8],[0.2,0.9]]',
    })
    await flush()
    expect(root(el).querySelectorAll('rect').length).toBe(4)
  })

  it('SVG has role=img', async () => {
    const { el } = mount(reg, 'sail-heatmap', {
      rows: 'a',
      cols: 'x',
      data: '0.5',
    })
    await flush()
    expect(root(el).querySelector('svg')?.getAttribute('role')).toBe('img')
  })

  it('aria-label leads with the label prop (kind + data summary)', async () => {
    // MIGRATED: aria-label is now "kind: summary" (engine ariaSummary), not bare.
    const { el } = mount(reg, 'sail-heatmap', {
      rows: 'a',
      cols: 'x',
      data: '0.5',
      label: 'Activity grid',
    })
    await flush()
    const aria = root(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('Activity grid')).toBe(true)
  })

  it('default aria-label leads with "heatmap"', async () => {
    const { el } = mount(reg, 'sail-heatmap', { rows: 'a', cols: 'x', data: '0.5' })
    await flush()
    const aria = root(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('heatmap')).toBe(true)
  })

  it('tone reflects to host attribute', () => {
    const { el } = mount(reg, 'sail-heatmap', { tone: 'success' })
    expect(el.getAttribute('tone')).toBe('success')
  })

  it('data reflects to host attribute', () => {
    const { el } = mount(reg, 'sail-heatmap', { data: '0.1,0.2' })
    expect(el.getAttribute('data')).toBe('0.1,0.2')
  })

  it('one <rect> per cell, rounded to the sm corner', async () => {
    const { el } = mount(reg, 'sail-heatmap', {
      rows: 'a',
      cols: 'x',
      data: '0.5',
    })
    await flush()
    const cells = root(el).querySelectorAll('rect')
    expect(cells.length).toBe(1)
    expect(cells[0]?.getAttribute('rx')).toBe('2')
  })

  it('paints cells with the tone token (no color literal)', async () => {
    const { el } = mount(reg, 'sail-heatmap', {
      rows: 'a,b',
      cols: 'x,y',
      data: '0.1,0.5,0.9,1',
      tone: 'success',
    })
    await flush()
    expect(nonTokenPaints(root(el))).toEqual([])
    expect(root(el).querySelector('rect')?.getAttribute('fill')).toBe('var(--success)')
  })

  it('cells carry no color-mix style (tokenized intensity, not a color literal)', async () => {
    // MIGRATED: the controller-built `style="fill:color-mix(in oklch,…)"` smell is
    // GONE — intensity is driven by fill-opacity over var(--<tone>), the only
    // token-clean continuous-intensity path. No color-mix anywhere on a cell.
    const { el } = mount(reg, 'sail-heatmap', {
      rows: 'a',
      cols: 'x',
      data: '0.8',
    })
    await flush()
    const rect = root(el).querySelector('rect')
    expect(rect?.getAttribute('style') ?? '').not.toContain('color-mix')
    expect(rect?.outerHTML ?? '').not.toContain('color-mix')
  })

  it('non-zero cell intensity binds fill-opacity (proportional to normalized value)', async () => {
    // MIGRATED: a max-intensity cell renders at full fill-opacity over the tone
    // role token; intensity ∈ 0..1 from the normalized value — no color-mix.
    const { el } = mount(reg, 'sail-heatmap', {
      rows: 'a,b',
      cols: 'x',
      data: '0,1',
    })
    await flush()
    const rects = root(el).querySelectorAll('rect')
    // The max-valued cell (last) reaches full intensity (fill-opacity 1).
    const maxOpacity = Number(rects[rects.length - 1]?.getAttribute('fill-opacity'))
    expect(maxOpacity).toBe(1)
  })

  it('zero-value cell renders empty (fill-opacity 0)', async () => {
    // MIGRATED: the min/zero cell is the empty presentation — fill-opacity 0,
    // not an empty style override of a color-mix string.
    const { el } = mount(reg, 'sail-heatmap', {
      rows: 'a,b',
      cols: 'x',
      data: '0,1',
    })
    await flush()
    const rects = root(el).querySelectorAll('rect')
    expect(Number(rects[0]?.getAttribute('fill-opacity'))).toBe(0)
  })

  it('a null cell renders empty (fill-opacity 0)', async () => {
    // MIGRATED null-gap contract: a null heatmap cell renders the empty/zero
    // presentation (fill-opacity 0), same as a zero-value cell.
    const { el } = mount(reg, 'sail-heatmap', {
      rows: 'a',
      cols: 'x,y',
      data: '[[null,1]]',
    })
    await flush()
    const rects = root(el).querySelectorAll('rect')
    expect(Number(rects[0]?.getAttribute('fill-opacity'))).toBe(0)
  })

  it('cell fill uses the tone role token (var(--<tone>), no literal)', async () => {
    // MIGRATED: the cell fill is the tone role token — fill-opacity, not the hue,
    // carries intensity.
    const { el } = mount(reg, 'sail-heatmap', {
      rows: 'a',
      cols: 'x',
      data: '1',
      tone: 'success',
    })
    await flush()
    const rect = root(el).querySelector('rect')
    const fill = rect?.getAttribute('fill') ?? ''
    expect(fill).toContain('var(--')
    expect(fill).not.toContain('color-mix')
  })

  it('loading attr composes a sail-skeleton (full-size loading profile)', async () => {
    // MIGRATED: heatmap is full-size despite being compact — loading composes
    // sail-skeleton via the engine isLoading discriminator.
    const { el } = mount(reg, 'sail-heatmap', {
      rows: 'a',
      cols: 'x',
      data: '0.5',
      loading: true,
    })
    await flush()
    expect(root(el).querySelector('[skeleton]')).not.toBeNull()
    expect(root(el).querySelector('svg[role="img"]')).toBeNull()
  })

  it('loading reflects to host attribute', () => {
    const { el } = mount(reg, 'sail-heatmap', { loading: true })
    expect(el.hasAttribute('loading')).toBe(true)
  })

  it('reacts to data attribute change (tokenized intensity, no color-mix)', async () => {
    // MIGRATED: the rewritten characterization asserts fill-opacity intensity, not
    // a color-mix substring — color-mix is the smell being removed (Slice 0).
    const { el } = mount(reg, 'sail-heatmap', {
      rows: 'a',
      cols: 'x,y',
      data: '0,0',
    })
    await flush()
    expect(root(el).querySelectorAll('rect').length).toBe(2)
    el.setAttribute('data', '0.5,0.8')
    await flush()
    const rects = root(el).querySelectorAll('rect')
    expect(rects.length).toBe(2)
    // No cell carries a color-mix string; intensity is now non-zero somewhere.
    const noColorMix = Array.from(rects).every(
      (r) => !(r.getAttribute('style') ?? '').includes('color-mix'),
    )
    expect(noColorMix).toBe(true)
    const hasIntensity = Array.from(rects).some(
      (r) => Number(r.getAttribute('fill-opacity')) > 0,
    )
    expect(hasIntensity).toBe(true)
  })

  it('viewBox reflects the measured grid dimensions', async () => {
    const { el } = mount(reg, 'sail-heatmap', {
      rows: 'a,b',
      cols: 'x,y,z',
      data: '[[1,2,3],[4,5,6]]',
      'cell-size': '20',
      gap: '3',
    })
    await flush()
    // 3 cols × 2 rows grid → a non-degenerate viewBox is emitted.
    const vb = root(el).querySelector('svg')?.getAttribute('viewBox') ?? ''
    expect(vb).toMatch(/^0 0 \d/)
    expect(vb).not.toBe('0 0 0 0')
  })

  it('carries no inline style attribute (intensity is fill-opacity, not style=)', async () => {
    const { el } = mount(reg, 'sail-heatmap', { rows: 'a,b', cols: 'x,y', data: '1,2,3,4' })
    await flush()
    expect(root(el).querySelector('[style]')).toBeNull()
  })

  it('the grid box is cell-size driven (cells × size + gaps), not a stretched viewBox', async () => {
    const { el } = mount(reg, 'sail-heatmap', {
      rows: 'a,b',
      cols: 'x,y,z',
      data: '1,2,3,4,5,6',
      'cell-size': '20',
      gap: '3',
    })
    await flush()
    const svg = root(el).querySelector('svg')
    // 3 cols → 3*20 + 2*3 = 66 ; 2 rows → 2*20 + 1*3 = 43
    expect(svg?.getAttribute('width')).toBe('66')
    expect(svg?.getAttribute('height')).toBe('43')
    expect(svg?.getAttribute('viewBox')).toBe('0 0 66 43')
  })
})
