import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<label>
  Brand color
  <color value="#c8f542" />
</label>`

describe('l1 color rest', () => {
  it('mounts and is locatable by field name', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'textbox', { name: 'Brand color' })).toBeTruthy()
    v.unmount()
  })
})
