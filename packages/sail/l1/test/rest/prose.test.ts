import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<prose>
  <h2>Release notes</h2>
  <p>The build went green.</p>
</prose>`

describe('l1 prose rest', () => {
  it('mounts and is locatable by heading', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'heading', { name: 'Release notes', level: 2 })).toBeTruthy()
    v.unmount()
  })
})
