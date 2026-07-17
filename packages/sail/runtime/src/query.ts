// query() / mutation() — the public read/write primitives that replace
// `live()` (see helm/data/missions/m-deck-standalone/scratch/data-arch-seed.md,
// "the five client primitives", ruling 3). `query()` is the ONLY public
// door to the cache engine — see native-cache.ts's CONTRACT header for
// the swap boundary this file sits on top of.
//
// ---------------------------------------------------------------------
// DERIVED IDENTITY — keys are never handwritten.
//
// Author-facing source:
//
//   const deals = query(() => crm.deals.list({ stage: stage() }))
//
// The COMPILER — not this file — reads that thunk statically: the
// imported client binding (`crm`) is the key root, the static member
// path (`deals.list`) is the key segments, and the call args become a
// TRACKED key tail (signal reads inside the args, e.g. `stage()`, are
// the deps that mint a NEW derived key when they change — a different
// identity, never a refetch of the old one). The compiler lowers the
// one-thunk call above into:
//
//   query(() => crm.deals.list({ stage: stage() }),
//         opts,
//         () => ['crm', 'deals', 'list', { stage: stage() }])
//
// i.e. it appends the derived KEY-FN as a third, compiler-only
// argument. `query()` calling itself with only two arguments outside
// compiled .sail source is a refusal, not a silent guess — see the
// thrown error below. Direct/uncompiled callers (tests, tooling) use
// `queryKeyed()`, which takes the key-fn explicitly and IS the
// compiler's real lowering target.
//
// A thunk the compiler cannot derive a key from (computed member
// access, a non-imported client, no call at all) is a compile error
// with did-you-mean teaching text — same muscle as `sx_vars.rs`'s
// `(--x)` resolution: derive or refuse, no third path. See the
// compiler crate's `check::query_keys` module.
// ---------------------------------------------------------------------

import { signal, effect, untrack, type Signal } from './signal'
import { onCleanup, createChildScope, withScope } from './effect-scope'
import type { LiveState } from './live'
import { nativeQueried, useNativeCache, type QueryKey, type NativeCacheOpts } from './native-cache'

export type { QueryKey } from './native-cache'

/** Options accepted by `query()`/`queryKeyed()` — identical knobs to
 *  the cache engine (`staleTime`/`gcTime`/`retry`), nothing more. */
export type QueryOpts = NativeCacheOpts

/**
 * Explicit-key read primitive — the compiler's lowering target for
 * `query()` (see module header) and the supported entry point for
 * direct/uncompiled callers (tests, tooling, Storybook).
 *
 * `keyFn` is TRACKED: it re-runs on every dependency change, and each
 * new key it produces attaches to a DIFFERENT cache entry — the old
 * attachment is released (its observer refcount drops, the entry may
 * GC on its own schedule) rather than refetched in place. The returned
 * `Signal<LiveState<T>>` is one stable object across key changes; this
 * function forwards whichever entry is current into it.
 */
export function queryKeyed<T>(
  keyFn: () => QueryKey,
  fn: (signal: AbortSignal) => Promise<T>,
  opts?: QueryOpts,
): Signal<LiveState<T>> {
  const outer = signal<LiveState<T>>({ kind: 'loading' })
  let inner: ReturnType<typeof createChildScope> | null = null

  const stopKeyEffect = effect(() => {
    const key = keyFn() // tracked — a new key on dep change re-runs this block
    untrack(() => {
      inner?.dispose()
      inner = createChildScope()
      const entrySig = withScope(inner, () => nativeQueried(key, fn, opts))
      const stopForward = effect(() => {
        outer.set(entrySig())
      })
      inner.cleanup(stopForward)
    })
  })

  onCleanup(() => {
    stopKeyEffect()
    inner?.dispose()
  })

  return outer
}

/**
 * The public read primitive. Author-facing source calls this with
 * TWO arguments — `query(fn, opts?)` — and the compiler appends the
 * derived key-fn as a hidden third argument during compilation (see
 * module header). Calling `query()` directly with no compiler-derived
 * key REFUSES loudly rather than guessing one: identity must be
 * statically derivable, never runtime-inferred.
 *
 * @throws if called without the compiler-supplied key-fn — use
 *   `queryKeyed(keyFn, fn, opts)` for direct/uncompiled calls.
 */
export function query<T>(
  fn: (signal: AbortSignal) => Promise<T>,
  opts?: QueryOpts,
  __compilerKeyFn?: () => QueryKey,
): Signal<LiveState<T>> {
  if (!__compilerKeyFn) {
    throw new Error(
      '[sail] query() was called without a compiler-derived cache key. ' +
        '`query(fn, opts?)` must be invoked from compiled .sail controller ' +
        'code, where the compiler statically derives the key from the ' +
        'thunk (imported client + member path + call args). For direct or ' +
        'uncompiled use (tests, tooling), call `queryKeyed(keyFn, fn, opts)` instead.',
    )
  }
  return queryKeyed(__compilerKeyFn, fn, opts)
}

// ---------------------------------------------------------------------------
// mutation() — the write primitive.
// ---------------------------------------------------------------------------

/**
 * PROPOSED resting-state union for `mutation()` — NOT frozen, NOT the
 * same union `LiveState<T>` governs for reads. Flagged in
 * query-rename-slice-spec.md for explicit owner sign-off before this
 * is documented as final. The frozen three-state `LiveState<T>`
 * governs READS; a mutation additionally needs a resting state before
 * its first call (`idle`), which a read never has (a read starts
 * fetching immediately, a mutation waits to be triggered) — so
 * folding `idle` into `loading` would make "about to run" and
 * "running" indistinguishable, which callers gating a submit button
 * on `state().kind !== 'loading'` need to tell apart.
 */
export type MutationState<T> =
  | { kind: 'idle' }
  | { kind: 'loading' }
  | { kind: 'ok'; value: T }
  | { kind: 'error'; error: Error }

/** Author-facing options. `invalidates` is method REFERENCES, resolved
 *  by the compiler's static pass to key prefixes — never strings. The
 *  compiler erases this field from the runtime call entirely, passing
 *  the resolved prefixes via the hidden `__compilerInvalidates`
 *  argument (see module header for the analogous `query()` lowering). */
export interface MutationOpts {
  invalidates?: unknown[]
}

export interface Mutation<T> {
  /** Current lifecycle state — starts `idle`, never refetches on its
   *  own (unlike a query, nothing runs until `mutate()` is called). */
  state: Signal<MutationState<T>>
  /** Trigger the mutation now: re-runs `fn` fresh (a fresh read of
   *  whatever signals it closes over, e.g. `draft()`), transitions
   *  `state` through the lifecycle, and — on success — invalidates
   *  the mutated method's namespace plus anything in `invalidates`.
   *  Resolves/rejects in step with `state`. */
  mutate: () => Promise<T>
}

const toError = (err: unknown): Error => (err instanceof Error ? err : new Error(String(err)))

/**
 * The public write primitive. Author-facing source:
 *
 *   const save = mutation(() => crm.deals.update(draft()))
 *   // save.mutate() triggers it; save.state reads the lifecycle.
 *
 * Auto-invalidation: a call to `crm.deals.update` invalidates the
 * `crm.deals.*` namespace on success — the compiler derives that
 * namespace from the same static member-path analysis `query()` uses
 * and passes it as `__compilerAutoInvalidate`. `opts.invalidates`
 * extends/overrides via method REFERENCES only
 * (`invalidates: [crm.deals.list, billing.summary]`), resolved by the
 * compiler into `__compilerInvalidates` prefixes. See
 * invalidate-design-proposal.md for the full grammar and the shared
 * `NativeCache.invalidate(prefix)` mechanism.
 */
export function mutation<T>(
  fn: (signal: AbortSignal) => Promise<T>,
  opts?: MutationOpts,
  __compilerAutoInvalidate?: QueryKey,
  __compilerInvalidates?: QueryKey[],
): Mutation<T> {
  void opts // erased at compile time; kept only for the author-facing type
  const cache = useNativeCache()
  const state = signal<MutationState<T>>({ kind: 'idle' })

  const mutate = async (): Promise<T> => {
    state.set({ kind: 'loading' })
    const controller = new AbortController()
    try {
      const value = await fn(controller.signal)
      state.set({ kind: 'ok', value })
      if (__compilerAutoInvalidate) cache.invalidate(__compilerAutoInvalidate)
      for (const prefix of __compilerInvalidates ?? []) cache.invalidate(prefix)
      return value
    } catch (err) {
      const error = toError(err)
      state.set({ kind: 'error', error })
      throw error
    }
  }

  return { state, mutate }
}
