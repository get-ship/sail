import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-donut/sui-donut.compiled.js'
import { createContainerRegistry, flush, mount, nonTokenPaints } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

// Light DOM: the component renders into the host element, so the host IS the
// query root — no shadow boundary, and no hand-authored classes to key on. The
// queries below are the component's real semantics: tags, ARIA, SVG paint.
const root = (el: HTMLElement): HTMLElement => el

/** The unspent ring — the donut's one <circle>, stroked in the muted token. */
const track = (el: HTMLElement) => root(el).querySelector('svg circle')
/** The slices — the donut's arcs are the only <path> elements it draws. */
const segments = (el: HTMLElement) => root(el).querySelectorAll('svg path')

const seg = (label: string, value: number, tone?: string) =>
  JSON.stringify([{ label, value, tone }])

describe('<sui-donut>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-donut')).toBeDefined()
  })

  it('renders into the light DOM (no shadow boundary)', async () => {
    const { el } = mount(reg, 'sui-donut', { data: seg('a', 1) })
    await flush()
    expect(el.shadowRoot).toBeNull()
    expect(el.querySelector('svg[role="img"]')).not.toBeNull()
  })

  it('SVG has role=img', async () => {
    const { el } = mount(reg, 'sui-donut', { data: seg('a', 1) })
    await flush()
    expect(root(el).querySelector('svg')?.getAttribute('role')).toBe('img')
  })

  it('aria-label leads with the label prop (kind + data summary)', async () => {
    // MIGRATED: aria-label is now "kind: summary" (engine ariaSummary), not bare.
    const { el } = mount(reg, 'sui-donut', { data: seg('a', 1), label: 'Sales mix' })
    await flush()
    const aria = root(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('Sales mix')).toBe(true)
  })

  it('aria-label data summary names the slice count', async () => {
    // MIGRATED: ariaSummary derives the point/slice count from the engine extent.
    const data = JSON.stringify([
      { label: 'a', value: 3 },
      { label: 'b', value: 1 },
    ])
    const { el } = mount(reg, 'sui-donut', { data })
    await flush()
    const aria = root(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria).toContain('2 points')
  })

  it('default aria-label leads with "donut chart"', async () => {
    const { el } = mount(reg, 'sui-donut', { data: seg('a', 1) })
    await flush()
    const aria = root(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('donut chart')).toBe(true)
  })

  it('track ring always rendered for non-empty data, stroked in the muted token', async () => {
    const { el } = mount(reg, 'sui-donut', { data: seg('a', 1) })
    await flush()
    expect(track(el)).not.toBeNull()
    expect(track(el)?.getAttribute('stroke')).toBe('var(--muted)')
  })

  it('empty data composes a .sui-empty-state (full-size presentation profile)', async () => {
    // MIGRATED: empty data routes through the engine isEmpty discriminator to
    // .sui-empty-state, replacing the silent no-segment render.
    const { el } = mount(reg, 'sui-donut')
    await flush()
    expect(root(el).querySelector('.sui-empty-state')).not.toBeNull()
    expect(root(el).querySelector('svg[role="img"]')).toBeNull()
  })

  it('empty data renders no segment paths', async () => {
    const { el } = mount(reg, 'sui-donut')
    await flush()
    expect(root(el).querySelectorAll('path').length).toBe(0)
  })

  it('renders one path per positive-value segment', async () => {
    const data = JSON.stringify([
      { label: 'a', value: 3, tone: 'info' },
      { label: 'b', value: 1, tone: 'success' },
      { label: 'c', value: 2, tone: 'warning' },
    ])
    const { el } = mount(reg, 'sui-donut', { data })
    await flush()
    expect(segments(el).length).toBe(3)
  })

  it('a null-valued slice is skipped (contributes no arc)', async () => {
    // MIGRATED null-gap contract: a null donut value skips that slice entirely —
    // the two finite slices render, the null one contributes nothing.
    const data = JSON.stringify([
      { label: 'a', value: 5 },
      { label: 'b', value: null },
      { label: 'c', value: 3 },
    ])
    const { el } = mount(reg, 'sui-donut', { data })
    await flush()
    const segs = segments(el)
    expect(segs.length).toBe(2)
    const titles = Array.from(segs).map((p) => p.querySelector('title')?.textContent ?? '')
    expect(titles.some((t) => t.includes('a'))).toBe(true)
    expect(titles.some((t) => t.includes('c'))).toBe(true)
  })

  it('single 100%-value segment renders one path', async () => {
    const { el } = mount(reg, 'sui-donut', { data: seg('only', 10) })
    await flush()
    expect(segments(el).length).toBe(1)
  })

  it('each segment path has a title child', async () => {
    const data = JSON.stringify([
      { label: 'x', value: 2 },
      { label: 'y', value: 3 },
    ])
    const { el } = mount(reg, 'sui-donut', { data })
    await flush()
    const paths = segments(el)
    for (const p of paths) {
      expect(p.querySelector('title')).not.toBeNull()
    }
  })

  it('loading attr composes a sui-skeleton (full-size loading profile)', async () => {
    // MIGRATED: the engine isLoading discriminator drives the loading branch.
    const { el } = mount(reg, 'sui-donut', { data: seg('a', 1), loading: true })
    await flush()
    expect(root(el).querySelector('[skeleton]')).not.toBeNull()
    expect(root(el).querySelector('svg[role="img"]')).toBeNull()
  })

  it('loading reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-donut', { loading: true })
    expect(el.hasAttribute('loading')).toBe(true)
  })

  it('data reflects to host attribute', () => {
    const d = seg('a', 1)
    const { el } = mount(reg, 'sui-donut', { data: d })
    expect(el.getAttribute('data')).toBe(d)
  })

  it('reacts to data attribute change', async () => {
    const { el } = mount(reg, 'sui-donut', {
      data: JSON.stringify([{ label: 'a', value: 1 }]),
    })
    await flush()
    expect(segments(el).length).toBe(1)
    el.setAttribute(
      'data',
      JSON.stringify([
        { label: 'a', value: 1 },
        { label: 'b', value: 2 },
        { label: 'c', value: 3 },
      ]),
    )
    await flush()
    expect(segments(el).length).toBe(3)
  })

  it('slot renders center content over the ring', async () => {
    const { el } = mount(reg, 'sui-donut', { data: seg('a', 5) }, '42')
    await flush()
    const slot = root(el).querySelector('slot:not([name])')
    expect(slot).not.toBeNull()
    expect(slot?.textContent?.trim()).toBe('42')
  })

  it('paints ring and slices with design tokens (no color literal)', async () => {
    const data = JSON.stringify([
      { label: 'a', value: 3, tone: 'info' },
      { label: 'b', value: 1, tone: 'success' },
    ])
    const { el } = mount(reg, 'sui-donut', { data })
    await flush()
    expect(nonTokenPaints(root(el))).toEqual([])
  })

  it('carries no inline style attribute — diameter drives the SVG box', async () => {
    const { el } = mount(reg, 'sui-donut', { data: seg('a', 5), diameter: '160' })
    await flush()
    expect(root(el).querySelector('[style]')).toBeNull()
    const svg = root(el).querySelector('svg')
    expect(svg?.getAttribute('width')).toBe('160')
    expect(svg?.getAttribute('height')).toBe('160')
    expect(svg?.getAttribute('viewBox')).toBe('0 0 160 160')
  })

  it('diameter drives the ring geometry (a real number, not the size channel)', async () => {
    const { el } = mount(reg, 'sui-donut', { data: seg('a', 5), diameter: '200' })
    await flush()
    expect(track(el)?.getAttribute('r')).toBe('92')
  })
})
