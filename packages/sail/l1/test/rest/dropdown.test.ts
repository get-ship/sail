import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<button popovertarget="menu">Actions</button>
<dropdown id="menu">
  <menu-item>Edit</menu-item>
  <menu-item>Duplicate</menu-item>
</dropdown>`

describe('l1 dropdown rest', () => {
  it('mounts and locates the trigger at rest', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'button', { name: 'Actions' })).toBeTruthy()
    v.unmount()
  })
})
