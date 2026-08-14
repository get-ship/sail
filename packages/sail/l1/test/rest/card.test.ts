import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<card>
  <h2 slot="header">Overview</h2>
  <p>Surface, radius, elevation on the element.</p>
</card>`

describe('l1 card rest', () => {
  it('mounts and is locatable by heading', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'heading', { name: 'Overview', level: 2 })).toBeTruthy()
    v.unmount()
  })
})
