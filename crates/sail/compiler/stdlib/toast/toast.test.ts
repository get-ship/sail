import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { click, getByRole, getByText, queryByRole } from '@ship/sail-test'

describe('l1 toast', () => {
  it('region and toast are locatable', () => {
    const v = renderRoute(`<toast-region>
  <toast>
    <strong>Saved.</strong> Your changes are live.
  </toast>
</toast-region>`)
    expect(getByRole(v.root, 'region')).toBeTruthy()
    expect(getByRole(v.root, 'status')).toBeTruthy()
    expect(getByText(v.root, 'Saved.')).toBeTruthy()
    v.unmount()
  })

  it('close dismisses and fires on-dismiss', async () => {
    let n = 0
    const v = renderRoute(
      `<toast duration="0" on-dismiss={gone}>
    <strong>Saved.</strong> Your changes are live.
  </toast>`,
      {
        controller: () => ({
          gone() {
            n += 1
          },
        }),
      },
    )
    await v.flush()
    expect(getByRole(v.root, 'status')).toBeTruthy()
    click(getByRole(v.root, 'button', { name: 'Dismiss' }))
    await v.flush()
    expect(queryByRole(v.root, 'status')).toBeNull()
    expect(n).toBe(1)
    v.unmount()
  })

  it('sticky duration never self-dismisses', async () => {
    const v = renderRoute(`<toast duration="0">Stays.</toast>`)
    await v.flush()
    expect(getByRole(v.root, 'status', { name: 'Stays.' })).toBeTruthy()
    await v.flush()
    expect(getByRole(v.root, 'status', { name: 'Stays.' })).toBeTruthy()
    v.unmount()
  })
})
