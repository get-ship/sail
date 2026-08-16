import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText } from '@ship/sail-test'

const SOURCE = `<table>
  <thead>
    <tr><th>Name</th><th>Team</th></tr>
  </thead>
  <tbody>
    <tr><td>Ada Lovelace</td><td>Engines</td></tr>
  </tbody>
</table>`

describe('l1 table rest', () => {
  it('mounts and is locatable by table and cell', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'table')).toBeTruthy()
    expect(getByText(v.root, 'Ada Lovelace')).toBeTruthy()
    v.unmount()
  })
})
