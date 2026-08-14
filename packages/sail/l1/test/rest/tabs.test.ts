import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { click, getByRole } from '@ship/sail-test'

const SOURCE = `<tabs active="one">
  <tab-list>
    <tab name="one">One</tab>
    <tab name="two">Two</tab>
  </tab-list>
  <tab-panel name="one">Panel one</tab-panel>
  <tab-panel name="two">Panel two</tab-panel>
</tabs>`

describe('l1 tabs rest', () => {
  it('mounts and is locatable by tab name', async () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'tab', { name: 'One', selected: true })).toBeTruthy()
    expect(getByRole(v.root, 'tab', { name: 'Two' })).toBeTruthy()
    click(getByRole(v.root, 'tab', { name: 'Two' }))
    await v.flush()
    expect(getByRole(v.root, 'tab', { name: 'Two', selected: true })).toBeTruthy()
    v.unmount()
  })
})
