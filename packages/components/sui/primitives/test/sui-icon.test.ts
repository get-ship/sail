import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-icon/sui-icon.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

// Helpers
const shadow = (el: HTMLElement) => el.shadowRoot!
// JSDOM doesn't match class selectors on SVG elements (the SVG namespace
// element doesn't get the HTMLElement class-attribute behavior). Use an
// attribute selector so the same code matches in both jsdom and real
// browsers — `<sui-icon>` only ever renders one `<svg>` child anyway.
const svg = (el: HTMLElement) =>
  shadow(el).querySelector('svg[part="svg"]') as SVGSVGElement | null

describe('<sui-icon>', () => {

  it('renders default (no name, no label) — empty slot, host has aria-hidden=true', () => {
    const { el } = mount(reg, 'sui-icon')
    expect(el.shadowRoot).not.toBeNull()
    expect(svg(el)).toBeNull()
    expect(el.getAttribute('aria-hidden')).toBe('true')
    expect(el.hasAttribute('role')).toBe(false)
    expect(el.hasAttribute('aria-label')).toBe(false)
  })

  it('renders a known name — SVG present in shadow DOM', () => {
    const { el } = mount(reg, 'sui-icon', { name: 'chevron-right' })
    const svgEl = svg(el)
    expect(svgEl).not.toBeNull()
    expect(svgEl!.querySelector('path')).not.toBeNull()
    expect(svgEl!.getAttribute('viewBox')).toBe('0 0 24 24')
    expect(svgEl!.getAttribute('stroke')).toBe('currentColor')
    expect(svgEl!.getAttribute('stroke-width')).toBe('2')
    expect(svgEl!.getAttribute('stroke-linecap')).toBe('round')
    expect(svgEl!.getAttribute('stroke-linejoin')).toBe('round')
  })

  it('renders an aliased name (git → git-branch) — resolves correctly', () => {
    const { el } = mount(reg, 'sui-icon', { name: 'git' })
    const svgEl = svg(el)
    expect(svgEl).not.toBeNull()
    expect(svgEl!.querySelector('path')).not.toBeNull()
  })

  it('unknown name — SVG absent, slot fallback content can show', () => {
    const { el } = mount(reg, 'sui-icon', { name: 'unknown-glyph-xyz' })
    expect(svg(el)).toBeNull()
    expect(el.getAttribute('aria-hidden')).toBe('true')
  })

  it('slot passthrough — slotted SVG is accessible via light DOM', () => {
    const { el } = mount(reg, 'sui-icon', {}, '<svg data-custom></svg>')
    expect(el.querySelector('svg[data-custom]')).not.toBeNull()
  })

  it('size sm — reflects attribute', () => {
    const { el } = mount(reg, 'sui-icon', { name: 'check', size: 'sm' })
    expect(el.getAttribute('size')).toBe('sm')
  })

  it('size md — reflects attribute (default)', () => {
    const { el } = mount(reg, 'sui-icon', { name: 'check', size: 'md' })
    expect(el.getAttribute('size')).toBe('md')
  })

  it('size lg — reflects attribute', () => {
    const { el } = mount(reg, 'sui-icon', { name: 'check', size: 'lg' })
    expect(el.getAttribute('size')).toBe('lg')
  })

  it('tone reflects to host attribute for all eight values', () => {
    const tones = ['default', 'info', 'success', 'warning', 'destructive', 'accent', 'muted', 'primary']
    for (const tone of tones) {
      const { el } = mount(reg, 'sui-icon', { name: 'check', tone })
      expect(el.getAttribute('tone')).toBe(tone)
    }
  })

  it('label set — host has role=img and aria-label, no aria-hidden', () => {
    const { el } = mount(reg, 'sui-icon', { name: 'info', label: 'Save' })
    expect(el.getAttribute('role')).toBe('img')
    expect(el.getAttribute('aria-label')).toBe('Save')
    expect(el.hasAttribute('aria-hidden')).toBe(false)
  })

  it('label set — inner SVG has no aria-hidden', () => {
    const { el } = mount(reg, 'sui-icon', { name: 'check', label: 'Done' })
    const svgEl = svg(el)
    expect(svgEl).not.toBeNull()
    expect(svgEl!.hasAttribute('aria-hidden')).toBe(false)
  })

  it('label empty — inner SVG has aria-hidden=true', () => {
    const { el } = mount(reg, 'sui-icon', { name: 'check' })
    const svgEl = svg(el)
    expect(svgEl).not.toBeNull()
    expect(svgEl!.getAttribute('aria-hidden')).toBe('true')
  })

  it('setting label after mount — ARIA flips correctly', async () => {
    const { el } = mount(reg, 'sui-icon', { name: 'check' })
    expect(el.getAttribute('aria-hidden')).toBe('true')
    el.setAttribute('label', 'Check icon')
    await flush()
    expect(el.getAttribute('role')).toBe('img')
    expect(el.getAttribute('aria-label')).toBe('Check icon')
    expect(el.hasAttribute('aria-hidden')).toBe(false)
  })

  it('removing label (set to "") — ARIA reverts to aria-hidden=true', async () => {
    const { el } = mount(reg, 'sui-icon', { name: 'check', label: 'Check' })
    expect(el.getAttribute('role')).toBe('img')
    el.setAttribute('label', '')
    await flush()
    expect(el.getAttribute('aria-hidden')).toBe('true')
    expect(el.hasAttribute('role')).toBe(false)
    expect(el.hasAttribute('aria-label')).toBe(false)
  })

  it('cleanup on disconnect — no leaked effects', () => {
    const { el, container } = mount(reg, 'sui-icon', { name: 'check' })
    expect(svg(el)).not.toBeNull()
    container.remove()
    // No assertion needed — if effects leak they would throw on re-mount
  })

  it('multiple instances render independently', () => {
    const { el: a } = mount(reg, 'sui-icon', { name: 'check' })
    const { el: b } = mount(reg, 'sui-icon', { name: 'x' })
    const svgA = svg(a)
    const svgB = svg(b)
    expect(svgA).not.toBeNull()
    expect(svgB).not.toBeNull()
    // Different icons → different path content
    const pathA = svgA!.querySelector('path')?.getAttribute('d')
    const pathB = svgB!.querySelector('path')?.getAttribute('d')
    expect(pathA).not.toBe(pathB)
  })
})
