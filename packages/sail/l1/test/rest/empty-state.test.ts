import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText } from '@ship/sail-test'

const SOURCE = `<empty-state>
  <strong slot="title">No results</strong>
  Try a different search.
  <button slot="action" variant="outline" size="sm">Clear filters</button>
</empty-state>`

describe('l1 empty-state rest', () => {
  it('mounts and is locatable by title and action', () => {
    const v = renderRoute(SOURCE)
    expect(getByText(v.root, 'No results')).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Clear filters' })).toBeTruthy()
    v.unmount()
  })
})
