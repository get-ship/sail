import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getAllByRole } from '@ship/sail-test'

describe('rest: slider', () => {
  it('single thumb is one slider', async () => {
    const v = renderRoute(`<slider value="40" />`)
    await v.flush()
    expect(getAllByRole(v.root, 'slider')).toHaveLength(1)
    v.unmount()
  })

  it('range exposes two sliders', async () => {
    const v = renderRoute(`<slider range value="20" value-end="80" />`)
    await v.flush()
    expect(getAllByRole(v.root, 'slider')).toHaveLength(2)
    v.unmount()
  })

  it('size lands on the sail-slider host so platform thumb vars apply', async () => {
    const v = renderRoute(`<slider value="50" size="sm" />`)
    await v.flush()
    const host = v.root.querySelector('.sail-slider')
    expect(host).not.toBeNull()
    expect(host!.getAttribute('size')).toBe('sm')
    v.unmount()
  })
})
