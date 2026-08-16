import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText, queryByRole } from '@ship/sail-test'

describe('l1 separator', () => {
  it('a labeled host is locatable by its name', () => {
    const v = renderRoute(`<separator label="Or"></separator>`)
    expect(getByRole(v.root, 'separator', { name: 'Or' })).toBeTruthy()
    v.unmount()
  })

  it('a presentation separator does not require a heading', () => {
    const v = renderRoute(`<p>above</p>
<separator></separator>
<p>below</p>`)
    expect(getByText(v.root, 'above')).toBeTruthy()
    expect(getByText(v.root, 'below')).toBeTruthy()
    expect(queryByRole(v.root, 'heading')).toBeNull()
    v.unmount()
  })
})
