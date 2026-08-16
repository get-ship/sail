import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

describe('l1 form', () => {
  it('is a named form that holds fields', () => {
    const v = renderRoute(`<form label="Contact">
  <label for="n">Name</label>
  <input id="n">
  <button type="submit">Send</button>
</form>`)
    expect(getByRole(v.root, 'form', { name: 'Contact' })).toBeTruthy()
    expect(getByRole(v.root, 'textbox', { name: 'Name' })).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Send' })).toBeTruthy()
    v.unmount()
  })
})
