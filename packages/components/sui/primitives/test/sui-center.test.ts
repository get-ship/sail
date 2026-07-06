import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-center/sui-center.compiled.js'
import { createContainerRegistry, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

describe('<sui-center>', () => {

  it('registers as a custom element', () => {
    const { el } = mount(reg, 'sui-center', {}, '<p>centered</p>')
    expect(el.tagName.toLowerCase()).toBe('sui-center')
    expect(el.shadowRoot).not.toBeNull()
  })

  it('default gap attribute absent (SUI defers attribute reflection until prop changes)', () => {
    const { el } = mount(reg, 'sui-center', {})
    expect(el.getAttribute('gap')).toBeNull()
  })

  it('reflects gap attribute when set', () => {
    const { el } = mount(reg, 'sui-center', { gap: '3' })
    expect(el.getAttribute('gap')).toBe('3')
  })

  it('reflects every gap value (0..8)', () => {
    for (const g of ['0', '1', '2', '3', '4', '5', '6', '7', '8']) {
      const { el } = mount(reg, 'sui-center', { gap: g })
      expect(el.getAttribute('gap')).toBe(g)
    }
  })

  it('projects default slot content into shadow', () => {
    const { el, shadow } = mount(reg, 'sui-center', {}, '<p>centered</p>')
    expect(shadow!.querySelector('slot:not([name])')).not.toBeNull()
  })

  it('host is a grid (the layout box) — :host has display:grid', () => {
    const { el } = mount(reg, 'sui-center', {})
    const styleEl = el.shadowRoot!.querySelector('style')!
    expect(styleEl.textContent).toContain(':host')
    expect(styleEl.textContent).toContain('display: grid')
    expect(styleEl.textContent).toContain('place-items: center')
  })

  it('every gap value has a matching :host rule that sets --center-gap', () => {
    const { el } = mount(reg, 'sui-center', {})
    const styleEl = el.shadowRoot!.querySelector('style')!
    const css = styleEl.textContent!
    for (const g of ['0', '1', '2', '3', '4', '5', '6', '7', '8']) {
      // The rule may use either `0` (literal zero) or `var(--space-N)` for the
      // value; both are valid -- the parity claim is that a --center-gap fill
      // exists for every value the prop accepts.
      expect(css).toMatch(new RegExp(`\\[gap='${g}'\\][^{]*\\{[^}]*--center-gap`))
    }
  })
})
