import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText, click } from '@ship/sail-test'

describe('l1 tabs', () => {
  it('switches panels on click', async () => {
    const v = renderRoute(`<tabs>
      <tab title="Overview">one</tab>
      <tab title="Activity">two</tab>
    </tabs>`)
    await v.flush()
    const overview = getByRole(v.root, 'tab', { name: 'Overview' })
    const activity = getByRole(v.root, 'tab', { name: 'Activity' })
    expect(overview).toBeTruthy()
    expect(activity).toBeTruthy()
    expect(getByText(v.root, 'one')).toBeTruthy()
    click(activity!)
    await v.flush()
    expect(getByText(v.root, 'two')).toBeTruthy()
    v.unmount()
  })
})
