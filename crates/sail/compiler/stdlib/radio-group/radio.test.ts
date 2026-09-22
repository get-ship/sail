import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

describe('l1 radio', () => {
  it('is locatable by name', () => {
    const v = renderRoute(`<radio value="monthly">Monthly</radio>`)
    expect(getByRole(v.root, 'radio', { name: 'Monthly' })).toBeTruthy()
    v.unmount()
  })
})
