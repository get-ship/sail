import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, click } from '@ship/sail-test'

const SOURCE = `<combobox placeholder="Pick a fruit" on-select={onSelect}>
  <option value="apple">Apple</option>
  <option value="banana">Banana</option>
</combobox>`

describe('l1 combobox', () => {
  it('clicking option Apple fires on-select with value apple', async () => {
    let got: { value?: string; label?: string } | null = null
    const v = renderRoute(SOURCE, {
      controller: () => ({
        onSelect(detail: { value: string; label: string }) {
          got = detail
        },
      }),
    })
    await v.flush()
    click(getByRole(v.root, 'textbox', { name: 'Pick a fruit' }))
    await v.flush()
    click(getByRole(v.root, 'option', { name: 'Apple' }))
    await v.flush()
    expect(got).toEqual({ value: 'apple', label: 'Apple' })
    v.unmount()
  })
})
