import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText } from '@ship/sail-test'

describe('l1 prose', () => {
  it('heading and body copy are locatable', async () => {
    const v = renderRoute(`<prose>
  <h1>Release notes</h1>
  <p>Body copy.</p>
</prose>`)
    await v.flush()
    expect(getByRole(v.root, 'heading', { name: 'Release notes' })).toBeTruthy()
    expect(getByText(v.root, 'Body copy.')).toBeTruthy()
    v.unmount()
  })
})
