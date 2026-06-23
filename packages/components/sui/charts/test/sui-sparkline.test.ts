import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-sparkline/sui-sparkline.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-sparkline>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-sparkline')).toBeDefined()
  })

  it('attaches a shadow root', () => {
    const { el } = mount(reg, 'sui-sparkline')
    expect(shadow(el)).not.toBeNull()
  })

  it('renders a stroke path for valid data', async () => {
    const { el } = mount(reg, 'sui-sparkline', { data: '1,2,3,4,5' })
    await flush()
    const paths = shadow(el).querySelectorAll('path')
    expect(paths.length).toBeGreaterThanOrEqual(1)
    expect(paths[0]?.getAttribute('d')).toMatch(/^M/)
  })

  it('SVG has role=img when data is valid', async () => {
    const { el } = mount(reg, 'sui-sparkline', { data: '1,2,3' })
    await flush()
    expect(shadow(el).querySelector('svg')?.getAttribute('role')).toBe('img')
  })

  it('aria-label leads with the label prop (kind + data summary)', async () => {
    // MIGRATED: aria-label is now "kind: summary" (engine ariaSummary), not bare.
    const { el } = mount(reg, 'sui-sparkline', { data: '1,2,3', label: 'Trend' })
    await flush()
    const aria = shadow(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('Trend')).toBe(true)
  })

  it('aria-label data summary names the point count', async () => {
    // MIGRATED: ariaSummary derives the point count from the engine extent.
    const { el } = mount(reg, 'sui-sparkline', { data: '1,2,3' })
    await flush()
    const aria = shadow(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria).toContain('3 points')
  })

  it('default aria-label leads with "sparkline"', async () => {
    const { el } = mount(reg, 'sui-sparkline', { data: '1,2,3' })
    await flush()
    const aria = shadow(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('sparkline')).toBe(true)
  })

  it('renders 2 paths when fill attr is set', async () => {
    const { el } = mount(reg, 'sui-sparkline', { data: '1,2,3,4', fill: true })
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(2)
  })

  it('area fill path has fill-opacity 0.18', async () => {
    const { el } = mount(reg, 'sui-sparkline', { data: '1,2,3,4', fill: true })
    await flush()
    const first = shadow(el).querySelector('path')
    expect(first?.getAttribute('fill-opacity')).toBe('0.18')
  })

  it('renders a single-point series as one centered path', async () => {
    // MIGRATED: the unified engine renders a lone finite point (centered at w/2)
    // rather than dropping series with < 2 points — same null-gap contract as line.
    const { el } = mount(reg, 'sui-sparkline', { data: '5' })
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(1)
  })

  it('a null gap splits the stroke into sub-paths (break, not interpolate)', async () => {
    // MIGRATED: a null comma slot breaks the stroke into one M-sub-path per run.
    const { el } = mount(reg, 'sui-sparkline', { data: '1,2,,4,5' })
    await flush()
    const stroke = shadow(el).querySelectorAll('path')
    const d = stroke[stroke.length - 1]?.getAttribute('d') ?? ''
    const moves = d.match(/M/g)?.length ?? 0
    expect(moves).toBe(2)
  })

  it('renders no paths for empty data', async () => {
    const { el } = mount(reg, 'sui-sparkline')
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(0)
  })

  it('empty data keeps the lightweight aria-hidden empty SVG (no empty-state)', async () => {
    // MIGRATED: sparkline is the ONLY lightweight exception — empty stays the
    // aria-hidden empty SVG, NOT a full-size sui-empty-state.
    const { el } = mount(reg, 'sui-sparkline')
    await flush()
    expect(shadow(el).querySelector('svg')?.getAttribute('aria-hidden')).toBe('true')
    expect(shadow(el).querySelector('sui-empty-state')).toBeNull()
  })

  it('loading attr composes a sui-spinner (inline loading profile, not skeleton)', async () => {
    // MIGRATED: the inline presentation uses sui-spinner for loading; the
    // full-size sui-skeleton would be too heavy inline.
    const { el } = mount(reg, 'sui-sparkline', { data: '1,2,3', loading: true })
    await flush()
    expect(shadow(el).querySelector('sui-spinner')).not.toBeNull()
    expect(shadow(el).querySelector('sui-skeleton')).toBeNull()
  })

  it('loading reflects to host attribute', () => {
    // MIGRATED: loading is a reflected bool prop so :host([loading]) CSS reacts.
    const { el } = mount(reg, 'sui-sparkline', { loading: true })
    expect(el.hasAttribute('loading')).toBe(true)
  })

  it('tone reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-sparkline', { tone: 'success' })
    expect(el.getAttribute('tone')).toBe('success')
  })

  it('fill reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-sparkline', { fill: true })
    expect(el.hasAttribute('fill')).toBe(true)
  })

  it('data reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-sparkline', { data: '1,2,3' })
    expect(el.getAttribute('data')).toBe('1,2,3')
  })

  it('shadow styles include prefers-reduced-motion rule', () => {
    const { el } = mount(reg, 'sui-sparkline')
    const styleEl = shadow(el).querySelector('style')
    expect(styleEl?.textContent).toContain('prefers-reduced-motion')
  })

  it('shadow <style> carries no color literal (token-clean)', () => {
    const { el } = mount(reg, 'sui-sparkline')
    const css = shadow(el).querySelector('style')?.textContent ?? ''
    expect(css).not.toMatch(/#[0-9a-fA-F]{3,8}\b/)
    expect(css).not.toMatch(/\brgba?\(/)
    expect(css).not.toMatch(/\boklch\(/)
    expect(css).not.toContain('color-mix')
  })

  it('reacts to data attribute change', async () => {
    const { el } = mount(reg, 'sui-sparkline', { data: '1,2,3' })
    await flush()
    const before = shadow(el).querySelector('path')?.getAttribute('d')
    el.setAttribute('data', '5,4,3,2,1')
    await flush()
    const after = shadow(el).querySelector('path')?.getAttribute('d')
    expect(after).not.toEqual(before)
  })
})
