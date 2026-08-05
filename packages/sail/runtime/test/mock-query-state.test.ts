// Behavioral coverage for the RC test seam: force loading / error / ok
// without a backend, and prove mock lock blocks real fetch + invalidate.

import { describe, expect, it } from 'vitest'
import { createScope, withScope } from '../src/effect-scope'
import { queryKeyed } from '../src/query'
import {
  mockQueryState,
  clearQueryMocks,
  useNativeCache,
} from '../src/native-cache'

const tick = () => new Promise((r) => setTimeout(r, 0))

describe('mockQueryState', () => {
  it('forces ok without calling the fetcher', async () => {
    const scope = createScope()
    let fetches = 0
    const key = ['crm', 'deals', 'list'] as const
    const outer = withScope(scope, () => {
      mockQueryState(key, { kind: 'ok', value: [{ id: 1 }] })
      return queryKeyed(
        () => key,
        () => {
          fetches++
          return Promise.resolve([{ id: 99 }])
        },
      )
    })
    expect(outer()).toEqual({ kind: 'ok', value: [{ id: 1 }] })
    await tick()
    await tick()
    expect(fetches).toBe(0)
    expect(outer()).toEqual({ kind: 'ok', value: [{ id: 1 }] })
    scope.dispose()
  })

  it('forces loading and error without a backend', async () => {
    const scope = createScope()
    const key = ['users', 'me'] as const
    let fetches = 0
    const outer = withScope(scope, () => {
      mockQueryState(key, { kind: 'loading' })
      return queryKeyed(
        () => key,
        () => {
          fetches++
          return Promise.resolve({ name: 'real' })
        },
      )
    })
    expect(outer()).toEqual({ kind: 'loading' })
    await tick()
    expect(fetches).toBe(0)

    withScope(scope, () =>
      mockQueryState(key, { kind: 'error', error: new Error('offline') }),
    )
    const err = outer()
    expect(err.kind).toBe('error')
    if (err.kind === 'error') expect(err.error.message).toBe('offline')
    await tick()
    expect(fetches).toBe(0)
    scope.dispose()
  })

  it('clearQueryMocks allows a subsequent real fetch', async () => {
    const scope = createScope()
    const key = ['clear-me'] as const
    let fetches = 0
    const outer = withScope(scope, () => {
      mockQueryState(key, { kind: 'ok', value: 'mocked' })
      return queryKeyed(
        () => key,
        () => {
          fetches++
          return Promise.resolve('live')
        },
        { staleTime: 0 },
      )
    })
    expect(outer()).toEqual({ kind: 'ok', value: 'mocked' })
    expect(fetches).toBe(0)

    withScope(scope, () => clearQueryMocks(key))
    // After unlock, invalidate triggers background refetch of observed entry.
    withScope(scope, () => useNativeCache().invalidate(key))
    await tick()
    await tick()
    expect(fetches).toBe(1)
    expect(outer()).toEqual({ kind: 'ok', value: 'live' })
    scope.dispose()
  })

  it('invalidate does not overwrite a mock-locked entry', async () => {
    const scope = createScope()
    const key = ['locked'] as const
    let fetches = 0
    withScope(scope, () => {
      mockQueryState(key, { kind: 'ok', value: 'hold' })
      queryKeyed(
        () => key,
        () => {
          fetches++
          return Promise.resolve('should-not-run')
        },
      )
      useNativeCache().invalidate(key)
    })
    await tick()
    await tick()
    expect(fetches).toBe(0)
    scope.dispose()
  })
})
