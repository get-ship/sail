// query() / mutation() — the public read/write primitives that replace
// `live()` (see helm/data/missions/m-deck-standalone/scratch/data-arch-seed.md,
// "the five client primitives", ruling 3).
//
// ---------------------------------------------------------------
// CONTRACT — query() is the ONLY public door to the cache engine.
//
// This header used to live on `queried.ts` (query-core-backed) and
// was reproduced on `native-cache.ts` for the spike. Now that
// `query()` is the public primitive `live()` used to be, THIS is
// where the swap boundary's contract is documented — `queried.ts` and
// `native-cache.ts` are implementation detail underneath it. Swapping
// the engine again means rewriting `queried.ts`/`native-cache.ts`
// alone; everything in this file, and everything above it (components,
// templates, the compiler's emitted code), is written against the
// five clauses below and must not need to change.
//
// 1. SUBSCRIBE. `query()`/`queryKeyed()` return a `Signal<LiveState<T>>`
//    live for the calling scope's lifetime: every transition lands via
//    `.set()` synchronously with the engine's own notification. A
//    derived-key change (see DERIVED IDENTITY below) attaches to a
//    DIFFERENT cache entry rather than refetching the old one, but
//    still forwards through the SAME outer signal object — callers
//    never see the reference change.
//
// 2. READ. Exactly the frozen `LiveState<T>` union — `loading`, `ok`,
//    `error`, never a fourth shape. A background revalidate of
//    already-`ok` data (an invalidate, a stale-while-revalidate
//    remount) never flips the signal back to `loading`.
//
// 3. INVALIDATE. `staleTime`/`gcTime`/`retry` are the only knobs
//    (`QueryOpts`, forwarded unmodified). `invalidate(prefix)` — the
//    same mechanism serving mutation auto-invalidation and the public
//    `invalidate()` grammar — is the one way to force a refetch; see
//    invalidate-design-proposal.md. Every fetch attempt owns a real
//    `AbortController`; dispose/GC/invalidate all abort it for real.
//
// 4. OWNER-TREE HOMING. One cache per owner-tree chain root
//    (`useNativeCache()`, walking `.parent` to `parent === null`),
//    never a module singleton. Two roots never share entries.
//
// 5. UNSUBSCRIBE. Registered via `onCleanup` on the CALLING scope —
//    no signal update after the owning scope is gone.
// ---------------------------------------------------------------
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
import { makeAbortController, type LiveState, type LiveSignal, type LiveSource } from './live-state'
import {
  nativeQuery,
  useNativeCache,
  whenCacheSettled,
  type QueryKey,
  type NativeCacheOpts,
} from './native-cache'

export type { QueryKey } from './native-cache'
/** SSR-only: resolves once every in-flight `query()` PULL has settled into
 *  the cache — see `native-cache.ts`'s `whenCacheSettled` for the contract.
 *  Re-exported here since `query()` is the public door this cache sits
 *  under. */
export { whenCacheSettled }

// query() OWNS the loading/ok/error + push-source public face. These
// shapes live in the leaf `live-state` module only to keep the
// query.ts ↔ native-cache.ts import graph acyclic; query.ts is where
// they are documented and re-exported from.
export type { LiveState, LiveSignal, LiveSource, LiveValueOf } from './live-state'

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
 *
 * PULL vs PUSH is type-directed at the runtime seam: if `fn` returns a
 * `Promise<T>` the entry is filled once and lets go stale (PULL); if it
 * returns a `LiveSource<T>` (a `subscribe` method — the shape a
 * streaming client method returns) the entry is fed by that subscription
 * (PUSH). One `LiveState<T>` face out either way. See `native-cache.ts`'s
 * `fetch`/`nativeQuery`.
 */
export function queryKeyed<T>(
  keyFn: () => QueryKey,
  fn: (signal: AbortSignal) => Promise<T> | LiveSource<T>,
  opts?: QueryOpts,
): Signal<LiveState<T>> {
  const outer = signal<LiveState<T>>({ kind: 'loading' })
  let inner: ReturnType<typeof createChildScope> | null = null

  const stopKeyEffect = effect(() => {
    const key = keyFn() // tracked — a new key on dep change re-runs this block
    untrack(() => {
      inner?.dispose()
      inner = createChildScope()
      const entrySig = withScope(inner, () => nativeQuery(key, fn, opts))
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
  fn: (signal: AbortSignal) => Promise<T> | LiveSource<T>,
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
  /** Return a SETTLED mutation (`ok`/`error`) to `idle` — clears a stale
   *  success banner or error message without re-running the mutation.
   *  The ergonomic escape for forms: submit → `ok` → `reset()` →
   *  `idle` for the next entry. A no-op while `idle` or `loading` (there
   *  is nothing settled to clear); a call in flight is invalidated so a
   *  late settle can't overwrite the `idle` reset. */
  reset: () => void
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
  // Bumped on every `mutate()` start and on `reset()`; a settle whose
  // token has gone stale (a `reset()` landed, or a later `mutate()`
  // started) is discarded so it can't clobber the current state.
  let token = 0

  const mutate = async (): Promise<T> => {
    const myToken = ++token
    state.set({ kind: 'loading' })
    const controller = makeAbortController()
    try {
      const value = await fn(controller.signal)
      if (myToken === token) {
        state.set({ kind: 'ok', value })
        if (__compilerAutoInvalidate) cache.invalidate(__compilerAutoInvalidate)
        for (const prefix of __compilerInvalidates ?? []) cache.invalidate(prefix)
      }
      return value
    } catch (err) {
      const error = toError(err)
      if (myToken === token) state.set({ kind: 'error', error })
      throw error
    }
  }

  const reset = (): void => {
    // Invalidate any in-flight/settled attempt and drop back to idle.
    token++
    if (state.__peek().kind !== 'idle') state.set({ kind: 'idle' })
  }

  return { state, mutate, reset }
}

// ---------------------------------------------------------------------------
// invalidate() — the public escape-hatch primitive.
// ---------------------------------------------------------------------------

/**
 * The compiler's lowering target for author-facing `invalidate(ref)` (see
 * `invalidate-design-proposal.md`). Author source:
 *
 *   invalidate(crm.deals.list)   // exact method
 *   invalidate(crm.deals)        // namespace
 *   invalidate(crm)              // whole client
 *
 * The compiler statically resolves `ref` to a key prefix (root + member
 * path — same resolution law `query()`/`mutation()` use) and rewrites the
 * call to `__invalidate(prefix)`. `invalidate()` itself is never called at
 * runtime — it is fully erased/rewritten at compile time — so this is the
 * ONLY export of the pair that does real work; `invalidate` exists purely
 * as the author-facing name the compiler pattern-matches on.
 *
 * Resolves the current owner-tree's cache via `useNativeCache()` (never a
 * singleton — see native-cache.ts's OWNER-TREE HOMING clause) and applies
 * the prefix match already implemented and tested on
 * `NativeCache.invalidate()`.
 */
export function __invalidate(prefix: QueryKey): void {
  useNativeCache().invalidate(prefix)
}

// ---------------------------------------------------------------------------
// Envelope unwrap helpers — read a `LiveState<T>` signal down to its value.
// (Moved here from the deleted `live.ts`; the jobspec keeps them unchanged.)
// ---------------------------------------------------------------------------

/**
 * Read a `LiveSignal<T>` and return its value or `undefined`.
 *
 * The full discriminated form is `sig().kind === 'ok' ? sig().value : ...`.
 * `unwrap(sig)` collapses that to a single call when the caller only
 * cares about "is the value here?" — typical for derived bodies that
 * pass through optional chains anyway:
 *
 * ```ts
 * derived: {
 *   total: ({ agents }) => unwrap(agents)?.items.length ?? 0,
 * }
 * ```
 *
 * Tracks signal reads — calling `unwrap` inside a `derived` or `effect`
 * subscribes the surrounding scope to the signal, just like calling
 * the signal directly.
 */
export function unwrap<T>(sig: LiveSignal<T>): T | undefined {
  const s = sig()
  return s.kind === 'ok' ? s.value : undefined
}

/**
 * Read a `LiveSignal<T>` and return its value or `fallback`. The
 * fallback applies for both `loading` and `error` states — callers
 * that need to discriminate should keep using the explicit `kind`
 * check or `<while-loading>`/`<while-error>` boundaries.
 *
 * ```ts
 * derived: {
 *   items: ({ agents }) => unwrapOr(agents, { items: [] }).items,
 * }
 * ```
 */
export function unwrapOr<T>(sig: LiveSignal<T>, fallback: T): T {
  const s = sig()
  return s.kind === 'ok' ? s.value : fallback
}
