import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<theme-picker></theme-picker>`

describe('l1 theme-picker rest', () => {
  it('mounts and is locatable by trigger name', async () => {
    const v = renderRoute(SOURCE)
    await v.flush()
    expect(getByRole(v.root, 'button', { name: /Theme:/ })).toBeTruthy()
    v.unmount()
  })
})
