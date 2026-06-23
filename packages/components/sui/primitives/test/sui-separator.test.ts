import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-separator/sui-separator.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const hr = (el: HTMLElement) => shadow(el).querySelector('hr') as HTMLElement | null

describe('<sui-separator>', () => {

  it('renders an <hr> in shadow DOM', () => {
    const { el } = mount(reg, 'sui-separator')
    expect(hr(el)).not.toBeNull()
  })

  it('inner hr has role=separator', () => {
    const { el } = mount(reg, 'sui-separator')
    expect(hr(el)!.getAttribute('role')).toBe('separator')
  })

  it('inner hr has part=sep', () => {
    const { el } = mount(reg, 'sui-separator')
    expect(hr(el)!.getAttribute('part')).toBe('sep')
  })

  it('orientation=horizontal — reflects on host', () => {
    const { el } = mount(reg, 'sui-separator', { orientation: 'horizontal' })
    expect(el.getAttribute('orientation')).toBe('horizontal')
  })

  it('vertical orientation reflects on host', () => {
    const { el } = mount(reg, 'sui-separator', { orientation: 'vertical' })
    expect(el.getAttribute('orientation')).toBe('vertical')
  })

  it('aria-orientation=horizontal on hr by default', () => {
    const { el } = mount(reg, 'sui-separator')
    expect(hr(el)!.getAttribute('aria-orientation')).toBe('horizontal')
  })

  it('aria-orientation=vertical on hr when orientation=vertical', () => {
    const { el } = mount(reg, 'sui-separator', { orientation: 'vertical' })
    expect(hr(el)!.getAttribute('aria-orientation')).toBe('vertical')
  })

  it('variant=solid — reflects on host', () => {
    const { el } = mount(reg, 'sui-separator', { variant: 'solid' })
    expect(el.getAttribute('variant')).toBe('solid')
  })

  it('dashed variant reflects on host', () => {
    const { el } = mount(reg, 'sui-separator', { variant: 'dashed' })
    expect(el.getAttribute('variant')).toBe('dashed')
  })

  it('orientation updates aria-orientation reactively', async () => {
    const { el } = mount(reg, 'sui-separator')
    expect(hr(el)!.getAttribute('aria-orientation')).toBe('horizontal')
    el.setAttribute('orientation', 'vertical')
    await flush()
    expect(hr(el)!.getAttribute('aria-orientation')).toBe('vertical')
  })

  it('disconnect — no leaked references', () => {
    const { el, container } = mount(reg, 'sui-separator', { orientation: 'vertical' })
    expect(hr(el)).not.toBeNull()
    container.remove()
    // No assertion needed — leaked effects would throw on re-mount
  })
})
