import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByText } from '@ship/sail-test'

const SOURCE = `<accordion>
  <strong slot="trigger">Advanced options</strong>
  <p>Hidden until asked for.</p>
</accordion>`

describe('l1 accordion rest', () => {
  it('mounts and is locatable by trigger text', () => {
    const v = renderRoute(SOURCE)
    expect(getByText(v.root, 'Advanced options')).toBeTruthy()
    v.unmount()
  })
})
