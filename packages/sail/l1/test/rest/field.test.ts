import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<field>
  <label for="display-name">Name</label>
  <input id="display-name" placeholder="Ada Lovelace" />
</field>`

describe('l1 field rest', () => {
  it('mounts and is locatable by labeled control', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'textbox', { name: 'Name' })).toBeTruthy()
    v.unmount()
  })
})
