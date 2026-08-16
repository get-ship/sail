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
  it('fill default — labeled control is locatable', async () => {
    const v = renderRoute(`<field>
  <label slot="label" for="display-name">Name</label>
  <input id="display-name" />
</field>`)
    await v.flush()
    expect(getByRole(v.root, 'textbox', { name: 'Name' })).toBeTruthy()
    v.unmount()
  })

  it('invalid surfaces the error on the labeled control', async () => {
    const v = renderRoute(`<field invalid>
  <label slot="label" for="display-name">Name</label>
  <input id="display-name" />
  <span slot="error">Enter a name</span>
</field>`)
    await v.flush()
    const box = getByRole(v.root, 'textbox', { name: 'Name' })
    expect(box.getAttribute('aria-invalid')).toBe('true')
    expect(getByRole(v.root, 'alert', { name: 'Enter a name' })).toBeTruthy()
    v.unmount()
  })

  it('valid field keeps the control valid', async () => {
    const v = renderRoute(`<field>
  <label slot="label" for="display-name">Name</label>
  <input id="display-name" />
  <span slot="error">Enter a name</span>
</field>`)
    await v.flush()
    const box = getByRole(v.root, 'textbox', { name: 'Name' })
    expect(box.getAttribute('aria-invalid')).toBe('false')
    v.unmount()
  })

  it('fill=false still names the compact control', async () => {
    const v = renderRoute(`<row>
  <field fill={false}>
    <label slot="label" for="q">Search questions</label>
    <input id="q" />
  </field>
  <button>Search</button>
</row>`)
    await v.flush()
    expect(getByRole(v.root, 'textbox', { name: 'Search questions' })).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Search' })).toBeTruthy()
    v.unmount()
  })

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
