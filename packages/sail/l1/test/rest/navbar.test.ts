import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText } from '@ship/sail-test'

const SOURCE = `<navbar bordered>
  <strong slot="brand">SHIP</strong>
  <a href="/docs">Docs</a>
  <button slot="actions" size="sm" variant="outline">Sign in</button>
</navbar>`

describe('l1 navbar rest', () => {
  it('mounts and is locatable by brand, link, and action', () => {
    const v = renderRoute(SOURCE)
    expect(getByText(v.root, 'SHIP')).toBeTruthy()
    expect(getByRole(v.root, 'link', { name: 'Docs' })).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Sign in' })).toBeTruthy()
    v.unmount()
  })
})
