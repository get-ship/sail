// Minimal behavioral spec for <sui-brand-mark> — the Ship logo mark. Asserts
// it renders, reflects size, and wires the labelling a11y contract (decorative
// by default, role=img + aria-label when a label is supplied).

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-brand-mark/sui-brand-mark.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

describe('<sui-brand-mark>', () => {

  it('renders an svg in the shadow root', async () => {
    const { el } = mount(reg, 'sui-brand-mark')
    await flush()
    expect(el.shadowRoot!.querySelector('svg')).not.toBeNull()
  })

  it('reflects the size prop on the host', () => {
    for (const size of ['sm', 'md', 'lg']) {
      const { el } = mount(reg, 'sui-brand-mark', { size })
      expect(el.getAttribute('size')).toBe(size)
    }
  })

  it('is decorative (aria-hidden, no role) when no label is given', async () => {
    const { el } = mount(reg, 'sui-brand-mark')
    await flush()
    expect(el.getAttribute('aria-hidden')).toBe('true')
    expect(el.hasAttribute('role')).toBe(false)
  })

  it('becomes role=img with an aria-label when label is set', async () => {
    const { el } = mount(reg, 'sui-brand-mark', { label: 'Ship' })
    await flush()
    expect(el.getAttribute('role')).toBe('img')
    expect(el.getAttribute('aria-label')).toBe('Ship')
    expect(el.hasAttribute('aria-hidden')).toBe(false)
  })

  it('toggles labelling reactively when the label prop changes', async () => {
    const { el } = mount(reg, 'sui-brand-mark', { label: 'Ship' })
    await flush()
    expect(el.getAttribute('role')).toBe('img')
    el.setAttribute('label', '')
    await flush()
    expect(el.getAttribute('aria-hidden')).toBe('true')
    expect(el.hasAttribute('role')).toBe(false)
  })
})
