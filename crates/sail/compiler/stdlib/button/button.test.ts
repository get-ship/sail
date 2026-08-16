import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const VARIANTS = ['default', 'solid', 'soft', 'outline', 'ghost'] as const
const TONES = ['primary', 'info', 'success', 'warning', 'destructive', 'accent', 'muted'] as const

describe('l1 button', () => {
  it('is locatable by button name', () => {
    const v = renderRoute(`<button>Save draft</button>`)
    expect(getByRole(v.root, 'button', { name: 'Save draft' })).toBeTruthy()
    v.unmount()
  })

  it('disabled is disabled', () => {
    const v = renderRoute(`<button disabled>Save draft</button>`)
    const el = getByRole(v.root, 'button', { name: 'Save draft' }) as HTMLButtonElement
    expect(el.disabled).toBe(true)
    v.unmount()
  })

  it('each variant is still a button with the same name', () => {
    for (const variant of VARIANTS) {
      const v = renderRoute(`<button variant="${variant}">Save draft</button>`)
      expect(getByRole(v.root, 'button', { name: 'Save draft' })).toBeTruthy()
      v.unmount()
    }
  })

  it('each tone is still a button with the same name', () => {
    for (const tone of TONES) {
      const v = renderRoute(`<button tone="${tone}">Save draft</button>`)
      expect(getByRole(v.root, 'button', { name: 'Save draft' })).toBeTruthy()
      v.unmount()
    }
  })
})
