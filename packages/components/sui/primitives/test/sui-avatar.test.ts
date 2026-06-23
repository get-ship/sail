// Characterization spec for <sui-avatar> — image / initials / icon avatar.
// The template picks img vs initials via `(iconSlug || src) && !errored`; the
// paired controller seeds an `errored` signal, flips it on img load error
// (Radix-parity fallback), and resets it when the source changes. role=img and
// aria-label (name → initials → 'avatar') live on the .root host span.

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-avatar/sui-avatar.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const root = (el: HTMLElement) => shadow(el).querySelector('.root')!
const img = (el: HTMLElement) => shadow(el).querySelector('img')
const initials = (el: HTMLElement) => shadow(el).querySelector('[part="initials"]')

describe('<sui-avatar>', () => {

  it('renders the .root host span with role=img (part="root")', () => {
    const { el } = mount(reg, 'sui-avatar')
    expect(el.shadowRoot).not.toBeNull()
    expect(root(el)).not.toBeNull()
    expect(root(el).getAttribute('part')).toBe('root')
    expect(root(el).getAttribute('role')).toBe('img')
  })

  describe('aria-label fallback chain', () => {
    it('uses the name when provided', () => {
      const { el } = mount(reg, 'sui-avatar', { name: 'Ada Lovelace' })
      expect(root(el).getAttribute('aria-label')).toBe('Ada Lovelace')
    })

    it('falls back to initials when name is absent', () => {
      const { el } = mount(reg, 'sui-avatar', { initials: 'AL' })
      expect(root(el).getAttribute('aria-label')).toBe('AL')
    })

    it('falls back to "avatar" when nothing is provided', () => {
      const { el } = mount(reg, 'sui-avatar')
      expect(root(el).getAttribute('aria-label')).toBe('avatar')
    })
  })

  describe('initials mode (no image)', () => {
    it('shows the initials part, not an img', () => {
      const { el } = mount(reg, 'sui-avatar', { name: 'Grace Hopper' })
      expect(img(el)).toBeNull()
      expect(initials(el)).not.toBeNull()
    })

    it('derives a single initial from the first character of the name', () => {
      const { el } = mount(reg, 'sui-avatar', { name: 'grace' })
      expect(initials(el)!.textContent!.trim()).toBe('G')
    })

    it('uses an explicit initials prop, uppercased and clamped to 2 chars', () => {
      const { el } = mount(reg, 'sui-avatar', { initials: 'abc' })
      expect(initials(el)!.textContent!.trim()).toBe('AB')
    })

    it('renders "?" when neither name nor initials is present', () => {
      const { el } = mount(reg, 'sui-avatar')
      expect(initials(el)!.textContent!.trim()).toBe('?')
    })
  })

  describe('image mode', () => {
    it('renders an <img part="img"> when src is set', () => {
      const { el } = mount(reg, 'sui-avatar', { src: 'https://x.dev/a.png', name: 'Ada' })
      const i = img(el)
      expect(i).not.toBeNull()
      expect(i!.getAttribute('part')).toBe('img')
      expect(i!.getAttribute('src')).toBe('https://x.dev/a.png')
      expect(i!.getAttribute('alt')).toBe('Ada')
      // The initials branch is not rendered while the image is shown.
      expect(initials(el)).toBeNull()
    })

    it('builds the simpleicons CDN url from icon-slug', () => {
      const { el } = mount(reg, 'sui-avatar', { 'icon-slug': 'github' })
      expect(img(el)!.getAttribute('src')).toBe('https://cdn.simpleicons.org/github')
    })
  })

  describe('errored fallback (controller)', () => {
    it('falls back to initials when the image errors', async () => {
      const { el } = mount(reg, 'sui-avatar', { src: 'https://x.dev/broken.png', name: 'Ada' })
      const i = img(el)!
      expect(i).not.toBeNull()
      // Image load failure flips `errored` → template swaps to initials.
      i.dispatchEvent(new Event('error', { bubbles: false }))
      await flush()
      expect(img(el)).toBeNull()
      expect(initials(el)).not.toBeNull()
      expect(initials(el)!.textContent!.trim()).toBe('A')
    })

    it('retries the image when the source changes after an error', async () => {
      const { el } = mount(reg, 'sui-avatar', { src: 'https://x.dev/broken.png', name: 'Ada' })
      img(el)!.dispatchEvent(new Event('error', { bubbles: false }))
      await flush()
      expect(img(el)).toBeNull()
      // A new source resets `errored` and the image branch returns.
      el.setAttribute('src', 'https://x.dev/good.png')
      await flush()
      const i = img(el)
      expect(i).not.toBeNull()
      expect(i!.getAttribute('src')).toBe('https://x.dev/good.png')
    })
  })

  describe('size matrix', () => {
    for (const size of ['xs', 'sm', 'md', 'lg', 'xl']) {
      it(`reflects size="${size}"`, () => {
        const { el } = mount(reg, 'sui-avatar', { size })
        expect(el.getAttribute('size')).toBe(size)
      })
    }
  })

  describe('shape matrix', () => {
    for (const shape of ['round', 'square']) {
      it(`reflects shape="${shape}"`, () => {
        const { el } = mount(reg, 'sui-avatar', { shape })
        expect(el.getAttribute('shape')).toBe(shape)
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
        const { el } = mount(reg, 'sui-avatar', { tone })
        expect(el.getAttribute('tone')).toBe(tone)
      })
    }
  })
})
