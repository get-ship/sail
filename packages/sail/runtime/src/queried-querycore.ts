// queried-querycore — the RETIRED @tanstack/query-core-backed
// implementation, kept ONLY as the differential-test reference now
// that the engine swap has landed (see native-cache-torture.test.ts
// section 7 and native-cache-torture-report.md / signals-cache-spike-report.md
// / data-arch-seed.md's "ENGINE RULED: NATIVE" sitting).
//
// This file is the pre-swap `queried.ts`, byte-for-byte except the
// export name (`queried` -> `queriedQueryCore`) and this header. It
// exists purely so the differential suite can keep proving native and
// query-core agree on transition sequences; it is NOT imported by
// `query.ts`, `queried.ts`, or any app/corpus code, and it is deleted
// along with the `@tanstack/query-core` dependency once that suite
// is retired (tracked in the query-slice report).
//
// `queried(key, fn, opts?)` is the query-core-backed sibling of
// `live()`: same LiveSignal<T> face, but backed by a cache with
// dedup, staleness, and GC instead of a bare re-run-on-dep-change
// effect. `live()` itself is UNCHANGED by this module — a separate
// primitive, so cutting query-core in doesn't touch the SSE/patch
// path `live()` also serves.

import type { QueryKey } from '@tanstack/query-core'
import { QueryObserver, QueryClient } from '@tanstack/query-core'
import { signal, type Signal } from './signal'
import { onCleanup, createScope, currentScope, type EffectScope } from './effect-scope'
import type { LiveState } from './live'

// Self-contained root-homed QueryClient resolution — a private copy of
// what `query-client.ts` used to do, so this test-only file has zero
// dependency on the now-native-backed `query-client.ts`.
const idKey = Symbol.for('sail.queryClient.querycoreTwin')

function rootOf(scope: EffectScope): EffectScope {
  let cur: EffectScope = scope
  while (cur.parent) cur = cur.parent
  return cur
}

let ANON_ROOT: EffectScope | null = null
function anonRoot(): EffectScope {
  if (!ANON_ROOT || ANON_ROOT.disposed) ANON_ROOT = createScope()
  return ANON_ROOT
}

function useQueryClientTwin(): QueryClient {
  const current = currentScope()
  const root = rootOf(current ?? anonRoot())
  const existing = root.contexts.get(idKey) as QueryClient | undefined
  if (existing) return existing

  const client = new QueryClient()
  client.mount()
  root.contexts.set(idKey, client)
  root.cleanup(() => {
    client.unmount()
    client.getQueryCache().clear()
  })
  return client
}

/** Options passed through to the underlying `QueryObserver`. */
export interface QueriedOpts {
  staleTime?: number
  gcTime?: number
  retry?: boolean | number
}

/**
 * DIFFERENTIAL-TEST-ONLY. See module header — this is the retired
 * query-core-backed twin of `queried()`, kept solely so
 * native-cache-torture.test.ts can keep proving parity.
 */
export function queriedQueryCore<T>(
  key: QueryKey,
  fn: () => Promise<T>,
  opts?: QueriedOpts,
): Signal<LiveState<T>> {
  const client = useQueryClientTwin()
  const observer = new QueryObserver<T, Error, T, T>(client, {
    queryKey: key,
    queryFn: fn,
    ...(opts?.staleTime !== undefined && { staleTime: opts.staleTime }),
    ...(opts?.gcTime !== undefined && { gcTime: opts.gcTime }),
    ...(opts?.retry !== undefined && { retry: opts.retry }),
  })

  const toLiveState = (result: ReturnType<typeof observer.getCurrentResult>): LiveState<T> => {
    if (result.status === 'pending') return { kind: 'loading' }
    if (result.status === 'error') return { kind: 'error', error: result.error }
    return { kind: 'ok', value: result.data }
  }

  const sig = signal<LiveState<T>>(toLiveState(observer.getCurrentResult()))

  const unsubscribe = observer.subscribe((result) => {
    sig.set(toLiveState(result))
  })

  onCleanup(unsubscribe)

  return sig
}
