import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText } from '@ship/sail-test'

const SOURCE = `<resizable>
  <p slot="start">Start pane</p>
  <p slot="end">End pane</p>
</resizable>`

describe('l1 resizable rest', () => {
  it('mounts and is locatable by panes and resize handle', async () => {
    const v = renderRoute(SOURCE)
    await v.flush()
    expect(getByText(v.root, 'Start pane')).toBeTruthy()
    expect(getByText(v.root, 'End pane')).toBeTruthy()
    expect(getByRole(v.root, 'separator', { name: 'Resize' })).toBeTruthy()
    v.unmount()
  })
})
