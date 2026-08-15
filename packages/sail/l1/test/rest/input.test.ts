import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<input placeholder="you@ship.dev" />`

describe('l1 input rest', () => {
  it('mounts and is locatable by placeholder name', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'textbox', { name: 'you@ship.dev' })).toBeTruthy()
    v.unmount()
  })
})
