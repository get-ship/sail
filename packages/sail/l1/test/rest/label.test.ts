import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText } from '@ship/sail-test'

const SOURCE = `<label for="email-field">Email</label>
<input id="email-field" placeholder="ada@lovelace.dev" />`

describe('l1 label rest', () => {
  it('mounts and names the associated control', () => {
    const v = renderRoute(SOURCE)
    expect(getByText(v.root, 'Email')).toBeTruthy()
    expect(getByRole(v.root, 'textbox', { name: 'Email' })).toBeTruthy()
    v.unmount()
  })
})
