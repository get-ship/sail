import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-card/sui-card.compiled.js'
import { createContainerRegistry, mount, flush } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

describe('<sui-card>', () => {

  describe('non-interactive (default)', () => {
    it('has no role or tabindex when interactive is false', () => {
      const { el } = mount(reg, 'sui-card', {})
      expect(el.getAttribute('role')).toBeNull()
      expect(el.getAttribute('tabindex')).toBeNull()
    })
  })

  describe('interactive=true', () => {
    it('sets role="button" on the host', () => {
      const { el } = mount(reg, 'sui-card', { interactive: true })
      expect(el.getAttribute('role')).toBe('button')
    })

    it('sets tabindex="0" on the host', () => {
      const { el } = mount(reg, 'sui-card', { interactive: true })
      expect(el.getAttribute('tabindex')).toBe('0')
    })

    it('does not override a pre-existing role attribute', () => {
      const container = reg.mk()
      container.innerHTML = '<sui-card interactive role="link"></sui-card>'
      const el = container.querySelector('sui-card')!
      expect(el.getAttribute('role')).toBe('link')
    })

    it('does not override a pre-existing tabindex attribute', () => {
      const container = reg.mk()
      container.innerHTML = '<sui-card interactive tabindex="-1"></sui-card>'
      const el = container.querySelector('sui-card')!
      expect(el.getAttribute('tabindex')).toBe('-1')
    })

    it('fires click on Enter key', () => {
      const { el } = mount(reg, 'sui-card', { interactive: true })
      let clicked = 0
      el.addEventListener('click', () => clicked++)
      el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
      expect(clicked).toBe(1)
    })

    it('fires click on Space key', () => {
      const { el } = mount(reg, 'sui-card', { interactive: true })
      let clicked = 0
      el.addEventListener('click', () => clicked++)
      el.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true }))
      expect(clicked).toBe(1)
    })

    it('does not fire click on other keys', () => {
      const { el } = mount(reg, 'sui-card', { interactive: true })
      let clicked = 0
      el.addEventListener('click', () => clicked++)
      el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true }))
      expect(clicked).toBe(0)
    })
  })

  describe('interactive prop toggled reactively', () => {
    it('removes role and tabindex when interactive is set to false', async () => {
      const { el } = mount(reg, 'sui-card', { interactive: true })
      expect(el.getAttribute('role')).toBe('button')
      el.removeAttribute('interactive')
      await flush()
      expect(el.getAttribute('role')).toBeNull()
      expect(el.getAttribute('tabindex')).toBeNull()
    })

    it('adds role and tabindex when interactive is set to true', async () => {
      const { el } = mount(reg, 'sui-card', {})
      expect(el.getAttribute('role')).toBeNull()
      el.setAttribute('interactive', '')
      await flush()
      expect(el.getAttribute('role')).toBe('button')
      expect(el.getAttribute('tabindex')).toBe('0')
    })

    it('stops firing click on keydown after interactive is removed', async () => {
      const { el } = mount(reg, 'sui-card', { interactive: true })
      el.removeAttribute('interactive')
      await flush()
      let clicked = 0
      el.addEventListener('click', () => clicked++)
      el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
      expect(clicked).toBe(0)
    })
  })
})
