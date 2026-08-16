import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByText } from '@ship/sail-test'

describe('l1 kbd', () => {
  it('the key text is locatable', () => {
    const v = renderRoute(`<p>Press <kbd>Ctrl</kbd> + <kbd>K</kbd> to open the command palette.</p>`)
    expect(getByText(v.root, 'Ctrl')).toBeTruthy()
    expect(getByText(v.root, 'K')).toBeTruthy()
    v.unmount()
  })
})
