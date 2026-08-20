import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, click } from '@ship/sail-test'

describe('l1 chip', () => {
  it('is locatable by name', async () => {
    const v = renderRoute(`<chip>Design</chip>`)
    await v.flush()
    expect(getByRole(v.root, 'button', { name: 'Design' })).toBeTruthy()
    v.unmount()
  })

  it('removable exposes a remove control', async () => {
    const v = renderRoute(`<chip removable>Design</chip>`)
    await v.flush()
    expect(getByRole(v.root, 'button', { name: 'Design' })).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Remove' })).toBeTruthy()
    v.unmount()
  })

  it('selected is pressed', async () => {
    const v = renderRoute(`<chip selected>Design</chip>`)
    await v.flush()
    expect(getByRole(v.root, 'button', { name: 'Design', pressed: true })).toBeTruthy()
    v.unmount()
  })

  it('remove click fires on-remove', async () => {
    let n = 0
    const v = renderRoute(`<chip removable on-remove={drop}>Design</chip>`, {
      controller: () => ({
        drop() {
          n += 1
        },
      }),
    })
    await v.flush()
    click(getByRole(v.root, 'button', { name: 'Remove' }))
    await v.flush()
    expect(n).toBe(1)
    v.unmount()
  })
})
