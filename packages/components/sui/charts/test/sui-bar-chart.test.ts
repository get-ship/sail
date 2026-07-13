import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-bar-chart/sui-bar-chart.compiled.js'
import { createContainerRegistry, flush, mount, nonTokenPaints } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

// Light DOM: the component renders into the host element, so the host IS the
// query root — no shadow boundary, and no hand-authored classes to key on. The
// queries below are the component's real semantics: tags, ARIA, SVG paint.
const root = (el: HTMLElement): HTMLElement => el

describe('<sui-bar-chart>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-bar-chart')).toBeDefined()
  })

  it('renders into the light DOM (no shadow boundary)', async () => {
    const { el } = mount(reg, 'sui-bar-chart', { data: '1,2,3' })
    await flush()
    expect(el.shadowRoot).toBeNull()
    expect(el.querySelector('svg[role="img"]')).not.toBeNull()
  })

  it('renders a rect per datum', async () => {
    const { el } = mount(reg, 'sui-bar-chart', { data: '1,2,3,4' })
    await flush()
    expect(root(el).querySelectorAll('rect').length).toBe(4)
  })

  it('SVG has role=img for non-empty data', async () => {
    const { el } = mount(reg, 'sui-bar-chart', { data: '1,2,3' })
    await flush()
    expect(root(el).querySelector('svg')?.getAttribute('role')).toBe('img')
  })

  it('aria-label leads with the label prop (kind + data summary)', async () => {
    // MIGRATED: aria-label is now "kind: summary" (engine ariaSummary), not bare.
    const { el } = mount(reg, 'sui-bar-chart', { data: '1,2', label: 'Sales' })
    await flush()
    const aria = root(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('Sales')).toBe(true)
  })

  it('default aria-label leads with "bar chart"', async () => {
    const { el } = mount(reg, 'sui-bar-chart', { data: '1,2' })
    await flush()
    const aria = root(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('bar chart')).toBe(true)
  })

  it('empty data composes a .sui-empty-state (no bare aria-hidden SVG)', async () => {
    // MIGRATED: the engine discriminator routes empty data to .sui-empty-state,
    // replacing the ad-hoc aria-hidden empty SVG.
    const { el } = mount(reg, 'sui-bar-chart')
    await flush()
    expect(root(el).querySelector('.sui-empty-state')).not.toBeNull()
    expect(root(el).querySelector('svg[role="img"]')).toBeNull()
  })

  it('empty data renders no rect elements', async () => {
    const { el } = mount(reg, 'sui-bar-chart')
    await flush()
    expect(root(el).querySelectorAll('rect').length).toBe(0)
  })

  it('horizontal orientation places first bar at x=0', async () => {
    const { el } = mount(reg, 'sui-bar-chart', { data: '1,2', orientation: 'horizontal' })
    await flush()
    const rects = root(el).querySelectorAll('rect')
    expect(Number(rects[0]?.getAttribute('x'))).toBe(0)
  })

  it('rect elements have rx=3 for rounded corners', async () => {
    const { el } = mount(reg, 'sui-bar-chart', { data: '5' })
    await flush()
    const rect = root(el).querySelector('rect')
    expect(rect?.getAttribute('rx')).toBe('3')
  })

  it('tone reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-bar-chart', { tone: 'success' })
    expect(el.getAttribute('tone')).toBe('success')
  })

  it('orientation reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-bar-chart', { orientation: 'horizontal' })
    expect(el.getAttribute('orientation')).toBe('horizontal')
  })

  it('data reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-bar-chart', { data: '1,2,3' })
    expect(el.getAttribute('data')).toBe('1,2,3')
  })

  it('paints every bar with a design token (no color literal)', async () => {
    const { el } = mount(reg, 'sui-bar-chart', { data: '1,2,3', tone: 'success' })
    await flush()
    expect(nonTokenPaints(root(el))).toEqual([])
    expect(root(el).querySelector('rect')?.getAttribute('fill')).toBe('var(--success)')
  })

  it('bars rest at 85% fill (the dimmed-mass treatment)', async () => {
    const { el } = mount(reg, 'sui-bar-chart', { data: '1,2,3' })
    await flush()
    const opacities = [...root(el).querySelectorAll('rect')].map((r) =>
      r.getAttribute('fill-opacity'),
    )
    expect(opacities).toEqual(['0.85', '0.85', '0.85'])
  })

  it('carries no inline style attribute (styling is the language, not style=)', async () => {
    const { el } = mount(reg, 'sui-bar-chart', { data: '1,2,3' })
    await flush()
    expect(root(el).querySelector('[style]')).toBeNull()
  })

  it('reacts to data attribute change', async () => {
    const { el } = mount(reg, 'sui-bar-chart', { data: '1,2,3' })
    await flush()
    expect(root(el).querySelectorAll('rect').length).toBe(3)
    el.setAttribute('data', '1,2,3,4,5')
    await flush()
    expect(root(el).querySelectorAll('rect').length).toBe(5)
  })

  it('each rect has a title child element', async () => {
    const { el } = mount(reg, 'sui-bar-chart', { data: '10,20' })
    await flush()
    const rects = root(el).querySelectorAll('rect')
    expect(rects[0]?.querySelector('title')).not.toBeNull()
    expect(rects[1]?.querySelector('title')).not.toBeNull()
  })
})
