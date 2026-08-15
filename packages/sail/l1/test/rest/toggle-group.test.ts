import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<toggle-group>
  <toggle value="left">Left</toggle>
  <toggle value="center">Center</toggle>
  <toggle value="right">Right</toggle>
</toggle-group>`

describe('l1 toggle-group rest', () => {
  it('mounts and is locatable by toggle names', async () => {
    const v = renderRoute(SOURCE)
    await v.flush()
    expect(getByRole(v.root, 'button', { name: 'Left' })).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Center' })).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Right' })).toBeTruthy()
    v.unmount()
  })
})
