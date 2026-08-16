import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<chip>Design</chip>`

describe('l1 chip rest', () => {
  it('mounts and is locatable by button name', async () => {
    const v = renderRoute(SOURCE)
    await v.flush()
    expect(getByRole(v.root, 'button', { name: 'Design' })).toBeTruthy()
    v.unmount()
  })
})
