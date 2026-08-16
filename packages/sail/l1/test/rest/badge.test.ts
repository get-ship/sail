import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByText } from '@ship/sail-test'

const SOURCE = `<badge>Active</badge>`

describe('l1 badge rest', () => {
  it('mounts and is locatable by text', () => {
    const v = renderRoute(SOURCE)
    expect(getByText(v.root, 'Active')).toBeTruthy()
    v.unmount()
  })
})
