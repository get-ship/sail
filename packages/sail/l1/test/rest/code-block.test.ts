import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByText } from '@ship/sail-test'

const SOURCE = `<code-block code="const n = signal(0)" lang="ts"></code-block>`

describe('l1 code-block rest', () => {
  it('mounts and is locatable by source text', async () => {
    const v = renderRoute(SOURCE)
    await v.flush()
    expect(getByText(v.root, /signal/)).toBeTruthy()
    v.unmount()
  })
})
