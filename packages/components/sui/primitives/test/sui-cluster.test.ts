import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-cluster/sui-cluster.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()

afterEach(() => {
  reg.cleanup()
})

describe('<sui-cluster>', () => {

  it('renders a shadow root with a default slot', async () => {
    const { el } = mount(reg, 'sui-cluster', {}, '<span>a</span><span>b</span>')
    await flush()
    expect(el.shadowRoot).not.toBeNull()
    expect(el.shadowRoot!.querySelector('slot')).not.toBeNull()
  })

  it('reflects gap, align, and justify attributes', async () => {
    const { el } = mount(reg, 'sui-cluster', {
      gap: '2',
      align: 'baseline',
      justify: 'between',
    })
    await flush()
    expect(el.getAttribute('gap')).toBe('2')
    expect(el.getAttribute('align')).toBe('baseline')
    expect(el.getAttribute('justify')).toBe('between')
  })

  it('wraps the slot in a single .cluster container', async () => {
    const { el } = mount(reg, 'sui-cluster')
    await flush()
    const inner = el.shadowRoot!.querySelector('.cluster') as HTMLElement
    expect(inner).not.toBeNull()
    expect(inner.querySelector('slot')).not.toBeNull()
  })
})
