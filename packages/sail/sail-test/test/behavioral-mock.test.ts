// Acceptance-shaped example: behavioral UI driven only with semantic
// queries + mockQueryState — no structure selectors, no backend.

import { describe, expect, it, afterEach } from 'vitest'
import { queryKeyed, effect, type LiveState } from '@ship/sail-runtime'
// Owner-tree scopes are emit/host ABI; tests use the internal helpers.
import { createScope, withScope } from '@ship/sail-runtime/internal'
import {
  getByRole,
  getByText,
  mockQueryState,
  clearQueryMocks,
} from '../src/index'

const tick = () => new Promise((r) => setTimeout(r, 0))

afterEach(() => {
  document.body.innerHTML = ''
})

type User = { name: string }

function paint(root: HTMLElement, state: LiveState<User>) {
  root.replaceChildren()
  if (state.kind === 'loading') {
    const status = document.createElement('div')
    status.setAttribute('role', 'status')
    status.setAttribute('aria-label', 'Loading')
    status.textContent = 'Loading…'
    root.appendChild(status)
    return
  }
  if (state.kind === 'error') {
    const alert = document.createElement('div')
    alert.setAttribute('role', 'alert')
    alert.textContent = state.error.message
    root.appendChild(alert)
    return
  }
  const heading = document.createElement('h1')
  heading.textContent = state.value.name
  const btn = document.createElement('button')
  btn.type = 'button'
  btn.textContent = 'Refresh'
  root.appendChild(heading)
  root.appendChild(btn)
}

describe('behavioral: semantic queries + mock seam', () => {
  it('forces loading → error → ok without backend; asserts via role/name/text', async () => {
    const scope = createScope()
    const root = document.createElement('div')
    document.body.appendChild(root)
    const key = ['users', 'profile'] as const
    let backendHits = 0

    const users = withScope(scope, () => {
      mockQueryState(key, { kind: 'loading' })
      return queryKeyed(
        () => key,
        () => {
          backendHits++
          return Promise.resolve({ name: 'from-network' })
        },
      )
    })

    const stop = withScope(scope, () =>
      effect(() => {
        paint(root, users())
      }),
    )

    await tick()
    expect(getByRole(root, 'status', { name: 'Loading' })).toBeTruthy()
    expect(backendHits).toBe(0)

    withScope(scope, () =>
      mockQueryState(key, { kind: 'error', error: new Error('Network offline') }),
    )
    await tick()
    expect(getByRole(root, 'alert').textContent).toBe('Network offline')
    expect(backendHits).toBe(0)

    withScope(scope, () => mockQueryState(key, { kind: 'ok', value: { name: 'Ada Lovelace' } }))
    await tick()
    expect(getByRole(root, 'heading', { name: 'Ada Lovelace', level: 1 })).toBeTruthy()
    expect(getByText(root, 'Ada Lovelace')).toBeTruthy()
    expect(getByRole(root, 'button', { name: 'Refresh' })).toBeTruthy()
    expect(backendHits).toBe(0)

    withScope(scope, () => clearQueryMocks(key))
    stop()
    scope.dispose()
  })
})
