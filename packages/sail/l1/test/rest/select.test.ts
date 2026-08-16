import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<label>
  Team
  <select>
    <option>Engineering</option>
    <option>Design</option>
  </select>
</label>`

describe('l1 select rest', () => {
  it('mounts and is locatable by combobox name', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'combobox', { name: 'Team' })).toBeTruthy()
    v.unmount()
  })
})
