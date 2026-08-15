import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<label>
  <checkbox />
  I agree
</label>`

describe('l1 checkbox rest', () => {
  it('mounts and is locatable by checkbox name', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'checkbox', { name: 'I agree' })).toBeTruthy()
    v.unmount()
  })
})
