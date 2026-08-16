import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByText } from '@ship/sail-test'

const SOURCE = `<error-boundary message="This panel could not load.">
  <p>A risky subtree.</p>
</error-boundary>`

describe('l1 error-boundary rest', () => {
  it('mounts and is locatable by content at rest', () => {
    const v = renderRoute(SOURCE)
    expect(getByText(v.root, 'A risky subtree.')).toBeTruthy()
    v.unmount()
  })
})
