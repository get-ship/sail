import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, click } from '@ship/sail-test'

describe('l1 menu-item', () => {
  it('is locatable by name', async () => {
    const v = renderRoute(`<menu-item>Edit</menu-item>`)
    await v.flush()
    expect(getByRole(v.root, 'button', { name: 'Edit' })).toBeTruthy()
    v.unmount()
  })

  it('click the button named Edit fires on-select with value edit', async () => {
    let got: { value?: string } | undefined
    const v = renderRoute(`<menu-item value="edit" on-select={pick}>Edit</menu-item>`, {
      controller: () => ({
        pick(detail: { value: string }) {
          got = detail
        },
      }),
    })
    await v.flush()
    click(getByRole(v.root, 'button', { name: 'Edit' }))
    await v.flush()
    expect(got).toEqual({ value: 'edit' })
    v.unmount()
  })
})
