// data() — server-owned signal with optional SSE patch subscription.
//
// The compiler emits each `data live x = source.query(...)` as a
// data({ initial, subscribe }) call. The signal carries an SSR'd value
// at hydrate time; patches arrive as `patch` SSE events and are routed
// through applyPatch (or a custom handler).
//
// See docs/ship-set/03-runtime-architecture.md § Data primitives.
// See docs/ship-set/05-server-data-bridge.md for the SchemaSource side.

import { applyPatch, type Patch } from './patch'
import { signal, type Signal } from './signal'
import { onCleanup } from './effect-scope'

/** Options for `data()` — initial value plus SSE/patch wiring. */
export interface DataOpts<T> {
  /** Initial value (typically server-rendered). */
  initial: T
  /** SSE endpoint URL; omit for static data (no live subscription). */
  subscribe?: string
  /** Custom patch handler — overrides the default `applyPatch` route. */
  onPatch?: (patch: Patch, sig: Signal<T>) => void
  /** Error handler for SSE / parse failures. */
  onError?: (err: unknown) => void
  /**
   * EventSource constructor injection — defaults to `globalThis.EventSource`.
   * Tests pass a fake; SSR passes `undefined` to skip subscription.
   */
  EventSource?: typeof globalThis.EventSource
}

/** The handle returned by `data()`: the live signal plus a disposer. */
export interface DataHandle<T> {
  /** The reactive signal — read it like any other signal. */
  signal: Signal<T>
  /** Disposer that closes the SSE connection. Idempotent. */
  dispose: () => void
}

/**
 * Create a server-owned reactive signal that can subscribe to SSE
 * patches. If `subscribe` is provided and an `EventSource` impl is
 * available, opens the stream and routes `patch` events through
 * `applyPatch` (or `onPatch` when supplied) into the signal.
 *
 * The ship-set compiler emits this for `data live x = ship.<dom>.<op>(...)`
 * declarations; SSR pre-renders `initial` and the runtime takes over
 * patches at hydrate time.
 *
 * @param opts Initial value, optional SSE URL, and patch / error hooks.
 * @returns `{ signal, dispose }` — read the signal, call `dispose` to
 *   close the stream.
 *
 * @example
 * ```ts
 * const { signal: agents, dispose } = data<{ items: Agent[] }>({
 *   initial: { items: [] },
 *   subscribe: '/api/v1/ship/agents/summaries',
 * })
 * effect(() => console.log(agents().items.length))
 * // later: dispose()
 * ```
 *
 * @see applyPatch for the default patch routing; signal for raw signals.
 */
export function data<T>(opts: DataOpts<T>): DataHandle<T> {
  const sig = signal(opts.initial)
  const ESCtor = opts.EventSource ?? globalThis.EventSource
  if (!opts.subscribe || !ESCtor) {
    return { signal: sig, dispose: () => {} }
  }

  const sse = new ESCtor(opts.subscribe)
  const onMessage = (ev: MessageEvent) => {
    let patch: Patch
    try {
      patch = JSON.parse(ev.data) as Patch
    } catch (err) {
      opts.onError?.(err)
      return
    }
    try {
      if (opts.onPatch) opts.onPatch(patch, sig)
      else applyPatch(sig, patch)
    } catch (err) {
      opts.onError?.(err)
    }
  }
  const onError = (err: Event) => opts.onError?.(err)

  sse.addEventListener('patch', onMessage as EventListener)
  sse.addEventListener('error', onError)

  let disposed = false
  const dispose = (): void => {
    if (disposed) return
    disposed = true
    sse.removeEventListener('patch', onMessage as EventListener)
    sse.removeEventListener('error', onError)
    sse.close()
  }

  // Auto-register the disposer with the current effect scope so
  // callers don't have to remember to call `.dispose()` manually.
  // Outside any scope this is a no-op; callers that want explicit
  // control still receive `dispose` on the returned handle.
  onCleanup(dispose)

  return { signal: sig, dispose }
}
