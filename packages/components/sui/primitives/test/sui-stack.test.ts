import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-stack/sui-stack.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()

afterEach(() => {
  reg.cleanup()
})

describe('<sui-stack>', () => {

  it('renders a shadow root with a default slot', async () => {
    const { el } = mount(reg, 'sui-stack', {}, '<p>a</p><p>b</p>')
    await flush()
    expect(el.shadowRoot).not.toBeNull()
    expect(el.shadowRoot!.querySelector('slot')).not.toBeNull()
  })

  it('reflects gap and align attributes', async () => {
    const { el } = mount(reg, 'sui-stack', { gap: '6', align: 'center' })
    await flush()
    expect(el.getAttribute('gap')).toBe('6')
    expect(el.getAttribute('align')).toBe('center')
  })

  it('wraps the slot in a single .stack container', async () => {
    const { el } = mount(reg, 'sui-stack')
    await flush()
    const inner = el.shadowRoot!.querySelector('.stack') as HTMLElement
    expect(inner).not.toBeNull()
    expect(inner.querySelector('slot')).not.toBeNull()
  })
})
