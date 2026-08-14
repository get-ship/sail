import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText } from '@ship/sail-test'

const SOURCE = `<fieldset>
  <legend>Account</legend>
  <button>Save account</button>
</fieldset>`

describe('l1 fieldset rest', () => {
  it('mounts with a named legend and a locatable button', () => {
    const v = renderRoute(SOURCE)
    expect(getByText(v.root, 'Account')).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Save account' })).toBeTruthy()
    v.unmount()
  })
})
