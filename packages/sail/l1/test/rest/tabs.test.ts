import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { click, getByRole } from '@ship/sail-test'

const SOURCE = `<tabs>
  <tab-list>
    <tab>One</tab>
    <tab>Two</tab>
  </tab-list>
  <tab-panel>Panel one</tab-panel>
  <tab-panel>Panel two</tab-panel>
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

  it('pairs tabs and panels by position without name attrs', async () => {
    const v = renderRoute(SOURCE)
    const one = getByRole(v.root, 'tabpanel', { name: 'One' })
    expect(one.hidden).toBe(false)
    expect(one.textContent).toContain('Panel one')
    click(getByRole(v.root, 'tab', { name: 'Two' }))
    await v.flush()
    expect(getByRole(v.root, 'tabpanel', { name: 'Two' }).hidden).toBe(false)
    expect(getByRole(v.root, 'tabpanel', { name: 'Two' }).textContent).toContain('Panel two')
    v.unmount()
  })

  it('orientation=vertical stamps list/tab/panel and sets tablist ARIA', () => {
    const v = renderRoute(`<tabs orientation="vertical">
  <tab-list>
    <tab>One</tab>
    <tab>Two</tab>
  </tab-list>
  <tab-panel>Panel one</tab-panel>
  <tab-panel>Panel two</tab-panel>
</tabs>`)
    const list = getByRole(v.root, 'tablist')
    expect(list.getAttribute('orientation')).toBe('vertical')
    expect(list.getAttribute('aria-orientation')).toBe('vertical')
    expect(getByRole(v.root, 'tab', { name: 'One' }).getAttribute('orientation')).toBe('vertical')
    expect(getByRole(v.root, 'tabpanel', { name: 'One' }).getAttribute('orientation')).toBe(
      'vertical',
    )
    v.unmount()
  })
})
