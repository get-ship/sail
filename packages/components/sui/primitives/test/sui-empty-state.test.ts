// Characterization spec for <sui-empty-state> — a centered placeholder for
// empty/missing content. No controller: the .box role + aria-live are driven
// by the `variant` expression, and the icon tile auto-hides via the
// `<slot name="icon" reflect-empty="data-no-icon">` runtime helper.

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-empty-state/sui-empty-state.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
// Anchor on the public part contract, not the .box styling class.
const box = (el: HTMLElement) => shadow(el).querySelector('[part~="box"]')!

describe('<sui-empty-state>', () => {

  it('renders a .box (part="box") in the shadow root', () => {
    const { el } = mount(reg, 'sui-empty-state')
    expect(el.shadowRoot).not.toBeNull()
    expect(box(el)).not.toBeNull()
    expect(box(el).getAttribute('part')).toBe('box')
  })

  it('exposes icon / title / desc / actions parts', () => {
    const { el } = mount(reg, 'sui-empty-state')
    expect(shadow(el).querySelector('[part~="icon"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="title"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="desc"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="actions"]')).not.toBeNull()
  })

  describe('default variant a11y', () => {
    it('the box is role=note with aria-live=off', () => {
      const { el } = mount(reg, 'sui-empty-state')
      expect(box(el).getAttribute('role')).toBe('note')
      expect(box(el).getAttribute('aria-live')).toBe('off')
    })
  })

  describe('connecting variant a11y', () => {
    it('the box becomes role=status with aria-live=polite', () => {
      const { el } = mount(reg, 'sui-empty-state', { variant: 'connecting' })
      expect(box(el).getAttribute('role')).toBe('status')
      expect(box(el).getAttribute('aria-live')).toBe('polite')
    })

    it('switching to connecting reactively updates role/aria-live', async () => {
      const { el } = mount(reg, 'sui-empty-state')
      expect(box(el).getAttribute('role')).toBe('note')
      el.setAttribute('variant', 'connecting')
      await flush()
      expect(box(el).getAttribute('role')).toBe('status')
      expect(box(el).getAttribute('aria-live')).toBe('polite')
    })
  })

  describe('variant matrix reflects on host', () => {
    for (const variant of ['default', 'inline', 'placeholder', 'connecting']) {
      it(`reflects variant="${variant}"`, () => {
        const { el } = mount(reg, 'sui-empty-state', { variant })
        expect(el.getAttribute('variant')).toBe(variant)
        expect(box(el)).not.toBeNull()
      })
    }
  })

  describe('size matrix', () => {
    for (const size of ['sm', 'md', 'lg']) {
      it(`reflects size="${size}"`, () => {
        const { el } = mount(reg, 'sui-empty-state', { size })
        expect(el.getAttribute('size')).toBe(size)
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
        const { el } = mount(reg, 'sui-empty-state', { tone })
        expect(el.getAttribute('tone')).toBe(tone)
      })
    }
  })

  describe('icon-tile reflect-empty', () => {
    it('marks the host data-no-icon when the icon slot is empty', async () => {
      const { el } = mount(reg, 'sui-empty-state', {}, '<span slot="title">Nothing here</span>')
      await flush()
      expect(el.hasAttribute('data-no-icon')).toBe(true)
    })

    it('clears data-no-icon when the icon slot has content', async () => {
      // `reflect-empty` counts an element as content only when it has trimmed
      // text OR element children — an svg with a child path qualifies.
      const { el } = mount(
        reg,
        'sui-empty-state',
        {},
        '<svg slot="icon"><path d="M0 0"/></svg><span slot="title">Empty</span>',
      )
      await flush()
      expect(el.hasAttribute('data-no-icon')).toBe(false)
    })
  })

  it('projects title / description / action slot content', async () => {
    const { el } = mount(
      reg,
      'sui-empty-state',
      {},
      '<span slot="title">No results</span>Try a different query<button slot="action">Reset</button>',
    )
    await flush()
    expect(el.textContent).toContain('No results')
    expect(el.textContent).toContain('Try a different query')
    expect(el.textContent).toContain('Reset')
    expect(shadow(el).querySelector('slot[name="title"]')).not.toBeNull()
    expect(shadow(el).querySelector('slot[name="action"]')).not.toBeNull()
  })
})
