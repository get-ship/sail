// Scaffold-style behavioral test: compile+mount via @ship/test-setup adapter,
// assert only with @ship/sail-test semantic queries (no structure / data-*).

import { describe, expect, it } from 'vitest'
import { signal } from '@ship/sail-runtime'
import { renderRoute } from '@ship/test-setup'
import { getByRole, getByText, click } from '../src/index'

describe('renderRoute + semantic queries (scaffold acceptance)', () => {
  it('locates and drives UI by role and name only', async () => {
    const v = renderRoute(
      `<column>
        <h1>{title()}</h1>
        <button type="button" label="Increment" on:click={() => count.set(count() + 1)}>
          count {count()}
        </button>
        <if {count() > 0}>
          <div role="status" live="polite">Updated</div>
        </if>
      </column>`,
      {
        controller: () => {
          const count = signal(0)
          const title = signal('Counter')
          return { count, title }
        },
      },
    )

    expect(getByRole(v.root, 'heading', { name: 'Counter', level: 1 })).toBeTruthy()
    const btn = getByRole(v.root, 'button', { name: /Increment|count/ })
    click(btn)
    await v.flush()
    expect(getByText(v.root, 'count 1')).toBeTruthy()
    expect(getByRole(v.root, 'status', { name: /Updated/ })).toBeTruthy()

    v.unmount()
  })
})
