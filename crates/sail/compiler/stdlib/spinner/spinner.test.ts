import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const VARIANTS = ['ring', 'dots', 'bars'] as const
const SIZES = ['xs', 'sm', 'md', 'lg'] as const

describe('l1 spinner', () => {
  it('author-void spinner is one wait status named Loading', () => {
    const v = renderRoute(`<spinner></spinner>`)
    expect(getByRole(v.root, 'status', { name: 'Loading' })).toBeTruthy()
    v.unmount()
  })

  it('variant ring dots bars is one wait status named Loading', () => {
    for (const variant of VARIANTS) {
      const v = renderRoute(`<spinner variant="${variant}"></spinner>`)
      expect(getByRole(v.root, 'status', { name: 'Loading' })).toBeTruthy()
      v.unmount()
    }
  })

  it('size ladder is one wait status named Loading', () => {
    for (const size of SIZES) {
      const v = renderRoute(`<spinner size="${size}"></spinner>`)
      expect(getByRole(v.root, 'status', { name: 'Loading' })).toBeTruthy()
      v.unmount()
    }
  })
})
