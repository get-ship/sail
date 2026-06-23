// Characterization spec for <sui-link> — a styled anchor wrapper. No
// controller: an inner <a part="link"> binds href/target/rel; the `external`
// flag forces target=_blank + rel=noopener noreferrer and renders an
// sr-only "(opens in new tab)" hint plus the external-link glyph via {#if}.

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-link/sui-link.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const anchor = (el: HTMLElement) => shadow(el).querySelector('a')!

describe('<sui-link>', () => {

  it('renders an anchor with part="link"', () => {
    const { el } = mount(reg, 'sui-link')
    expect(el.shadowRoot).not.toBeNull()
    const a = anchor(el)
    expect(a).not.toBeNull()
    expect(a.getAttribute('part')).toBe('link')
  })

  it('binds the href onto the inner anchor', () => {
    const { el } = mount(reg, 'sui-link', { href: '/docs/intro' })
    expect(anchor(el).getAttribute('href')).toBe('/docs/intro')
  })

  it('projects default-slot label content', () => {
    const { el } = mount(reg, 'sui-link', { href: '/x' }, 'Read more')
    expect(el.textContent).toContain('Read more')
    expect(shadow(el).querySelector('a slot:not([name])')).not.toBeNull()
  })

  describe('internal (default) link', () => {
    it('does not force target or rel, and renders no external affordance', () => {
      const { el } = mount(reg, 'sui-link', { href: '/x' })
      const a = anchor(el)
      // target/rel default to the (empty) explicit props.
      expect(a.getAttribute('target')).not.toBe('_blank')
      expect(a.getAttribute('rel')).not.toBe('noopener noreferrer')
      // No external glyph / sr-only hint when not external.
      expect(shadow(el).querySelector('.ext')).toBeNull()
      expect(shadow(el).querySelector('.sr-only')).toBeNull()
    })

    it('honours an explicit target / rel override', () => {
      const { el } = mount(reg, 'sui-link', { href: '/x', target: '_self', rel: 'nofollow' })
      const a = anchor(el)
      expect(a.getAttribute('target')).toBe('_self')
      expect(a.getAttribute('rel')).toBe('nofollow')
    })
  })

  describe('external link', () => {
    it('reflects the external boolean attribute', () => {
      const { el } = mount(reg, 'sui-link', { href: 'https://x.dev', external: true })
      expect(el.hasAttribute('external')).toBe(true)
    })

    it('forces target=_blank and rel=noopener noreferrer', () => {
      const { el } = mount(reg, 'sui-link', { href: 'https://x.dev', external: true })
      const a = anchor(el)
      expect(a.getAttribute('target')).toBe('_blank')
      expect(a.getAttribute('rel')).toBe('noopener noreferrer')
    })

    it('renders the sr-only new-tab hint and the external glyph', () => {
      const { el } = mount(reg, 'sui-link', { href: 'https://x.dev', external: true })
      const sr = shadow(el).querySelector('.sr-only')
      expect(sr).not.toBeNull()
      expect(sr!.textContent).toContain('opens in new tab')
      const glyph = shadow(el).querySelector('.ext')
      expect(glyph).not.toBeNull()
      expect(glyph!.getAttribute('aria-hidden')).toBe('true')
      expect(glyph!.querySelector('svg')).not.toBeNull()
    })

    it('external precedence wins over an explicit target', () => {
      const { el } = mount(reg, 'sui-link', { href: 'https://x.dev', external: true, target: '_self' })
      expect(anchor(el).getAttribute('target')).toBe('_blank')
    })
  })

  it('toggling external reactively adds then removes the glyph', async () => {
    const { el } = mount(reg, 'sui-link', { href: 'https://x.dev' })
    expect(shadow(el).querySelector('.ext')).toBeNull()
    el.setAttribute('external', '')
    await flush()
    expect(shadow(el).querySelector('.ext')).not.toBeNull()
    expect(anchor(el).getAttribute('target')).toBe('_blank')
    el.removeAttribute('external')
    await flush()
    expect(shadow(el).querySelector('.ext')).toBeNull()
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
        const { el } = mount(reg, 'sui-link', { href: '/x', tone })
        expect(el.getAttribute('tone')).toBe(tone)
      })
    }
  })

  describe('variant matrix', () => {
    for (const variant of ['default', 'underline', 'subtle']) {
      it(`reflects variant="${variant}"`, () => {
        const { el } = mount(reg, 'sui-link', { href: '/x', variant })
        expect(el.getAttribute('variant')).toBe(variant)
      })
    }
  })
})
