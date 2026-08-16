import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<button popovertarget="pop">Open popover</button>
<popover id="pop">
  Rides the platform top layer.
</popover>`

describe('l1 popover rest', () => {
  it('mounts and locates the trigger at rest', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'button', { name: 'Open popover' })).toBeTruthy()
    v.unmount()
  })
})
