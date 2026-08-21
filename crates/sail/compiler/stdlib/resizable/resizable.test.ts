import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

describe('l1 resizable', () => {
  it('handle is a separator with Resize name', async () => {
    const v = renderRoute(`<resizable><div slot="start">left</div><div slot="end">right</div></resizable>`)
    await v.flush()
    expect(getByRole(v.root, 'separator', { name: 'Resize' })).toBeTruthy()
    v.unmount()
  })

  it('arrow keys nudge valueno', async () => {
    const v = renderRoute(`<resizable><div slot="start">left</div><div slot="end">right</div></resizable>`)
    await v.flush()
    const handle = getByRole(v.root, 'separator', { name: 'Resize' }) as HTMLElement
    expect(handle).toBeTruthy()
    const before = handle.getAttribute('aria-valuenow')
    handle.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true }))
    await v.flush()
    const after = handle.getAttribute('aria-valuenow')
    expect(before).not.toBe(after)
    v.unmount()
  })

  it('vertical orientation stamps aria-orientation', async () => {
    const v = renderRoute(`<resizable orientation="vertical"><div slot="start">top</div><div slot="end">bottom</div></resizable>`)
    await v.flush()
    const handle = getByRole(v.root, 'separator', { name: 'Resize' }) as HTMLElement
    expect(handle.getAttribute('aria-orientation')).toBe('vertical')
    v.unmount()
  })
})
