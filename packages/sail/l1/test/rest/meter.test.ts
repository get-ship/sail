import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByText } from '@ship/sail-test'

const SOURCE = `<label>
  Disk free
  <meter value="72" min="0" max="100"></meter>
</label>`

describe('l1 meter rest', () => {
  it('mounts and is locatable by labeled gauge', () => {
    const v = renderRoute(SOURCE)
    expect(getByText(v.root, 'Disk free')).toBeTruthy()
    v.unmount()
  })
})
