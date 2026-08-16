import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

function field(attrs = '') {
  return `<field${attrs}>
  <label slot="label" for="email">Email</label>
  <input id="email" type="email" />
  <span slot="hint">We will never share this.</span>
  <span slot="error">Enter a valid email.</span>
</field>`
}

describe('l1 field', () => {
  it('fill field describes the control with the hint', async () => {
    const v = renderRoute(field())
    await v.flush()
    const input = getByRole(v.root, 'textbox', { name: 'Email' })
    const id = input.getAttribute('aria-describedby')
    expect(id).toBeTruthy()
    expect(v.root.querySelector(`#${CSS.escape(id!)}`)?.textContent).toContain(
      'We will never share this.',
    )
    expect(input.getAttribute('aria-invalid')).toBe('false')
    v.unmount()
  })

  it('invalid fill field describes the control with the error', async () => {
    const v = renderRoute(field(' invalid'))
    await v.flush()
    const input = getByRole(v.root, 'textbox', { name: 'Email' })
    const id = input.getAttribute('aria-describedby')
    expect(id).toBeTruthy()
    expect(v.root.querySelector(`#${CSS.escape(id!)}`)?.textContent).toContain(
      'Enter a valid email.',
    )
    expect(input.getAttribute('aria-invalid')).toBe('true')
    v.unmount()
  })
})
