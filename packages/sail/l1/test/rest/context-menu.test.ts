import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<button id="canvas">right-click target</button>
<context-menu for="canvas">
  <menu-item value="cut">Cut</menu-item>
  <menu-item value="copy">Copy</menu-item>
</context-menu>`

describe('l1 context-menu rest', () => {
  it('mounts and locates the named target at rest', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'button', { name: 'right-click target' })).toBeTruthy()
    v.unmount()
  })
})
