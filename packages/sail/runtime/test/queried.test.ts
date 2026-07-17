// queried() — post-swap, a thin forward onto `nativeQueried()` (see
// `queried.ts`'s header). The battery of behavioral cases this file
// used to run directly against @tanstack/query-core (dedup, remount
// staleTime, stale-then-revalidate, owner-tree isolation, unmount,
// SSR dehydrate/hydrate) now live on `nativeQueried()` itself in
// native-cache.test.ts and native-cache-torture.test.ts — duplicating
// them here would just be re-testing the forward. This file only
// proves the forward itself holds.

import { describe, expect, it } from 'vitest'
import { createScope, withScope } from '../src/effect-scope'
import { queried } from '../src/queried'

const tick = () => new Promise((r) => setTimeout(r, 0))

describe('queried', () => {
  it('forwards to nativeQueried: starts loading, resolves to ok, honors opts', async () => {
    const scope = createScope()
    const s = withScope(scope, () => queried(['q1'], () => Promise.resolve(42), { staleTime: 1000 }))
    expect(s().kind).toBe('loading')
    await tick()
    await tick()
    expect(s()).toEqual({ kind: 'ok', value: 42 })
  })

  it('maps rejection to the error state', async () => {
    const scope = createScope()
    const s = withScope(scope, () =>
      queried(['q-err'], () => Promise.reject(new Error('boom')), { retry: false }),
    )
    await tick()
    await tick()
    expect(s()).toEqual({ kind: 'error', error: expect.any(Error) })
  })
})
