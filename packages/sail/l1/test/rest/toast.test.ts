import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText } from '@ship/sail-test'

const SOURCE = `<toast-region>
  <toast>
    <strong>Saved.</strong> Your changes are live.
  </toast>
</toast-region>`

describe('l1 toast rest', () => {
  it('mounts and is locatable by notice and dismiss', () => {
    const v = renderRoute(SOURCE)
    expect(getByText(v.root, 'Saved.')).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Dismiss' })).toBeTruthy()
    v.unmount()
  })
})
