import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByText } from '@ship/sail-test'

const SOURCE = `<p>Press <kbd>Ctrl</kbd> + <kbd>K</kbd> to open the command palette.</p>`

describe('l1 kbd rest', () => {
  it('mounts and is locatable by keycap text', () => {
    const v = renderRoute(SOURCE)
    expect(getByText(v.root, 'Ctrl')).toBeTruthy()
    expect(getByText(v.root, 'K')).toBeTruthy()
    v.unmount()
  })
})
