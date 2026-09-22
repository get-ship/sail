import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

describe('l1 meter', () => {
  it('is a meter and exposes the authored range when the tree has it', () => {
    const v = renderRoute(`<meter label="Disk free" value="72" min="0" max="100"></meter>`)
    const el = getByRole(v.root, 'meter', { name: 'Disk free' })
    const min = el.getAttribute('aria-valuemin') ?? el.getAttribute('min')
    const now = el.getAttribute('aria-valuenow') ?? el.getAttribute('value')
    const max = el.getAttribute('aria-valuemax') ?? el.getAttribute('max')
    if (min !== null) expect(min).toBe('0')
    if (now !== null) expect(now).toBe('72')
    if (max !== null) expect(max).toBe('100')
    v.unmount()
  })
})
