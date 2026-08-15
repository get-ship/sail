import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<radio-group label="Billing period" name="plan" value="monthly">
  <radio value="monthly">Monthly</radio>
  <radio value="yearly">Yearly</radio>
</radio-group>`

describe('l1 radio-group rest', () => {
  it('mounts and is locatable by group and option', async () => {
    const v = renderRoute(SOURCE)
    await v.flush()
    expect(getByRole(v.root, 'radiogroup')).toBeTruthy()
    expect(getByRole(v.root, 'radio', { name: 'Monthly' })).toBeTruthy()
    expect(getByRole(v.root, 'radio', { name: 'Yearly' })).toBeTruthy()
    v.unmount()
  })
})
