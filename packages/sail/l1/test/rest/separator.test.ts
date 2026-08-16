import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByText } from '@ship/sail-test'

const SOURCE = `<p>above</p>
<separator></separator>
<p>below</p>`

describe('l1 separator rest', () => {
  it('mounts between named content', () => {
    const v = renderRoute(SOURCE)
    expect(getByText(v.root, 'above')).toBeTruthy()
    expect(getByText(v.root, 'below')).toBeTruthy()
    v.unmount()
  })
})
