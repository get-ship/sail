import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByText } from '@ship/sail-test'

const SOURCE = `<file />`

describe('l1 file rest', () => {
  it('mounts and is locatable by browse hint', () => {
    const v = renderRoute(SOURCE)
    expect(getByText(v.root, 'Drop a file or click to browse')).toBeTruthy()
    v.unmount()
  })
})
