import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const KINDS = ['date', 'time', 'datetime-local', 'month', 'week'] as const

describe('l1 date', () => {
  it('labeled date is locatable as a textbox', async () => {
    const v = renderRoute(`<label>
  Birthday
  <date />
</label>`)
    await v.flush()
    expect(getByRole(v.root, 'textbox', { name: 'Birthday' })).toBeTruthy()
    v.unmount()
  })

  it('each type is still a labeled textbox', async () => {
    for (const type of KINDS) {
      const v = renderRoute(`<label>
  Birthday
  <date type="${type}" />
</label>`)
      await v.flush()
      expect(getByRole(v.root, 'textbox', { name: 'Birthday' })).toBeTruthy()
      v.unmount()
    }
  })

  it('disabled is disabled', async () => {
    const v = renderRoute(`<label>
  Birthday
  <date disabled />
</label>`)
    await v.flush()
    const el = getByRole(v.root, 'textbox', { name: 'Birthday' }) as HTMLInputElement
    expect(el.disabled).toBe(true)
    v.unmount()
  })
})
