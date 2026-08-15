import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<alert>Saved just now.</alert>`

describe('l1 alert rest', () => {
  it('mounts and is locatable by status name', async () => {
    const v = renderRoute(SOURCE)
    await v.flush()
    expect(getByRole(v.root, 'status', { name: 'Saved just now.' })).toBeTruthy()
    v.unmount()
  })
})
