import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { click, getByRole } from '@ship/sail-test'

const SIZES = ['sm', 'md', 'lg'] as const
const VARIANTS = ['default', 'simple'] as const

function pages(attrs = '', pageAttrs = '') {
  return `<pagination${attrs} value="2">
  <page-item value="1"${pageAttrs}>1</page-item>
  <page-item value="2"${pageAttrs}>2</page-item>
  <page-item value="3"${pageAttrs}>3</page-item>
</pagination>`
}

describe('l1 pagination', () => {
  it('default is a named nav with the current page', async () => {
    const v = renderRoute(pages())
    await v.flush()
    expect(getByRole(v.root, 'navigation', { name: 'Pagination' })).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: '1' })).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: '2', current: 'page' })).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: '3' })).toBeTruthy()
    v.unmount()
  })

  it('clicking a page makes it current', async () => {
    const v = renderRoute(pages())
    await v.flush()
    click(getByRole(v.root, 'button', { name: '3' }))
    await v.flush()
    expect(getByRole(v.root, 'button', { name: '3', current: 'page' })).toBeTruthy()
    v.unmount()
  })

  it('each variant is still a named nav with a current page', async () => {
    for (const variant of VARIANTS) {
      const v = renderRoute(pages(` variant="${variant}"`, ` variant="${variant}"`))
      await v.flush()
      expect(getByRole(v.root, 'navigation', { name: 'Pagination' })).toBeTruthy()
      expect(getByRole(v.root, 'button', { name: '2', current: 'page' })).toBeTruthy()
      v.unmount()
    }
  })

  it('page size ladder is still a named page button', async () => {
    for (const size of SIZES) {
      const v = renderRoute(pages('', ` size="${size}"`))
      await v.flush()
      expect(getByRole(v.root, 'button', { name: '2', current: 'page' })).toBeTruthy()
      v.unmount()
    }
  })

  it('disabled page is disabled', async () => {
    const v = renderRoute(`<pagination value="1">
  <page-item value="1">1</page-item>
  <page-item value="2" disabled>2</page-item>
</pagination>`)
    await v.flush()
    const el = getByRole(v.root, 'button', { name: '2' }) as HTMLButtonElement
    expect(el.disabled).toBe(true)
    v.unmount()
  })
})
