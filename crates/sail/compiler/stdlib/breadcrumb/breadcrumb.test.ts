import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText } from '@ship/sail-test'

const SIZES = ['sm', 'md', 'lg'] as const

function trail(attrs = '') {
  return `<breadcrumb${attrs}>
  <a href="/components">components</a>
  <a href="/docs">docs</a>
  <small>breadcrumb</small>
</breadcrumb>`
}

describe('l1 breadcrumb', () => {
  it('named Breadcrumb navigation exposes earlier crumbs as links and current page text', async () => {
    const v = renderRoute(trail())
    await v.flush()
    expect(getByRole(v.root, 'navigation', { name: 'Breadcrumb' })).toBeTruthy()
    expect(getByRole(v.root, 'link', { name: 'components' })).toBeTruthy()
    expect(getByRole(v.root, 'link', { name: 'docs' })).toBeTruthy()
    expect(getByText(v.root, 'breadcrumb')).toBeTruthy()
    v.unmount()
  })

  it('each size still exposes earlier crumbs as links and current page text', async () => {
    for (const size of SIZES) {
      const v = renderRoute(trail(` size="${size}"`))
      await v.flush()
      expect(getByRole(v.root, 'navigation', { name: 'Breadcrumb' })).toBeTruthy()
      expect(getByRole(v.root, 'link', { name: 'components' })).toBeTruthy()
      expect(getByText(v.root, 'breadcrumb')).toBeTruthy()
      v.unmount()
    }
  })
})
