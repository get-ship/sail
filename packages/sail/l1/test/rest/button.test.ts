import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<button>Save draft</button>`

describe('l1 button rest', () => {
  it('mounts and is locatable by button name', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'button', { name: 'Save draft' })).toBeTruthy()
    v.unmount()
  })
})
