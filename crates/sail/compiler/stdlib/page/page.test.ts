import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

describe('l1 page', () => {
  it('a child heading is locatable inside the document region', () => {
    const v = renderRoute(`<page>
  <h1>The Harbor Gazette</h1>
</page>`)
    expect(getByRole(v.root, 'heading', { name: 'The Harbor Gazette', level: 1 })).toBeTruthy()
    expect(getByRole(v.root, 'region', { name: 'Page' })).toBeTruthy()
    v.unmount()
  })
})
