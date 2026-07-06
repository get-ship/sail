import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-grid/sui-grid.compiled.js'
import { createContainerRegistry, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

describe('<sui-grid>', () => {

  it('registers as a custom element', () => {
    const { el } = mount(reg, 'sui-grid', {}, '<div>item</div>')
    expect(el.tagName.toLowerCase()).toBe('sui-grid')
    expect(el.shadowRoot).not.toBeNull()
  })

  it('default attribute absence — SUI defers reflection until prop changes (min/md, gap/3, flow/dense)', () => {
    const { el } = mount(reg, 'sui-grid', {})
    expect(el.getAttribute('min')).toBeNull()
    expect(el.getAttribute('gap')).toBeNull()
    expect(el.getAttribute('flow')).toBeNull()
  })

  it('reflects min when set', () => {
    const { el } = mount(reg, 'sui-grid', { min: 'lg' })
    expect(el.getAttribute('min')).toBe('lg')
  })

  it('reflects every min value (xs|sm|md|lg)', () => {
    for (const m of ['xs', 'sm', 'md', 'lg']) {
      const { el } = mount(reg, 'sui-grid', { min: m })
      expect(el.getAttribute('min')).toBe(m)
    }
  })

  it('reflects gap when set', () => {
    const { el } = mount(reg, 'sui-grid', { gap: '5' })
    expect(el.getAttribute('gap')).toBe('5')
  })

  it('reflects every gap value (0..8)', () => {
    for (const g of ['0', '1', '2', '3', '4', '5', '6', '7', '8']) {
      const { el } = mount(reg, 'sui-grid', { gap: g })
      expect(el.getAttribute('gap')).toBe(g)
    }
  })

  it('reflects flow when set', () => {
    const { el } = mount(reg, 'sui-grid', { flow: 'auto' })
    expect(el.getAttribute('flow')).toBe('auto')
  })

  it('reflects both flow values (auto|dense)', () => {
    for (const f of ['auto', 'dense']) {
      const { el } = mount(reg, 'sui-grid', { flow: f })
      expect(el.getAttribute('flow')).toBe(f)
    }
  })

  it('projects default slot content into shadow', () => {
    const { shadow } = mount(reg, 'sui-grid', {}, '<div>item</div>')
    expect(shadow!.querySelector('slot:not([name])')).not.toBeNull()
  })

  it('the inner .grid wrapper is a CSS grid that reads --grid-min and --grid-gap', () => {
    const { el } = mount(reg, 'sui-grid', {})
    const styleEl = el.shadowRoot!.querySelector('style')!
    const css = styleEl.textContent!
    // .grid uses display:grid + the auto-fill minmax that defines the primitive
    expect(css).toContain('.grid')
    expect(css).toContain('display: grid')
    expect(css).toContain('grid-template-columns: repeat(auto-fill, minmax(var(--grid-min), 1fr))')
    expect(css).toContain('grid-auto-flow: dense')
    expect(css).toContain('gap: var(--grid-gap)')
  })

  it('every gap value has a matching :host rule that fills --grid-gap', () => {
    const { el } = mount(reg, 'sui-grid', {})
    const styleEl = el.shadowRoot!.querySelector('style')!
    const css = styleEl.textContent!
    for (const g of ['0', '1', '2', '3', '4', '5', '6', '7', '8']) {
      expect(css).toMatch(new RegExp(`\\[gap='${g}'\\][^{]*\\{[^}]*--grid-gap`))
    }
  })

  it('every min value has a matching :host rule that fills --grid-min', () => {
    const { el } = mount(reg, 'sui-grid', {})
    const styleEl = el.shadowRoot!.querySelector('style')!
    const css = styleEl.textContent!
    for (const m of ['xs', 'sm', 'md', 'lg']) {
      expect(css).toMatch(new RegExp(`\\[min='${m}'\\][^{]*\\{[^}]*--grid-min`))
    }
  })

  it('flow=auto overrides the default dense packing', () => {
    const { el } = mount(reg, 'sui-grid', { flow: 'auto' })
    const styleEl = el.shadowRoot!.querySelector('style')!
    const css = styleEl.textContent!
    expect(css).toMatch(/\[flow='auto'\][^{]*\.grid[^{]*\{[^}]*grid-auto-flow: auto/)
  })

  it('per-child data-col-span opt-in is honored by .grid ::slotted selectors', () => {
    const { el } = mount(reg, 'sui-grid', {}, '<div data-col-span="2">wide</div>')
    const styleEl = el.shadowRoot!.querySelector('style')!
    const css = styleEl.textContent!
    // Every span value 2..4 has a .grid ::slotted(*[data-col-span='N']) rule.
    for (const n of ['2', '3', '4']) {
      expect(css).toMatch(new RegExp(`\\[data-col-span='${n}'\\][^{]*\\{[^}]*grid-column: span ${n}`))
    }
  })

  it('per-child data-row-span opt-in is honored by .grid ::slotted selectors', () => {
    const { el } = mount(reg, 'sui-grid', {}, '<div data-row-span="3">tall</div>')
    const styleEl = el.shadowRoot!.querySelector('style')!
    const css = styleEl.textContent!
    for (const n of ['2', '3', '4']) {
      expect(css).toMatch(new RegExp(`\\[data-row-span='${n}'\\][^{]*\\{[^}]*grid-row: span ${n}`))
    }
  })

  it('max-width: 720px media block collapses the spans back to 1x1', () => {
    const { el } = mount(reg, 'sui-grid', {})
    const styleEl = el.shadowRoot!.querySelector('style')!
    const css = styleEl.textContent!
    expect(css).toContain('@media (max-width: 720px)')
    expect(css).toMatch(/@media \(max-width: 720px\)\s*\{[^}]*\[data-col-span\][^{]*\{[^}]*grid-column: span 1/)
  })
})
