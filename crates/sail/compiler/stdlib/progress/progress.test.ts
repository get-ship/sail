import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

describe('l1 progress', () => {
  it('is a progressbar and exposes the authored range when the tree has it', () => {
    const v = renderRoute(`<progress value="64" max="100"></progress>`)
    const el = getByRole(v.root, 'progressbar')
    const min = el.getAttribute('aria-valuemin') ?? el.getAttribute('min')
    const now = el.getAttribute('aria-valuenow') ?? el.getAttribute('value')
    const max = el.getAttribute('aria-valuemax') ?? el.getAttribute('max')
    if (min !== null) expect(min).toBe('0')
    if (now !== null) expect(now).toBe('64')
    if (max !== null) expect(max).toBe('100')
    v.unmount()
  })

  it('indeterminate is still a progressbar', () => {
    const v = renderRoute(`<progress></progress>`)
    const el = getByRole(v.root, 'progressbar')
    expect(el).toBeTruthy()
    const now = el.getAttribute('aria-valuenow') ?? el.getAttribute('value')
    if (now !== null) expect(now).toBe('')
    v.unmount()
  })
})
