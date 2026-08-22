import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, click } from '@ship/sail-test'

describe('l1 context-menu', () => {
  it('right-click the for-target, then click Cut fires on-select', async () => {
    let got: { value?: string } | null = null
    const v = renderRoute(
      `<button id="canvas">Canvas</button>
<context-menu for="canvas" on-select={pick}>
  <menu-item value="cut">Cut</menu-item>
</context-menu>`,
      {
        controller: () => ({
          pick(detail: { value?: string }) {
            got = detail
          },
        }),
      },
    )
    await v.flush()
    const target = getByRole(v.root, 'button', { name: 'Canvas' })
    target.dispatchEvent(
      new MouseEvent('contextmenu', { bubbles: true, cancelable: true, clientX: 40, clientY: 50 }),
    )
    await v.flush()
    click(getByRole(v.root, 'button', { name: 'Cut' }))
    await v.flush()
    expect(got).toEqual({ value: 'cut' })
    v.unmount()
  })
})
