import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText, queryByRole, queryByText } from '@ship/sail-test'

const ROUTE = `<table>
  <thead>
    <tr><th>Region</th><th numeric>Revenue</th></tr>
  </thead>
  <tbody>
    <tr><td>North</td><td numeric>1,200</td></tr>
    <tr><td>South</td><td numeric>980</td></tr>
  </tbody>
</table>`

describe('l1 table', () => {
  it('is a single native table hosting slotted rows', async () => {
    const v = renderRoute(ROUTE)
    await v.flush()
    expect(getByRole(v.root, 'table')).toBeTruthy()
    expect(getByText(v.root, 'Region')).toBeTruthy()
    expect(getByText(v.root, 'Revenue')).toBeTruthy()
    expect(getByText(v.root, 'North')).toBeTruthy()
    expect(getByText(v.root, '980')).toBeTruthy()
    v.unmount()
  })

  it('ships no search field — searchable was deleted with tableQuery', async () => {
    const v = renderRoute(ROUTE)
    await v.flush()
    expect(queryByRole(v.root, 'searchbox')).toBeNull()
    expect(queryByText(v.root, 'Search')).toBeNull()
    v.unmount()
  })
})
