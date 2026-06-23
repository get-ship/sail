// Characterization spec for <sui-chip> — an interactive filter/selection pill
// whose HOST is the interactive surface. The paired controller sets
// role=button + tabindex on mount, syncs aria-pressed / aria-disabled to the
// selected / disabled signals, activates on Enter/Space, and dispatches
// `ship-remove` on the × button or Backspace/Delete.

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-chip/sui-chip.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const press = (el: HTMLElement, key: string) =>
  el.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true }))

describe('<sui-chip>', () => {

  it('renders a .chip (part="chip") in the shadow root', () => {
    const { el } = mount(reg, 'sui-chip')
    expect(el.shadowRoot).not.toBeNull()
    const chip = shadow(el).querySelector('.chip')
    expect(chip).not.toBeNull()
    expect(chip!.getAttribute('part')).toBe('chip')
  })

  describe('host interactivity (controller mount)', () => {
    it('sets role=button on the host', () => {
      const { el } = mount(reg, 'sui-chip')
      expect(el.getAttribute('role')).toBe('button')
    })

    it('sets tabindex=0 when enabled', () => {
      const { el } = mount(reg, 'sui-chip')
      expect(el.getAttribute('tabindex')).toBe('0')
    })

    it('does not override a pre-existing role', () => {
      const container = reg.mk()
      container.innerHTML = '<sui-chip role="switch"></sui-chip>'
      const el = container.querySelector('sui-chip')!
      expect(el.getAttribute('role')).toBe('switch')
    })

    it('starts at tabindex=-1 (not focusable) when disabled', () => {
      const { el } = mount(reg, 'sui-chip', { disabled: true })
      expect(el.getAttribute('tabindex')).toBe('-1')
    })
  })

  describe('aria-pressed sync', () => {
    it('reflects aria-pressed=false by default', () => {
      const { el } = mount(reg, 'sui-chip')
      expect(el.getAttribute('aria-pressed')).toBe('false')
    })

    it('reflects aria-pressed=true when selected', () => {
      const { el } = mount(reg, 'sui-chip', { selected: true })
      expect(el.getAttribute('aria-pressed')).toBe('true')
    })

    it('updates aria-pressed reactively when selected toggles', async () => {
      const { el } = mount(reg, 'sui-chip')
      expect(el.getAttribute('aria-pressed')).toBe('false')
      el.setAttribute('selected', '')
      await flush()
      expect(el.getAttribute('aria-pressed')).toBe('true')
      el.removeAttribute('selected')
      await flush()
      expect(el.getAttribute('aria-pressed')).toBe('false')
    })
  })

  describe('aria-disabled sync', () => {
    it('sets aria-disabled + tabindex=-1 when disabled', () => {
      const { el } = mount(reg, 'sui-chip', { disabled: true })
      expect(el.getAttribute('aria-disabled')).toBe('true')
      expect(el.getAttribute('tabindex')).toBe('-1')
    })

    it('clears aria-disabled and restores tabindex when re-enabled', async () => {
      const { el } = mount(reg, 'sui-chip', { disabled: true })
      el.removeAttribute('disabled')
      await flush()
      expect(el.hasAttribute('aria-disabled')).toBe(false)
      expect(el.getAttribute('tabindex')).toBe('0')
    })
  })

  describe('keyboard activation', () => {
    it('clicks the host on Enter', () => {
      const { el } = mount(reg, 'sui-chip')
      let clicked = 0
      el.addEventListener('click', () => clicked++)
      press(el, 'Enter')
      expect(clicked).toBe(1)
    })

    it('clicks the host on Space', () => {
      const { el } = mount(reg, 'sui-chip')
      let clicked = 0
      el.addEventListener('click', () => clicked++)
      press(el, ' ')
      expect(clicked).toBe(1)
    })

    it('does not activate on unrelated keys', () => {
      const { el } = mount(reg, 'sui-chip')
      let clicked = 0
      el.addEventListener('click', () => clicked++)
      press(el, 'Tab')
      expect(clicked).toBe(0)
    })

    it('ignores keyboard activation when disabled', () => {
      const { el } = mount(reg, 'sui-chip', { disabled: true })
      let clicked = 0
      el.addEventListener('click', () => clicked++)
      press(el, 'Enter')
      expect(clicked).toBe(0)
    })
  })

  describe('removable', () => {
    it('renders the × remove button (part="remove") only when removable', () => {
      const plain = mount(reg, 'sui-chip')
      expect(shadow(plain.el).querySelector('.remove')).toBeNull()
      const removable = mount(reg, 'sui-chip', { removable: true })
      const btn = shadow(removable.el).querySelector('.remove')
      expect(btn).not.toBeNull()
      expect(btn!.getAttribute('part')).toBe('remove')
      expect(btn!.getAttribute('aria-label')).toBe('Remove')
    })

    it('dispatches a bubbling/composed ship-remove with empty-object detail on × click', () => {
      const { el } = mount(reg, 'sui-chip', { removable: true })
      let evt: CustomEvent | null = null
      el.addEventListener('ship-remove', (e) => { evt = e as CustomEvent })
      const btn = shadow(el).querySelector('.remove') as HTMLButtonElement
      btn.click()
      expect(evt).not.toBeNull()
      expect(evt!.bubbles).toBe(true)
      expect(evt!.composed).toBe(true)
      expect(evt!.detail).toEqual({})
    })

    it('dispatches ship-remove on Backspace when removable', () => {
      const { el } = mount(reg, 'sui-chip', { removable: true })
      let removed = 0
      el.addEventListener('ship-remove', () => removed++)
      press(el, 'Backspace')
      expect(removed).toBe(1)
    })

    it('dispatches ship-remove on Delete when removable', () => {
      const { el } = mount(reg, 'sui-chip', { removable: true })
      let removed = 0
      el.addEventListener('ship-remove', () => removed++)
      press(el, 'Delete')
      expect(removed).toBe(1)
    })

    it('does not dispatch ship-remove on Backspace when not removable', () => {
      const { el } = mount(reg, 'sui-chip')
      let removed = 0
      el.addEventListener('ship-remove', () => removed++)
      press(el, 'Backspace')
      expect(removed).toBe(0)
    })

    it('does not dispatch ship-remove from the × when disabled', () => {
      const { el } = mount(reg, 'sui-chip', { removable: true, disabled: true })
      let removed = 0
      el.addEventListener('ship-remove', () => removed++)
      const btn = shadow(el).querySelector('.remove') as HTMLButtonElement
      btn.click()
      expect(removed).toBe(0)
    })
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
        const { el } = mount(reg, 'sui-chip', { tone })
        expect(el.getAttribute('tone')).toBe(tone)
      })
    }
  })

  describe('variant matrix', () => {
    for (const variant of ['solid', 'soft', 'outline', 'ghost']) {
      it(`reflects variant="${variant}"`, () => {
        const { el } = mount(reg, 'sui-chip', { variant })
        expect(el.getAttribute('variant')).toBe(variant)
      })
    }
  })

  describe('size matrix', () => {
    for (const size of ['sm', 'md', 'lg']) {
      it(`reflects size="${size}"`, () => {
        const { el } = mount(reg, 'sui-chip', { size })
        expect(el.getAttribute('size')).toBe(size)
      })
    }
  })

  it('exposes a leading slot wrapper (part="leading")', () => {
    const { el } = mount(reg, 'sui-chip')
    const leading = shadow(el).querySelector('.leading')
    expect(leading).not.toBeNull()
    expect(leading!.getAttribute('part')).toBe('leading')
    expect(leading!.querySelector('slot[name="leading"]')).not.toBeNull()
  })

  it('projects default-slot label content', () => {
    const { el } = mount(reg, 'sui-chip', {}, 'Filter')
    expect(el.textContent).toContain('Filter')
  })
})
