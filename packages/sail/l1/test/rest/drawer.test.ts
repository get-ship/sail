import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<button command="show" commandfor="filters">Filters</button>
<drawer id="filters" side="end">
  <header>Filters</header>
  <section>Narrow the catalog.</section>
  <footer>
    <button command="close" commandfor="filters">Cancel</button>
  </footer>
</drawer>`

describe('l1 drawer rest', () => {
  it('mounts and locates the trigger at rest', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'button', { name: 'Filters' })).toBeTruthy()
    v.unmount()
  })
})
