// Characterization spec for <sui-skeleton> — a shimmer placeholder for loading
// states. No controller: the shape/size channels reflect onto the host and the
// inner .sk element carries the loading a11y semantics (role/aria-busy) the Lit
// original set on the host.

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-skeleton/sui-skeleton.compiled.js'
import { createContainerRegistry, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const bar = (el: HTMLElement) => shadow(el).querySelector('.sk')!

describe('<sui-skeleton>', () => {

  it('attaches a shadow root with the .sk bar (part="sk")', () => {
    const { el } = mount(reg, 'sui-skeleton')
    expect(el.shadowRoot).not.toBeNull()
    expect(bar(el)).not.toBeNull()
    expect(bar(el).getAttribute('part')).toBe('sk')
  })

  it('the bar carries loading a11y semantics', () => {
    const { el } = mount(reg, 'sui-skeleton')
    const sk = bar(el)
    expect(sk.getAttribute('role')).toBe('status')
    expect(sk.getAttribute('aria-label')).toBe('Loading')
    expect(sk.getAttribute('aria-live')).toBe('polite')
    expect(sk.getAttribute('aria-busy')).toBe('true')
  })

  describe('shape matrix', () => {
    for (const shape of ['text', 'line', 'block', 'circle']) {
      it(`reflects shape="${shape}"`, () => {
        const { el } = mount(reg, 'sui-skeleton', { shape })
        expect(el.getAttribute('shape')).toBe(shape)
        expect(bar(el)).not.toBeNull()
      })
    }
  })

  describe('size matrix', () => {
    for (const size of ['sm', 'md', 'lg']) {
      it(`reflects size="${size}"`, () => {
        const { el } = mount(reg, 'sui-skeleton', { size })
        expect(el.getAttribute('size')).toBe(size)
      })
    }
  })

  it('does not reflect default shape/size as host attributes', () => {
    // Defaults (shape=text, size=md) are not mirrored to the host until set.
    const { el } = mount(reg, 'sui-skeleton')
    expect(el.hasAttribute('shape')).toBe(false)
    expect(el.hasAttribute('size')).toBe(false)
  })

  it('keeps a11y semantics under a circle+lg combination', () => {
    const { el } = mount(reg, 'sui-skeleton', { shape: 'circle', size: 'lg' })
    expect(el.getAttribute('shape')).toBe('circle')
    expect(el.getAttribute('size')).toBe('lg')
    expect(bar(el).getAttribute('aria-busy')).toBe('true')
  })
})
