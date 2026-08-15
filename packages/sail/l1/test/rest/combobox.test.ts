import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<combobox placeholder="Pick a fruit">
  <option value="apple">Apple</option>
  <option value="banana">Banana</option>
</combobox>`

describe('l1 combobox rest', () => {
  it('mounts and is locatable by combobox name', async () => {
    const v = renderRoute(SOURCE)
    await v.flush()
    expect(getByRole(v.root, 'combobox', { name: 'Pick a fruit' })).toBeTruthy()
    v.unmount()
  })
})
