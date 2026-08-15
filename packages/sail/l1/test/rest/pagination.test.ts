import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<pagination value="2">
  <page value="1">1</page>
  <page value="2">2</page>
  <page value="3">3</page>
</pagination>`

describe('l1 pagination rest', () => {
  it('mounts and is locatable by current page', async () => {
    const v = renderRoute(SOURCE)
    await v.flush()
    expect(getByRole(v.root, 'button', { name: '1' })).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: '2', current: 'page' })).toBeTruthy()
    v.unmount()
  })
})
