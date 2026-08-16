import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText } from '@ship/sail-test'

const SOURCE = `<error-state code="ERR_TIMEOUT">
  <strong slot="title">Couldn't load your data</strong>
  The request timed out.
  <button slot="action">Retry</button>
</error-state>`

describe('l1 error-state rest', () => {
  it('mounts and is locatable by alert title', async () => {
    const v = renderRoute(SOURCE)
    await v.flush()
    expect(getByRole(v.root, 'alert')).toBeTruthy()
    expect(getByText(v.root, "Couldn't load your data")).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Retry' })).toBeTruthy()
    v.unmount()
  })
})
