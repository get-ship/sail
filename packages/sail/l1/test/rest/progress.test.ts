import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<progress value="64" max="100"></progress>`

describe('l1 progress rest', () => {
  it('mounts and is locatable as a progressbar', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'progressbar')).toBeTruthy()
    v.unmount()
  })
})
