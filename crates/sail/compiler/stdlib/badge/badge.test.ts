import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByText } from '@ship/sail-test'

const SIZES = ['sm', 'md', 'lg'] as const
const VARIANTS = ['soft', 'solid', 'outline'] as const

describe('l1 badge', () => {
  it('text is locatable', () => {
    const v = renderRoute(`<badge>Active</badge>`)
    expect(getByText(v.root, 'Active')).toBeTruthy()
    v.unmount()
  })

  it('dot does not destroy the name', () => {
    const v = renderRoute(`<badge dot>Live</badge>`)
    expect(getByText(v.root, 'Live')).toBeTruthy()
    v.unmount()
  })

  it('each size still exposes the text', () => {
    for (const size of SIZES) {
      const v = renderRoute(`<badge size="${size}">Live</badge>`)
      expect(getByText(v.root, 'Live')).toBeTruthy()
      v.unmount()
    }
  })

  it('each variant still exposes the text', () => {
    for (const variant of VARIANTS) {
      const v = renderRoute(`<badge variant="${variant}">Live</badge>`)
      expect(getByText(v.root, 'Live')).toBeTruthy()
      v.unmount()
    }
  })
})
