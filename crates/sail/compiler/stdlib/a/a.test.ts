import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const VARIANTS = ['text', 'default', 'solid', 'soft', 'outline', 'ghost'] as const
const TONES = ['primary', 'info', 'success', 'warning', 'danger', 'accent', 'muted'] as const

describe('l1 a', () => {
  it('bare href is locatable as a link by name', () => {
    const v = renderRoute(`<a href="/docs">Docs</a>`)
    expect(getByRole(v.root, 'link', { name: 'Docs' })).toBeTruthy()
    v.unmount()
  })

  it('each variant is still a link with the same name', () => {
    for (const variant of VARIANTS) {
      const v = renderRoute(`<a href="/docs" variant="${variant}">Docs</a>`)
      expect(getByRole(v.root, 'link', { name: 'Docs' })).toBeTruthy()
      v.unmount()
    }
  })

  it('each tone is still a link with the same name', () => {
    for (const tone of TONES) {
      const v = renderRoute(`<a href="/docs" tone="${tone}">Docs</a>`)
      expect(getByRole(v.root, 'link', { name: 'Docs' })).toBeTruthy()
      v.unmount()
    }
  })
})
