import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

function contact(attrs = '') {
  return `<form label="Contact"${attrs}>
  <label for="n">Name</label>
  <input id="n">
  <button type="submit">Send</button>
</form>`
}

describe('l1 form', () => {
  it('default is a named form that holds fields', () => {
    const v = renderRoute(contact())
    expect(getByRole(v.root, 'form', { name: 'Contact' })).toBeTruthy()
    expect(getByRole(v.root, 'textbox', { name: 'Name' })).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Send' })).toBeTruthy()
    v.unmount()
  })

  it('layout=fill is a named form that holds fields', () => {
    const v = renderRoute(contact(' layout="fill"'))
    expect(getByRole(v.root, 'form', { name: 'Contact' })).toBeTruthy()
    expect(getByRole(v.root, 'textbox', { name: 'Name' })).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Send' })).toBeTruthy()
    v.unmount()
  })
})
