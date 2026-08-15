import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<label>
  Bio
  <textarea placeholder="Tell us about yourself"></textarea>
</label>`

describe('l1 textarea rest', () => {
  it('mounts and is locatable by field name', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'textbox', { name: 'Bio' })).toBeTruthy()
    v.unmount()
  })
})
