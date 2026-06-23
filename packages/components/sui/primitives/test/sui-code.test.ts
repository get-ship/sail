import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-code/sui-code.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-code>', () => {

  it('renders inline variant by default', async () => {
    const { el } = mount(reg, 'sui-code', {}, 'foo')
    await flush()
    expect(shadow(el).querySelector('code.inline')).not.toBeNull()
    expect(shadow(el).querySelector('.block')).toBeNull()
  })

  it('block attribute switches to block variant', async () => {
    const { el } = mount(reg, 'sui-code', { block: true }, 'const x = 1')
    await flush()
    expect(shadow(el).querySelector('.block')).not.toBeNull()
    expect(shadow(el).querySelector('pre')).not.toBeNull()
    expect(shadow(el).querySelector('code.inline')).toBeNull()
  })

  it('block with lang renders header', async () => {
    const { el } = mount(reg, 'sui-code', { block: true, lang: 'ts' }, 'const x = 1')
    await flush()
    const header = shadow(el).querySelector('header[part="label"]')
    expect(header).not.toBeNull()
    expect(header!.textContent?.trim()).toBe('ts')
  })

  it('block without lang omits header', async () => {
    const { el } = mount(reg, 'sui-code', { block: true }, 'some code')
    await flush()
    expect(shadow(el).querySelector('header')).toBeNull()
  })

  it('block has aria-label with lang', async () => {
    const { el } = mount(reg, 'sui-code', { block: true, lang: 'js' }, 'x()')
    await flush()
    const div = shadow(el).querySelector('[part="block"]')!
    expect(div.getAttribute('aria-label')).toBe('js code block')
  })

  it('block has default aria-label without lang', async () => {
    const { el } = mount(reg, 'sui-code', { block: true }, 'x()')
    await flush()
    const div = shadow(el).querySelector('[part="block"]')!
    expect(div.getAttribute('aria-label')).toBe('code block')
  })

  it('block has role=region', async () => {
    const { el } = mount(reg, 'sui-code', { block: true }, 'x()')
    await flush()
    const div = shadow(el).querySelector('[part="block"]')!
    expect(div.getAttribute('role')).toBe('region')
  })

  it('reacts to block attribute toggle', async () => {
    const { el } = mount(reg, 'sui-code', {}, 'x')
    await flush()
    expect(shadow(el).querySelector('code.inline')).not.toBeNull()
    el.setAttribute('block', '')
    await flush()
    expect(shadow(el).querySelector('pre')).not.toBeNull()
  })

  it('reacts to lang attribute change', async () => {
    const { el } = mount(reg, 'sui-code', { block: true }, 'x')
    await flush()
    expect(shadow(el).querySelector('header')).toBeNull()
    el.setAttribute('lang', 'rust')
    await flush()
    expect(shadow(el).querySelector('header')?.textContent?.trim()).toBe('rust')
  })

  it('pre part is rendered inside block', async () => {
    const { el } = mount(reg, 'sui-code', { block: true }, 'hello')
    await flush()
    expect(shadow(el).querySelector('pre[part="pre"]')).not.toBeNull()
  })
})
