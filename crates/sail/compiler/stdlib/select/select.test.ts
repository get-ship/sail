import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

describe('l1 select', () => {
  it('labeled combobox is locatable', async () => {
    const v = renderRoute(`<label>
  Country
  <select>
    <option>United States</option>
  </select>
</label>`)
    await v.flush()
    expect(getByRole(v.root, 'combobox', { name: 'Country' })).toBeTruthy()
    v.unmount()
  })
})
