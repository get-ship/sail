// query-client — scope-homed @tanstack/query-core QueryClient.
//
// Mirrors store.ts's root-homing idiom: one QueryClient per chain
// root (per-request on the server), NEVER a module singleton — two
// parallel server requests must not share a cache, and a client-side
// app has exactly one root per mount.
//
// Unlike a store instance (arbitrary author-defined shape), a
// QueryClient is a single opaque object with its own internal
// QueryCache; there's nothing to seed from SSR here beyond what
// `dehydrate`/`hydrate` (query-core's own API) already handles at the
// call site. This module only owns WHERE the client lives — not SSR
// payload plumbing, which stays the caller's job.

import { QueryClient } from '@tanstack/query-core'
import { createScope, currentScope, type EffectScope } from './effect-scope'

const idKey = Symbol.for('sail.queryClient')

/** Find the root of `scope`'s chain — the scope with `parent === null`. */
function rootOf(scope: EffectScope): EffectScope {
  let cur: EffectScope = scope
  while (cur.parent) cur = cur.parent
  return cur
}

/** The scope used when `useQueryClient` is called without a current
 *  scope (tests, module-level bootstrap). Lazily created — mirrors
 *  `store.ts`'s `anonRoot`. */
let ANON_ROOT: EffectScope | null = null
function anonRoot(): EffectScope {
  if (!ANON_ROOT || ANON_ROOT.disposed) ANON_ROOT = createScope()
  return ANON_ROOT
}

/**
 * Resolve the QueryClient for the current owner-tree chain root. The
 * first call for a given root allocates a fresh `QueryClient`, calls
 * `.mount()`, and registers `.unmount()` + cache clear as a cleanup
 * on that root — so the cache and its GC timers die when the root
 * scope (a server request, or the app root) disposes. Subsequent
 * calls on any descendant scope return the same instance.
 *
 * NEVER a module singleton: two parallel server requests, each with
 * their own root scope, get two independent clients and caches.
 */
export function useQueryClient(): QueryClient {
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
