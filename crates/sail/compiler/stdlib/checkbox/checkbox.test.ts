import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { click, getByRole } from '@ship/sail-test'

describe('l1 checkbox', () => {
  it('is a named checkbox', () => {
    const v = renderRoute(`<label>
  <checkbox />
  I agree
</label>`)
    expect(getByRole(v.root, 'checkbox', { name: 'I agree', checked: false })).toBeTruthy()
    v.unmount()
  })

  it('clicking toggles checked', async () => {
    const v = renderRoute(`<label>
  <checkbox />
  I agree
</label>`)
    click(getByRole(v.root, 'checkbox', { name: 'I agree' }))
    await v.flush()
    expect(getByRole(v.root, 'checkbox', { name: 'I agree', checked: true })).toBeTruthy()
    click(getByRole(v.root, 'checkbox', { name: 'I agree' }))
    await v.flush()
    expect(getByRole(v.root, 'checkbox', { name: 'I agree', checked: false })).toBeTruthy()
    v.unmount()
  })

  it('disabled is disabled', () => {
    const v = renderRoute(`<label>
  <checkbox disabled />
  I agree
</label>`)
    const el = getByRole(v.root, 'checkbox', { name: 'I agree' }) as HTMLInputElement
    expect(el.disabled).toBe(true)
    v.unmount()
  })
})
