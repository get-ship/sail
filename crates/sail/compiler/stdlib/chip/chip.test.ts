import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

describe('l1 chip', () => {
  it('is locatable by name', async () => {
    const v = renderRoute(`<chip>Design</chip>`)
    await v.flush()
    expect(getByRole(v.root, 'button', { name: 'Design' })).toBeTruthy()
    v.unmount()
  })

  it('removable exposes a remove control', async () => {
    const v = renderRoute(`<chip removable>Design</chip>`)
    await v.flush()
    expect(getByRole(v.root, 'button', { name: 'Design' })).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Remove' })).toBeTruthy()
    v.unmount()
  })
})
