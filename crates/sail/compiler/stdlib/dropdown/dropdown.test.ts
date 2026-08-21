import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, click } from '@ship/sail-test'

describe('l1 dropdown', () => {
  it('activating a slotted menu-item fires on-select', async () => {
    let got: { value?: string } | null = null
    const v = renderRoute(
      `<button id="dd-trigger" popovertarget="dd1">Actions</button>
<dropdown id="dd1" on-select={pick}>
  <menu-item value="rename">Rename</menu-item>
  <menu-item value="archive">Archive</menu-item>
</dropdown>`,
      {
        controller: () => ({
          pick(detail: { value?: string }) {
            got = detail
          },
        }),
      },
    )
    // Harness contract (test/setup.ts): closed popovers sit outside the a11y
    // tree; open the panel before asserting its items. Native popovertarget
    // activation is browser-only.
    ;(v.root.querySelector('div[popover]') as HTMLElement | null)?.showPopover()
    await v.flush()
    click(getByRole(v.root, 'button', { name: 'Rename' }))
    await v.flush()
    expect(got).toEqual({ value: 'rename' })
    v.unmount()
  })
})
