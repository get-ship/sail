import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByText } from '@ship/sail-test'

const SOURCE = `<ticker>
  <p>tokens · signals · a11y</p>
</ticker>`

describe('l1 ticker rest', () => {
  it('mounts and is locatable by track text', () => {
    const v = renderRoute(SOURCE)
    expect(getByText(v.root, 'tokens · signals · a11y')).toBeTruthy()
    v.unmount()
  })
})
