import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<label>
  Start date
  <date />
</label>`

describe('l1 date rest', () => {
  it('mounts and is locatable by field name', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'textbox', { name: 'Start date' })).toBeTruthy()
    v.unmount()
  })
})
