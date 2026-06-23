// Characterization spec for <sui-spinner> — a rotating ring loader. No
// controller: the size/tone channels reflect onto the host; the inner .ring
// flips between role=status (labelled) and aria-hidden (unlabelled) driven by
// the `label` template expression.

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-spinner/sui-spinner.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const ring = (el: HTMLElement) => shadow(el).querySelector('.ring')!

describe('<sui-spinner>', () => {

  it('attaches a shadow root with the .ring (part="ring")', () => {
    const { el } = mount(reg, 'sui-spinner')
    expect(el.shadowRoot).not.toBeNull()
    expect(ring(el)).not.toBeNull()
    expect(ring(el).getAttribute('part')).toBe('ring')
  })

  it('defaults to the "Loading" label → role=status + aria-label', () => {
    const { el } = mount(reg, 'sui-spinner')
    const r = ring(el)
    expect(r.getAttribute('role')).toBe('status')
    expect(r.getAttribute('aria-label')).toBe('Loading')
    // With a label present, the ring is not hidden from a11y.
    expect(r.getAttribute('aria-hidden')).toBeNull()
  })

  it('an explicit label drives role=status + that aria-label', () => {
    const { el } = mount(reg, 'sui-spinner', { label: 'Saving changes' })
    const r = ring(el)
    expect(r.getAttribute('role')).toBe('status')
    expect(r.getAttribute('aria-label')).toBe('Saving changes')
    expect(r.getAttribute('aria-hidden')).toBeNull()
  })

  it('an empty label hides the ring (aria-hidden, no role)', () => {
    const { el } = mount(reg, 'sui-spinner', { label: '' })
    const r = ring(el)
    expect(r.getAttribute('aria-hidden')).toBe('true')
    expect(r.getAttribute('role')).toBeNull()
    expect(r.getAttribute('aria-label')).toBeNull()
  })

  it('updates a11y reactively when label is cleared then restored', async () => {
    const { el } = mount(reg, 'sui-spinner', { label: 'Working' })
    expect(ring(el).getAttribute('role')).toBe('status')
    el.setAttribute('label', '')
    await flush()
    expect(ring(el).getAttribute('aria-hidden')).toBe('true')
    expect(ring(el).getAttribute('role')).toBeNull()
    el.setAttribute('label', 'Back')
    await flush()
    expect(ring(el).getAttribute('role')).toBe('status')
    expect(ring(el).getAttribute('aria-label')).toBe('Back')
  })

  describe('size matrix', () => {
    for (const size of ['xs', 'sm', 'md', 'lg']) {
      it(`reflects size="${size}"`, () => {
        const { el } = mount(reg, 'sui-spinner', { size })
        expect(el.getAttribute('size')).toBe(size)
        expect(ring(el)).not.toBeNull()
      })
    }
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
        const { el } = mount(reg, 'sui-spinner', { tone })
        expect(el.getAttribute('tone')).toBe(tone)
      })
    }
  })

  it('reflects the centered boolean attribute', () => {
    const { el } = mount(reg, 'sui-spinner', { centered: true })
    expect(el.hasAttribute('centered')).toBe(true)
  })
})
