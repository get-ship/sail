import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { click, getByRole } from '@ship/sail-test'

const SOURCE = `<radio-group label="Billing period" name="plan" value="monthly">
  <radio value="monthly">Monthly</radio>
  <radio value="yearly">Yearly</radio>
</radio-group>`

describe('l1 radio-group', () => {
  it('is a group whose radios are locatable', async () => {
    const v = renderRoute(SOURCE)
    await v.flush()
    expect(getByRole(v.root, 'radiogroup')).toBeTruthy()
    expect(getByRole(v.root, 'radio', { name: 'Monthly', checked: true })).toBeTruthy()
    expect(getByRole(v.root, 'radio', { name: 'Yearly', checked: false })).toBeTruthy()
    v.unmount()
  })

  it('selecting one is the contract', async () => {
    const v = renderRoute(SOURCE)
    await v.flush()
    click(getByRole(v.root, 'radio', { name: 'Yearly' }))
    await v.flush()
    expect(getByRole(v.root, 'radio', { name: 'Yearly', checked: true })).toBeTruthy()
    expect(getByRole(v.root, 'radio', { name: 'Monthly', checked: false })).toBeTruthy()
    v.unmount()
  })
})
