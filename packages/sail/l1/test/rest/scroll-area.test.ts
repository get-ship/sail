import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByText } from '@ship/sail-test'

const SOURCE = `<scroll-area>
  <p>Overflow content stays scrollable.</p>
</scroll-area>`

describe('l1 scroll-area rest', () => {
  it('mounts and is locatable by content', () => {
    const v = renderRoute(SOURCE)
    expect(getByText(v.root, 'Overflow content stays scrollable.')).toBeTruthy()
    v.unmount()
  })
})
