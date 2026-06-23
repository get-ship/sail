import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-tab-panel/sui-tab-panel.compiled.js'
import '../src/sui-tabs/sui-tabs.compiled.js'
import '../src/sui-tabs/sui-tab.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()

afterEach(() => {
  reg.cleanup()
})

describe('<sui-tab-panel>', () => {

  it('declares a shadow root with a default slot', async () => {
    const { el } = mount(reg, 'sui-tab-panel', { name: 'one' }, 'panel body')
    await flush()
    expect(el.shadowRoot).not.toBeNull()
    expect(el.shadowRoot!.querySelector('slot')).not.toBeNull()
  })

  it('sets role=tabpanel + tabindex=0 + aria-labelledby on mount', async () => {
    const { el } = mount(reg, 'sui-tab-panel', { name: 'overview' })
    await flush()
    expect(el.getAttribute('role')).toBe('tabpanel')
    expect(el.getAttribute('tabindex')).toBe('0')
    expect(el.getAttribute('aria-labelledby')).toBe('sui-tab-overview')
    expect(el.id).toBe('sui-tabpanel-overview')
  })

  it('respects an externally-set role and tabindex', async () => {
    const { container } = mount(reg, 'div')
    container.innerHTML =
      '<sui-tab-panel name="x" role="region" tabindex="-1"></sui-tab-panel>'
    const el = container.querySelector('sui-tab-panel')! as HTMLElement
    await flush()
    expect(el.getAttribute('role')).toBe('region')
    expect(el.getAttribute('tabindex')).toBe('-1')
  })

  it('parent sui-tabs activates the matching panel by name', async () => {
    const { container } = mount(reg, 'div')
    container.innerHTML = `
      <sui-tabs active="one">
        <sui-tab name="one" label="One"></sui-tab>
        <sui-tab name="two" label="Two"></sui-tab>
        <sui-tab-panel name="one">first</sui-tab-panel>
        <sui-tab-panel name="two">second</sui-tab-panel>
      </sui-tabs>
    `
    await flush()
    const panels = container.querySelectorAll('sui-tab-panel')
    expect(panels[0]!.hasAttribute('active')).toBe(true)
    expect(panels[1]!.hasAttribute('active')).toBe(false)
    expect(panels[1]!.hasAttribute('hidden')).toBe(true)

    container.querySelector('sui-tabs')!.setAttribute('active', 'two')
    await flush()
    expect(panels[0]!.hasAttribute('active')).toBe(false)
    expect(panels[1]!.hasAttribute('active')).toBe(true)
  })
})
