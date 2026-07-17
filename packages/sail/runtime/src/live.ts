// live() — function-form data binding with a tagged-union lifecycle.
//
// `data()` (in src/data.ts) takes the options-shape `{ initial,
// subscribe }` and is the lower-level primitive the patch layer wires
// into. `live()` is the higher-level form: pass a thunk that returns
// the SDK call result, get back a signal whose value walks the
// loading → ok | error states. Tracked dep changes inside the thunk
// re-fire the source.
//
// Two source shapes are supported:
//   - Promise<T> | T          — Phase-1 round-trip; resolves to ok.
//   - LiveSource<T>           — SSE/patch path; subscribe() owns the
//                               signal lifecycle. The Phase-3 SDK
//                               client returns this shape.

import { effect, signal, untrack, type Signal } from './signal'
import { onCleanup } from './effect-scope'

/** The lifecycle of a `LiveSignal<T>` value. */
export type LiveState<T> =
  | { kind: 'loading' }
  | { kind: 'error'; error: Error }
  | { kind: 'ok'; value: T }

/** A signal that carries the loading/ok/error lifecycle of a live source. */
export type LiveSignal<T> = Signal<LiveState<T>>

/**
 * Source shape for SSE/patch-streamed data. `subscribe` takes ownership
 * of the live signal: it transitions to `ok` on the first patch, applies
 * subsequent patches, and may transition to `error` on transport failure.
 * The returned disposer closes the connection.
 *
 * The Phase-3 ship-set-sdk-client returns this shape from each method;
 * `live()` detects it via the `subscribe` property and wires it through.
 */
export interface LiveSource<T> {
  subscribe: (sig: LiveSignal<T>) => () => void
}

/**
 * The value a `<data source>` call yields, mirroring `live()`'s source union:
 * unwrap a `LiveSource<U>`, await a `Promise<U>`, else the value itself. Used by
 * the compiler-generated `Scope` to type a `<data>` signal from its declared
 * `<client>`'s method return — e.g. `feed.clock(): LiveSource<Tick>` ⇒
 * `LiveValueOf<ReturnType<…>> = Tick`.
 */
export type LiveValueOf<R> = R extends LiveSource<infer U> ? U : Awaited<R>

/** Options for `live()`. */
export interface LiveOpts {
  /**
   * Called when the source rejects (or throws synchronously). Runs
   * after the signal has transitioned to `{ kind: 'error', error }`.
   */
  onError?: (err: unknown) => void
}

const toError = (err: unknown): Error =>
  err instanceof Error ? err : new Error(String(err))

const isLiveSource = <T>(v: unknown): v is LiveSource<T> =>
  typeof v === 'object' &&
  v !== null &&
  typeof (v as { subscribe?: unknown }).subscribe === 'function'

/**
 * Run `source` inside a tracked reactive scope; expose the result as a
 * `LiveSignal<T>` whose value walks `loading → ok | error`.
 *
 * Behavior:
 *
 * - Initial value is `{ kind: 'loading' }`.
 * - `source` runs eagerly. If it returns a Promise (or a sync value
 *   that we wrap), the signal updates to `{ kind: 'ok', value }` on
 *   resolve or `{ kind: 'error', error }` on reject.
 * - If `source` returns a `LiveSource<T>`, `subscribe(sig)` is called
 *   immediately. The subscriber owns transitions — typically pushes
 *   the first SSE patch into `ok`, then applies subsequent patches.
 *   The disposer returned by `subscribe` is invoked on dep-change
 *   re-runs and on tear-down.
 * - Any tracked signal read inside `source` becomes a dep. When a dep
 *   changes, the signal flips back to `loading`, the previous in-flight
 *   resolution is invalidated (token-based), and `source` runs again.
 *
 * @param source Thunk that returns a Promise, a `LiveSource<T>`, or
 *   a sync value. Tracked signal reads inside register as deps.
 * @param opts Optional `onError` hook fired on each rejection.
 * @returns A `LiveSignal<T>` — read it like any other signal.
 *
 * @example
 * ```ts
 * // Promise round-trip
 * const id = state('agent-1')
 * const agent = live(() => ship.agents.get(id()))
 *
 * // SSE/patch (LiveSource)
 * const agents = live(() => ship.agents.summaries())
 * effect(() => {
 *   const s = agents()
 *   if (s.kind === 'ok') console.log(s.value.items.length)
 * })
 * ```
 *
 * @see data for the lower-level options-shape primitive; bindWhileLoading
 *   and bindWhileError for boundary primitives that mount/unmount on
 *   the lifecycle states.
 */
export function live<T>(
  source: () => LiveSource<T> | Promise<T> | T,
  opts?: LiveOpts,
): LiveSignal<T> {
  const sig = signal<LiveState<T>>({ kind: 'loading' })
  let token = 0

  // Capture the effect's stop fn AND register it with the current
  // mount-time owner so the EventSource closes when the view unmounts.
  // Pre-fix: the effect's stop fn was discarded, the SSE subscription
  // never got cleaned up, and every navigation that touched a view
  // with `<data live>` leaked the EventSource + its handlers + the
  // signal graph holding them. After ~tens of navigations the tab
  // OOM'd. (`mount.ts` runs `setup()` inside `withScope` so registered
  // cleanups land in the MountHandle's cleanup list.)
  const stop = effect(() => {
    const myToken = ++token
    // Only `source()` runs tracked — its signal reads (the query the
    // controller builds) are the effect's real deps and the only thing
    // that should re-fire it. Everything that touches `sig` runs under
    // `untrack`: a `LiveSource.subscribe` that reads `sig()` (to merge or
    // accumulate state) must NOT subscribe this effect to `sig`, or its
    // own `sig.set` re-fires the effect → re-calls `source` → re-subscribes
    // → unbounded re-entry → `RangeError: Maximum call stack size exceeded`.
    let result: LiveSource<T> | Promise<T> | T
    try {
      result = source()
    } catch (err) {
      if (myToken !== token) return
      untrack(() => {
        const error = toError(err)
        sig.set({ kind: 'error', error })
        opts?.onError?.(err)
      })
      return
    }

    return untrack(() => {
      sig.set({ kind: 'loading' })

      if (isLiveSource<T>(result)) {
        let disposed = false
        // `disposed` keeps the disposer idempotent (it runs on re-fire and
        // again on unmount). Re-entry is already prevented by `untrack`
        // above — this run is never a `sig` subscriber, so the stream's
        // `sig.set` can't re-fire it.
        const unsub = result.subscribe(sig)
        return () => {
          if (disposed) return
          disposed = true
          try {
            unsub()
          } catch (err) {
            opts?.onError?.(err)
          }
        }
      }

      Promise.resolve(result as Promise<T> | T).then(
        (value) => {
          if (myToken !== token) return
          sig.set({ kind: 'ok', value })
        },
        (err) => {
          if (myToken !== token) return
          const error = toError(err)
          sig.set({ kind: 'error', error })
          opts?.onError?.(err)
        },
      )
      return undefined
    })
  })

  onCleanup(stop)
  return sig
}

// ---------------------------------------------------------------------------
// Envelope unwrap helpers
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
