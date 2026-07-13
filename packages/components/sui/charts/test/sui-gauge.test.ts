import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-gauge/sui-gauge.compiled.js'
import { createContainerRegistry, flush, mount, nonTokenPaints } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

// Light DOM: the component renders into the host element, so the host IS the
// query root — no shadow boundary, and no hand-authored classes to key on. The
// queries below are the component's real semantics: tags, ARIA, SVG paint.
const root = (el: HTMLElement): HTMLElement => el

// The gauge draws at most two arcs, in this order: the 270° track (always), then
// the value arc (only when the clamped ratio is > 0). Position IS the semantic.
const arcs = (el: HTMLElement) => [...root(el).querySelectorAll('svg path')]
const trackArc = (el: HTMLElement) => arcs(el)[0] ?? null
const valueArc = (el: HTMLElement) => arcs(el)[1] ?? null

describe('<sui-gauge>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-gauge')).toBeDefined()
  })

  it('renders into the light DOM (no shadow boundary)', async () => {
    const { el } = mount(reg, 'sui-gauge', { value: '50' })
    await flush()
    expect(el.shadowRoot).toBeNull()
    expect(el.querySelector('svg[role="img"]')).not.toBeNull()
  })

  it('SVG has role=img', async () => {
    const { el } = mount(reg, 'sui-gauge', { value: '50' })
    await flush()
    expect(root(el).querySelector('svg')?.getAttribute('role')).toBe('img')
  })

  it('aria-label leads with the label prop (kind + data summary)', async () => {
    // MIGRATED: aria-label is now "kind: summary" (engine ariaSummary), not bare.
    const { el } = mount(reg, 'sui-gauge', { value: '50', label: 'CPU load' })
    await flush()
    const aria = root(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('CPU load')).toBe(true)
  })

  it('default aria-label leads with "gauge"', async () => {
    const { el } = mount(reg, 'sui-gauge', { value: '50' })
    await flush()
    const aria = root(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('gauge')).toBe(true)
  })

  it('track path always rendered, stroked in the muted token', async () => {
    const { el } = mount(reg, 'sui-gauge', { value: '50' })
    await flush()
    expect(trackArc(el)).not.toBeNull()
    expect(trackArc(el)?.getAttribute('stroke')).toBe('var(--muted)')
  })

  it('value path rendered when value > 0', async () => {
    const { el } = mount(reg, 'sui-gauge', { value: '40', max: '100' })
    await flush()
    expect(valueArc(el)).not.toBeNull()
  })

  it('no value path when value is 0', async () => {
    const { el } = mount(reg, 'sui-gauge', { value: '0' })
    await flush()
    expect(valueArc(el)).toBeNull()
  })

  it('value reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-gauge', { value: '75' })
    expect(el.getAttribute('value')).toBe('75')
  })

  it('tone reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-gauge', { tone: 'success' })
    expect(el.getAttribute('tone')).toBe('success')
  })

  it('reacts to value attribute change', async () => {
    const { el } = mount(reg, 'sui-gauge', { value: '10', max: '100' })
    await flush()
    const before = valueArc(el)?.getAttribute('d')
    el.setAttribute('value', '80')
    await flush()
    const after = valueArc(el)?.getAttribute('d')
    expect(after).not.toEqual(before)
  })

  it('value above max renders a fill path (clamped to 100%)', async () => {
    const { el } = mount(reg, 'sui-gauge', { value: '200', max: '100' })
    await flush()
    expect(valueArc(el)).not.toBeNull()
  })

  it('value below min renders no fill path (clamped to 0)', async () => {
    const { el } = mount(reg, 'sui-gauge', { value: '-10', min: '0' })
    await flush()
    expect(valueArc(el)).toBeNull()
  })

  it('low-threshold triggers destructive arc color', async () => {
    const { el } = mount(reg, 'sui-gauge', {
      value: '10',
      'low-threshold': '20',
    })
    await flush()
    const fill = valueArc(el)
    expect(fill?.getAttribute('stroke')).toBe('var(--destructive)')
  })

  it('high-threshold triggers warning arc color', async () => {
    const { el } = mount(reg, 'sui-gauge', {
      value: '50',
      'low-threshold': '20',
      'high-threshold': '80',
    })
    await flush()
    const fill = valueArc(el)
    expect(fill?.getAttribute('stroke')).toBe('var(--warning)')
  })

  it('above high-threshold uses success arc color', async () => {
    const { el } = mount(reg, 'sui-gauge', {
      value: '90',
      'low-threshold': '20',
      'high-threshold': '80',
    })
    await flush()
    const fill = valueArc(el)
    expect(fill?.getAttribute('stroke')).toBe('var(--success)')
  })

  it('loading attr composes a sui-skeleton (full-size loading profile)', async () => {
    // MIGRATED: gauge is full-size despite being compact — loading composes
    // sui-skeleton via the engine isLoading discriminator.
    const { el } = mount(reg, 'sui-gauge', { value: '50', loading: true })
    await flush()
    expect(root(el).querySelector('[skeleton]')).not.toBeNull()
    expect(root(el).querySelector('svg[role="img"]')).toBeNull()
  })

  it('loading reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-gauge', { loading: true })
    expect(el.hasAttribute('loading')).toBe(true)
  })

  it('paints the arc with design tokens (no color literal)', async () => {
    const { el } = mount(reg, 'sui-gauge', { value: '50', tone: 'success' })
    await flush()
    expect(nonTokenPaints(root(el))).toEqual([])
  })

  it('carries no inline style attribute — diameter drives the SVG box', async () => {
    const { el } = mount(reg, 'sui-gauge', { value: '50', diameter: '160' })
    await flush()
    expect(root(el).querySelector('[style]')).toBeNull()
    const svg = root(el).querySelector('svg')
    expect(svg?.getAttribute('width')).toBe('160')
    expect(svg?.getAttribute('height')).toBe('160')
    expect(svg?.getAttribute('viewBox')).toBe('0 0 160 160')
  })

  it('diameter drives the arc geometry (a real number, not the size channel)', async () => {
    const { el } = mount(reg, 'sui-gauge', { value: '50', diameter: '200' })
    await flush()
    const d = trackArc(el)?.getAttribute('d') ?? ''
    expect(d).not.toContain('NaN')
    expect(d.length).toBeGreaterThan(0)
  })
})
