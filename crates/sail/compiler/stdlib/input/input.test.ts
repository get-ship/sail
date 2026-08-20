import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, queryByRole, click } from '@ship/sail-test'

describe('l1 input', () => {
  it('labeled text is locatable as a textbox', async () => {
    const v = renderRoute(`<label for="em">Email</label>
<input id="em" type="email" />`)
    await v.flush()
    expect(getByRole(v.root, 'textbox', { name: 'Email' })).toBeTruthy()
    expect(queryByRole(v.root, 'button', { name: 'Show' })).toBeNull()
    v.unmount()
  })

  it('password Show click becomes Hide', async () => {
    const v = renderRoute(`<label for="pw">Password</label>
<input id="pw" type="password" />`)
    await v.flush()
    const box = getByRole(v.root, 'textbox', { name: 'Password' }) as HTMLInputElement
    expect(box.type).toBe('password')
    click(getByRole(v.root, 'button', { name: 'Show' }))
    await v.flush()
    expect(getByRole(v.root, 'button', { name: 'Hide' })).toBeTruthy()
    expect(queryByRole(v.root, 'button', { name: 'Show' })).toBeNull()
    expect((getByRole(v.root, 'textbox', { name: 'Password' }) as HTMLInputElement).type).toBe(
      'text',
    )
    click(getByRole(v.root, 'button', { name: 'Hide' }))
    await v.flush()
    expect(getByRole(v.root, 'button', { name: 'Show' })).toBeTruthy()
    expect((getByRole(v.root, 'textbox', { name: 'Password' }) as HTMLInputElement).type).toBe(
      'password',
    )
    v.unmount()
  })

  it('reveal off keeps password without a Show control', async () => {
    const v = renderRoute(`<label for="pw">Password</label>
<input id="pw" type="password" reveal={false} />`)
    await v.flush()
    expect(getByRole(v.root, 'textbox', { name: 'Password' })).toBeTruthy()
    expect(queryByRole(v.root, 'button', { name: 'Show' })).toBeNull()
    expect(queryByRole(v.root, 'button', { name: 'Hide' })).toBeNull()
    v.unmount()
  })
})
