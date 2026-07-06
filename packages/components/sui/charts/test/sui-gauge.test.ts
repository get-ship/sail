import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-gauge/sui-gauge.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-gauge>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-gauge')).toBeDefined()
  })

  it('attaches a shadow root', () => {
    const { el } = mount(reg, 'sui-gauge')
    expect(shadow(el)).not.toBeNull()
  })

  it('SVG has role=img', async () => {
    const { el } = mount(reg, 'sui-gauge', { value: '50' })
    await flush()
    expect(shadow(el).querySelector('svg')?.getAttribute('role')).toBe('img')
  })

  it('aria-label leads with the label prop (kind + data summary)', async () => {
    // MIGRATED: aria-label is now "kind: summary" (engine ariaSummary), not bare.
    const { el } = mount(reg, 'sui-gauge', { value: '50', label: 'CPU load' })
    await flush()
    const aria = shadow(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('CPU load')).toBe(true)
  })

  it('default aria-label leads with "gauge"', async () => {
    const { el } = mount(reg, 'sui-gauge', { value: '50' })
    await flush()
    const aria = shadow(el).querySelector('svg')?.getAttribute('aria-label') ?? ''
    expect(aria.startsWith('gauge')).toBe(true)
  })

  it('track path always rendered', async () => {
    const { el } = mount(reg, 'sui-gauge', { value: '50' })
    await flush()
    expect(shadow(el).querySelector('path[part="track"]')).not.toBeNull()
  })

  it('value path rendered when value > 0', async () => {
    const { el } = mount(reg, 'sui-gauge', { value: '40', max: '100' })
    await flush()
    expect(shadow(el).querySelector('path[part="fill"]')).not.toBeNull()
  })

  it('no value path when value is 0', async () => {
    const { el } = mount(reg, 'sui-gauge', { value: '0' })
    await flush()
    expect(shadow(el).querySelector('path[part="fill"]')).toBeNull()
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
    const before = shadow(el).querySelector('path[part="fill"]')?.getAttribute('d')
    el.setAttribute('value', '80')
    await flush()
    const after = shadow(el).querySelector('path[part="fill"]')?.getAttribute('d')
    expect(after).not.toEqual(before)
  })

  it('value above max renders a fill path (clamped to 100%)', async () => {
    const { el } = mount(reg, 'sui-gauge', { value: '200', max: '100' })
    await flush()
    expect(shadow(el).querySelector('path[part="fill"]')).not.toBeNull()
  })

  it('value below min renders no fill path (clamped to 0)', async () => {
    const { el } = mount(reg, 'sui-gauge', { value: '-10', min: '0' })
    await flush()
    expect(shadow(el).querySelector('path[part="fill"]')).toBeNull()
  })

  it('low-threshold triggers destructive arc color', async () => {
    const { el } = mount(reg, 'sui-gauge', {
      value: '10',
      'low-threshold': '20',
    })
    await flush()
    const fill = shadow(el).querySelector('path[part="fill"]')
    expect(fill?.getAttribute('stroke')).toBe('var(--destructive)')
  })

  it('high-threshold triggers warning arc color', async () => {
    const { el } = mount(reg, 'sui-gauge', {
      value: '50',
      'low-threshold': '20',
      'high-threshold': '80',
    })
    await flush()
    const fill = shadow(el).querySelector('path[part="fill"]')
    expect(fill?.getAttribute('stroke')).toBe('var(--warning)')
  })

  it('above high-threshold uses success arc color', async () => {
    const { el } = mount(reg, 'sui-gauge', {
      value: '90',
      'low-threshold': '20',
      'high-threshold': '80',
    })
    await flush()
    const fill = shadow(el).querySelector('path[part="fill"]')
    expect(fill?.getAttribute('stroke')).toBe('var(--success)')
  })

  it('loading attr composes a sui-skeleton (full-size loading profile)', async () => {
    // MIGRATED: gauge is full-size despite being compact — loading composes
    // sui-skeleton via the engine isLoading discriminator.
    const { el } = mount(reg, 'sui-gauge', { value: '50', loading: true })
    await flush()
    expect(shadow(el).querySelector('[skeleton]')).not.toBeNull()
    expect(shadow(el).querySelector('svg[role="img"]')).toBeNull()
  })

  it('loading reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-gauge', { loading: true })
    expect(el.hasAttribute('loading')).toBe(true)
  })

  it('shadow styles include prefers-reduced-motion rule', () => {
    const { el } = mount(reg, 'sui-gauge')
    const styleEl = shadow(el).querySelector('style')
    expect(styleEl?.textContent).toContain('prefers-reduced-motion')
  })

  it('shadow <style> carries no color literal (token-clean)', () => {
    const { el } = mount(reg, 'sui-gauge')
    const css = shadow(el).querySelector('style')?.textContent ?? ''
    expect(css).not.toMatch(/#[0-9a-fA-F]{3,8}\b/)
    expect(css).not.toMatch(/\brgba?\(/)
    expect(css).not.toMatch(/\boklch\(/)
    expect(css).not.toContain('color-mix')
  })
})
