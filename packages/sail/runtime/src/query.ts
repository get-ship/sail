// query() / mutation() — public read/write primitives over the cache engine.
//
// Returns Signal<LiveState<T>>; background revalidate never flips ok→loading.
// One cache per owner-tree root (not a module singleton).

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
/** Resolves when every in-flight `query()` pull has settled (SSR). */
export { whenCacheSettled }

export type {
  LiveState,
  LiveSignal,
  LiveSource,
  LiveValueOf,
  OptimisticHandle,
} from './live-state'

/** Options for `query()`/`queryKeyed()` (`staleTime`/`gcTime`/`retry`). */
export type QueryOpts = NativeCacheOpts

/**
 * Explicit-key read: tracked `keyFn`; each new key attaches to a different
 * cache entry. Outer `Signal<LiveState<T>>` identity is stable across key
 * changes. `Promise` → pull; `LiveSource` → push.
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
 * Public read primitive. Compiled code supplies a hidden key-fn as the
 * third arg; uncompiled calls without it throw (use `queryKeyed`).
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
 * Mutation lifecycle: four states. Unlike `LiveState` (reads start
 * fetching immediately), mutations need `idle` before the first call so
 * "about to run" and "running" stay distinct for submit gating.
 */
export type MutationState<T> =
  | { kind: 'idle' }
  | { kind: 'loading' }
  | { kind: 'ok'; value: T }
  | { kind: 'error'; error: Error }

/**
 * Author-facing options. `invalidates` is method REFERENCES (never strings);
 * the compiler erases it and passes resolved prefixes via `__compilerInvalidates`.
 */
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
 * Public write primitive. On success, invalidates `__compilerAutoInvalidate`
 * (namespace of the mutated method) plus any `__compilerInvalidates` prefixes.
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
 * Compiler lowering target for author-facing `invalidate(ref)`.
 * Runtime form is a key-prefix invalidate on the owner-tree cache
 * (via `useNativeCache()`, not a module singleton).
 */
export function __invalidate(prefix: QueryKey): void {
  useNativeCache().invalidate(prefix)
}

// ---------------------------------------------------------------------------
// Envelope unwrap helpers — read a `LiveState<T>` signal down to its value.
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
