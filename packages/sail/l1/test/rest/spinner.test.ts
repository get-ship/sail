import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByText } from '@ship/sail-test'

const SOURCE = `<p>fetching… <spinner></spinner></p>`

describe('l1 spinner rest', () => {
  it('mounts next to named wait copy', () => {
    const v = renderRoute(SOURCE)
    expect(getByText(v.root, 'fetching…')).toBeTruthy()
    v.unmount()
  })
})
