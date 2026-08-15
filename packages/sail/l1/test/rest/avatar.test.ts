import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByText } from '@ship/sail-test'

const SOURCE = `<avatar initials="AL" name="Ada Lovelace"></avatar>`

describe('l1 avatar rest', () => {
  it('mounts and is locatable by initials', async () => {
    const v = renderRoute(SOURCE)
    await v.flush()
    expect(getByText(v.root, 'AL')).toBeTruthy()
    v.unmount()
  })
})
