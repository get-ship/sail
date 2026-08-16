import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<button popovertarget="profile">@ada</button>
<hover-card id="profile">
  <strong>Ada Lovelace</strong>
</hover-card>`

describe('l1 hover-card rest', () => {
  it('mounts and locates the trigger at rest', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'button', { name: '@ada' })).toBeTruthy()
    v.unmount()
  })
})
