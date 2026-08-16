import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText } from '@ship/sail-test'

describe('l1 toast', () => {
  it('region and toast are locatable', () => {
    const v = renderRoute(`<toast-region>
  <toast>
    <strong>Saved.</strong> Your changes are live.
  </toast>
</toast-region>`)
    expect(getByRole(v.root, 'region')).toBeTruthy()
    expect(getByRole(v.root, 'status')).toBeTruthy()
    expect(getByText(v.root, 'Saved.')).toBeTruthy()
    v.unmount()
  })
})
