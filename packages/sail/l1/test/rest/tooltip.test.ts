import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<tooltip content="Deletes the branch">
  <button tone="destructive" variant="outline">Delete branch</button>
</tooltip>`

describe('l1 tooltip rest', () => {
  it('mounts and locates the trigger at rest', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'button', { name: 'Delete branch' })).toBeTruthy()
    v.unmount()
  })
})
