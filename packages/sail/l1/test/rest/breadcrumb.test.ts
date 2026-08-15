import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<breadcrumb>
  <a href="/">Home</a>
  <a href="/components">Components</a>
  <small>breadcrumb</small>
</breadcrumb>`

describe('l1 breadcrumb rest', () => {
  it('mounts and is locatable by navigation name', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'navigation', { name: 'Breadcrumb' })).toBeTruthy()
    expect(getByRole(v.root, 'link', { name: 'Home' })).toBeTruthy()
    v.unmount()
  })
})
