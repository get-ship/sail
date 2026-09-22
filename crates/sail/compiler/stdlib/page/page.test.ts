import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText } from '@ship/sail-test'

describe('l1 page', () => {
  it('stacks children inside the named document region', () => {
    const v = renderRoute(`<page>
  <h1>The Harbor Gazette</h1>
  <h2>Shipping news</h2>
  <p slot="route">Arrivals board</p>
</page>`)
    const page = getByRole(v.root, 'region', { name: 'Page' })
    expect(getByRole(page, 'heading', { name: 'The Harbor Gazette', level: 1 })).toBeTruthy()
    expect(getByRole(page, 'heading', { name: 'Shipping news', level: 2 })).toBeTruthy()
    expect(getByText(page, 'Arrivals board')).toBeTruthy()
    v.unmount()
  })
})
