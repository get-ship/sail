// Characterization spec for <sui-badge> — a static, non-interactive status
// label. Drives the production WASM-compiled emit through the `.compiled.js`
// shim (resolved by the vitest plugin in @ship/test-setup), so a passing test
// proves the shipped component registers, reflects its tone/variant/size/pill
// channels, projects its slots, and exposes the right shadow parts.

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-badge/sui-badge.compiled.js'
import { createContainerRegistry, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-badge>', () => {

  it('attaches a shadow root with the .badge part', () => {
    const { el } = mount(reg, 'sui-badge')
    expect(el.shadowRoot).not.toBeNull()
    const badge = shadow(el).querySelector('.badge')
    expect(badge).not.toBeNull()
    expect(badge!.getAttribute('part')).toBe('badge')
  })

  it('renders without reflecting default channels as host attributes', () => {
    // `reflect` mirrors a prop to the host attribute when the prop is SET,
    // not for its default — an unconfigured badge carries no tone/variant/
    // size/pill attributes, leaning on the base `.badge` (soft) styling.
    const { el } = mount(reg, 'sui-badge')
    expect(el.hasAttribute('tone')).toBe(false)
    expect(el.hasAttribute('variant')).toBe(false)
    expect(el.hasAttribute('size')).toBe(false)
    expect(el.hasAttribute('pill')).toBe(false)
    expect(shadow(el).querySelector('.badge')).not.toBeNull()
  })

  describe('tone matrix', () => {
    for (const tone of [
      'default',
      'primary',
      'info',
      'success',
      'warning',
      'destructive',
      'accent',
      'muted',
    ]) {
      it(`reflects tone="${tone}"`, () => {
        const { el } = mount(reg, 'sui-badge', { tone })
        expect(el.getAttribute('tone')).toBe(tone)
        expect(shadow(el).querySelector('.badge')).not.toBeNull()
      })
    }
  })

  describe('variant matrix', () => {
    for (const variant of ['solid', 'soft', 'outline']) {
      it(`reflects variant="${variant}"`, () => {
        const { el } = mount(reg, 'sui-badge', { variant })
        expect(el.getAttribute('variant')).toBe(variant)
      })
    }
  })

  describe('size matrix', () => {
    for (const size of ['sm', 'md', 'lg']) {
      it(`reflects size="${size}"`, () => {
        const { el } = mount(reg, 'sui-badge', { size })
        expect(el.getAttribute('size')).toBe(size)
      })
    }
  })

  it('reflects the pill boolean attribute', () => {
    const { el } = mount(reg, 'sui-badge', { pill: true })
    expect(el.hasAttribute('pill')).toBe(true)
  })

  it('projects default-slot label content', () => {
    const { el } = mount(reg, 'sui-badge', {}, 'New')
    // The light-DOM child is projected through the default slot.
    expect(el.textContent).toContain('New')
    expect(shadow(el).querySelector('slot:not([name])')).not.toBeNull()
  })

  it('exposes a leading slot + .leading part wrapper', () => {
    const { el } = mount(reg, 'sui-badge')
    const leading = shadow(el).querySelector('.leading')
    expect(leading).not.toBeNull()
    expect(leading!.getAttribute('part')).toBe('leading')
    expect(leading!.querySelector('slot[name="leading"]')).not.toBeNull()
  })

  it('has no interactive role (purely presentational)', () => {
    const { el } = mount(reg, 'sui-badge')
    expect(el.getAttribute('role')).toBeNull()
    expect(el.getAttribute('tabindex')).toBeNull()
  })
})
