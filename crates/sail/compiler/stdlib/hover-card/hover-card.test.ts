import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

describe('l1 hover-card', () => {
  it('panel is a dialog (HOST_ANATOMY)', async () => {
    const v = renderRoute(`<a href="#" popovertarget="hc1">trigger</a><hover-card id="hc1">preview</hover-card>`)
    await v.flush()
    // Harness contract (test/setup.ts): closed popovers sit outside the a11y
    // tree. Open the panel, then assert its dialog role.
    ;(v.root.querySelector('div[popover]') as HTMLElement | null)?.showPopover()
    await v.flush()
    expect(getByRole(v.root, 'dialog')).toBeTruthy()
    v.unmount()
  })

  it('Escape hides the popover via hotkeys', async () => {
    const v = renderRoute(`<a href="#" popovertarget="hc2">trigger</a><hover-card id="hc2">preview</hover-card>`)
    await v.flush()
    ;(v.root.querySelector('div[popover]') as HTMLElement | null)?.showPopover()
    await v.flush()
    const panel = getByRole(v.root, 'dialog') as HTMLElement
    expect(panel).toBeTruthy()
    // show via API then Escape must HIDE the panel (controller keys)
    panel.showPopover()
    await v.flush()
    panel.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    await v.flush()
    expect(panel.hidden).toBe(true)
    v.unmount()
  })
})
