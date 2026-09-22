import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { click, getByRole } from '@ship/sail-test'

describe('l1 switch', () => {
  it('is a named switch', async () => {
    const v = renderRoute(`<label>
  Notifications
  <switch></switch>
</label>`)
    await v.flush()
    expect(getByRole(v.root, 'switch', { name: 'Notifications', checked: false })).toBeTruthy()
    v.unmount()
  })

  it('clicking toggles checked', async () => {
    const v = renderRoute(`<label>
  Notifications
  <switch></switch>
</label>`)
    await v.flush()
    click(getByRole(v.root, 'switch', { name: 'Notifications' }))
    await v.flush()
    expect(getByRole(v.root, 'switch', { name: 'Notifications', checked: true })).toBeTruthy()
    click(getByRole(v.root, 'switch', { name: 'Notifications' }))
    await v.flush()
    expect(getByRole(v.root, 'switch', { name: 'Notifications', checked: false })).toBeTruthy()
    v.unmount()
  })

  it('disabled is disabled', async () => {
    const v = renderRoute(`<label>
  Notifications
  <switch disabled></switch>
</label>`)
    await v.flush()
    const el = getByRole(v.root, 'switch', { name: 'Notifications' }) as HTMLInputElement
    expect(el.disabled).toBe(true)
    v.unmount()
  })
})
