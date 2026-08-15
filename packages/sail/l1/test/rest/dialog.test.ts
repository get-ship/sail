import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<button command="show-modal" commandfor="confirm">Delete project</button>
<dialog id="confirm">
  <header>Delete project</header>
  <section>This cannot be undone.</section>
  <footer>
    <button command="close" commandfor="confirm">Cancel</button>
  </footer>
</dialog>`

describe('l1 dialog rest', () => {
  it('mounts and locates the trigger at rest', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'button', { name: 'Delete project' })).toBeTruthy()
    v.unmount()
  })
})
