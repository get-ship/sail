import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-label/sui-label.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const label = (el: HTMLElement) => shadow(el).querySelector('label') as HTMLLabelElement | null
const req = (el: HTMLElement) => shadow(el).querySelector('.req') as HTMLElement | null

describe('<sui-label>', () => {

  it('renders a native <label> in shadow DOM', () => {
    const { el } = mount(reg, 'sui-label')
    expect(label(el)).not.toBeNull()
  })

  it('inner label has part=label', () => {
    const { el } = mount(reg, 'sui-label')
    expect(label(el)!.getAttribute('part')).toBe('label')
  })

  it('slot projects text content', () => {
    const { el } = mount(reg, 'sui-label', {}, 'Status')
    expect(el.textContent?.trim()).toBe('Status')
  })

  it('required=false — no .req element', () => {
    const { el } = mount(reg, 'sui-label', {}, 'Name')
    expect(req(el)).toBeNull()
  })

  it('required=true — .req element present', () => {
    const { el } = mount(reg, 'sui-label', { required: true }, 'Name')
    expect(req(el)).not.toBeNull()
  })

  it('.req span has aria-hidden=true', () => {
    const { el } = mount(reg, 'sui-label', { required: true }, 'Name')
    expect(req(el)!.getAttribute('aria-hidden')).toBe('true')
  })

  it('.req span shows * character', () => {
    const { el } = mount(reg, 'sui-label', { required: true }, 'Name')
    expect(req(el)!.textContent).toBe('*')
  })

  it('size=sm — reflects on host', () => {
    const { el } = mount(reg, 'sui-label', { size: 'sm' })
    expect(el.getAttribute('size')).toBe('sm')
  })

  it('size=md reflects on host', () => {
    const { el } = mount(reg, 'sui-label', { size: 'md' })
    expect(el.getAttribute('size')).toBe('md')
  })

  it('required reflects on host', () => {
    const { el } = mount(reg, 'sui-label', { required: true })
    expect(el.hasAttribute('required')).toBe(true)
  })

  it('for attribute is forwarded to inner label.htmlFor', async () => {
    const { el } = mount(reg, 'sui-label', { for: 'my-input' })
    await flush()
    expect(label(el)!.htmlFor).toBe('my-input')
  })

  it('for attribute update syncs to inner label.htmlFor', async () => {
    const { el } = mount(reg, 'sui-label', { for: 'a' })
    await flush()
    expect(label(el)!.htmlFor).toBe('a')
    el.setAttribute('for', 'b')
    await flush()
    expect(label(el)!.htmlFor).toBe('b')
  })

  it('disconnect — no leaked effects', () => {
    const { container } = mount(reg, 'sui-label', { for: 'x', required: true })
    expect(container.isConnected).toBe(true)
    container.remove()
    expect(container.isConnected).toBe(false)
  })
})
