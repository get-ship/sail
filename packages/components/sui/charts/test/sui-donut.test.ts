import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-donut/sui-donut.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

const seg = (label: string, value: number, tone?: string) =>
  JSON.stringify([{ label, value, tone }])

describe('<sui-donut>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-donut')).toBeDefined()
  })

  it('attaches a shadow root', () => {
    const { el } = mount(reg, 'sui-donut')
    expect(shadow(el)).not.toBeNull()
  })

  it('SVG has role=img', async () => {
    const { el } = mount(reg, 'sui-donut', { data: seg('a', 1) })
    await flush()
    expect(shadow(el).querySelector('svg')?.getAttribute('role')).toBe('img')
  })

  it('aria-label leads with the label prop (kind + data summary)', async () => {
    // MIGRATED: aria-label is now "kind: summary" (engine ariaSummary), not bare.
    const { el } = mount(reg, 'sui-donut', { data: seg('a', 1), label: 'Sales mix' })
    await flush()
    const aria = shadow(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
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
    const aria = shadow(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria).toContain('2 points')
  })

  it('default aria-label leads with "donut chart"', async () => {
    const { el } = mount(reg, 'sui-donut', { data: seg('a', 1) })
    await flush()
    const aria = shadow(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('donut chart')).toBe(true)
  })

  it('track circle always rendered for non-empty data', async () => {
    const { el } = mount(reg, 'sui-donut', { data: seg('a', 1) })
    await flush()
    expect(shadow(el).querySelector('circle[part="track"]')).not.toBeNull()
  })

  it('empty data composes a sui-empty-state (full-size presentation profile)', async () => {
    // MIGRATED: empty data routes through the engine isEmpty discriminator to
    // sui-empty-state, replacing the silent no-segment render.
    const { el } = mount(reg, 'sui-donut')
    await flush()
    expect(shadow(el).querySelector('sui-empty-state')).not.toBeNull()
    expect(shadow(el).querySelector('svg[role="img"]')).toBeNull()
  })

  it('empty data renders no segment paths', async () => {
    const { el } = mount(reg, 'sui-donut')
    await flush()
    expect(shadow(el).querySelectorAll('path').length).toBe(0)
  })

  it('renders one path per positive-value segment', async () => {
    const data = JSON.stringify([
      { label: 'a', value: 3, tone: 'info' },
      { label: 'b', value: 1, tone: 'success' },
      { label: 'c', value: 2, tone: 'warning' },
    ])
    const { el } = mount(reg, 'sui-donut', { data })
    await flush()
    expect(shadow(el).querySelectorAll('path[part="segment"]').length).toBe(3)
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
    const segs = shadow(el).querySelectorAll('path[part="segment"]')
    expect(segs.length).toBe(2)
    const titles = Array.from(segs).map((p) => p.querySelector('title')?.textContent ?? '')
    expect(titles.some((t) => t.includes('a'))).toBe(true)
    expect(titles.some((t) => t.includes('c'))).toBe(true)
  })

  it('single 100%-value segment renders one path', async () => {
    const { el } = mount(reg, 'sui-donut', { data: seg('only', 10) })
    await flush()
    expect(shadow(el).querySelectorAll('path[part="segment"]').length).toBe(1)
  })

  it('each segment path has a title child', async () => {
    const data = JSON.stringify([
      { label: 'x', value: 2 },
      { label: 'y', value: 3 },
    ])
    const { el } = mount(reg, 'sui-donut', { data })
    await flush()
    const paths = shadow(el).querySelectorAll('path[part="segment"]')
    for (const p of paths) {
      expect(p.querySelector('title')).not.toBeNull()
    }
  })

  it('loading attr composes a sui-skeleton (full-size loading profile)', async () => {
    // MIGRATED: the engine isLoading discriminator drives the loading branch.
    const { el } = mount(reg, 'sui-donut', { data: seg('a', 1), loading: true })
    await flush()
    expect(shadow(el).querySelector('sui-skeleton')).not.toBeNull()
    expect(shadow(el).querySelector('svg[role="img"]')).toBeNull()
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
    expect(shadow(el).querySelectorAll('path[part="segment"]').length).toBe(1)
    el.setAttribute(
      'data',
      JSON.stringify([
        { label: 'a', value: 1 },
        { label: 'b', value: 2 },
        { label: 'c', value: 3 },
      ]),
    )
    await flush()
    expect(shadow(el).querySelectorAll('path[part="segment"]').length).toBe(3)
  })

  it('slot renders center content', async () => {
    const { el } = mount(reg, 'sui-donut', { data: seg('a', 5) }, '42')
    await flush()
    expect(shadow(el).querySelector('.center')).not.toBeNull()
    expect(el.textContent?.trim()).toBe('42')
  })

  it('shadow styles include prefers-reduced-motion rule', () => {
    const { el } = mount(reg, 'sui-donut')
    const styleEl = shadow(el).querySelector('style')
    expect(styleEl?.textContent).toContain('prefers-reduced-motion')
  })

  it('shadow <style> carries no color literal (token-clean)', () => {
    const { el } = mount(reg, 'sui-donut')
    const css = shadow(el).querySelector('style')?.textContent ?? ''
    expect(css).not.toMatch(/#[0-9a-fA-F]{3,8}\b/)
    expect(css).not.toMatch(/\brgba?\(/)
    expect(css).not.toMatch(/\boklch\(/)
    expect(css).not.toContain('color-mix')
  })

  it('segment paths have part=segment attribute', async () => {
    const { el } = mount(reg, 'sui-donut', { data: seg('a', 5) })
    await flush()
    const path = shadow(el).querySelector('path')
    expect(path?.getAttribute('part')).toBe('segment')
  })
})
