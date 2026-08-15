import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<a href="/docs">Docs</a>`

describe('l1 a rest', () => {
  it('mounts and is locatable by link name', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'link', { name: 'Docs' })).toBeTruthy()
    v.unmount()
  })
})
