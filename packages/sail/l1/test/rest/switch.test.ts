import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<label>
  Notifications
  <switch></switch>
</label>`

describe('l1 switch rest', () => {
  it('mounts and is locatable by switch name', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'switch', { name: 'Notifications' })).toBeTruthy()
    v.unmount()
  })
})
