// Combined acceptance path (SPEC-TEST-1 FIX):
//   1. sail compile + mount via renderRoute (not hand-painted DOM)
//   2. mockQueryState / clearQueryMocks force loading → error → ok
//   3. assert only with @ship/sail-test role/name/text (no structure / data-*)
//   4. zero backend hits while mocked

import { describe, expect, it } from 'vitest'
import { queryKeyed } from '@ship/sail-runtime'
import { renderRoute } from '@ship/test-setup'
import {
  getByRole,
  getByText,
  queryByRole,
  mockQueryState,
  clearQueryMocks,
} from '../src/index'

const PROFILE_KEY = ['acceptance', 'profile'] as const

type Profile = { name: string }

const TEMPLATE = `
<column>
  <while-loading {profile}>
    <div role="status" aria-label="Loading profile">Loading…</div>
  </while-loading>
  <while-error {profile} as="err">
    <div role="alert">{err.message}</div>
  </while-error>
  <if {profile().kind === 'ok'}>
    <h1>{profile().value.name}</h1>
    <button type="button">Refresh</button>
  </if>
</column>
`

describe('acceptance: sail compile + mockQueryState + semantic queries', () => {
  it('forces loading → error → ok on a compiled route with zero backend hits', async () => {
    let backendHits = 0

    const v = renderRoute(TEMPLATE, {
      controller: () => {
        // Setup runs inside renderRoute's owner-tree scope — same cache as query.
        mockQueryState(PROFILE_KEY, { kind: 'loading' })
        const profile = queryKeyed<Profile>(
          () => PROFILE_KEY,
          () => {
            backendHits++
            return Promise.resolve({ name: 'from-network' })
          },
          { retry: false },
        )
        return { profile }
      },
    })

    await v.flush()
    expect(getByRole(v.root, 'status', { name: 'Loading profile' })).toBeTruthy()
    expect(queryByRole(v.root, 'alert')).toBeNull()
    expect(queryByRole(v.root, 'heading')).toBeNull()
    expect(backendHits).toBe(0)

    v.inScope(() =>
      mockQueryState(PROFILE_KEY, {
        kind: 'error',
        error: new Error('Network offline'),
      }),
    )
    await v.flush()
    expect(getByRole(v.root, 'alert').textContent).toBe('Network offline')
    expect(queryByRole(v.root, 'status', { name: 'Loading profile' })).toBeNull()
    expect(backendHits).toBe(0)

    v.inScope(() =>
      mockQueryState(PROFILE_KEY, {
        kind: 'ok',
        value: { name: 'Ada Lovelace' },
      }),
    )
    await v.flush()
    expect(getByRole(v.root, 'heading', { name: 'Ada Lovelace', level: 1 })).toBeTruthy()
    expect(getByText(v.root, 'Ada Lovelace')).toBeTruthy()
    expect(getByRole(v.root, 'button', { name: 'Refresh' })).toBeTruthy()
    expect(queryByRole(v.root, 'alert')).toBeNull()
    expect(queryByRole(v.root, 'status', { name: 'Loading profile' })).toBeNull()
    expect(backendHits).toBe(0)

    v.inScope(() => clearQueryMocks(PROFILE_KEY))
    v.unmount()
  })
})
