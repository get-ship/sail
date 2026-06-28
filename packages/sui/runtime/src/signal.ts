// SUI runtime — fine-grained reactive signals.
//
// Solid-shaped: signals are sources, derived are memoized computeds,
// effects subscribe and re-run on dep change. Reading inside a tracked
// scope (effect/derived) auto-subscribes; writing notifies subscribers.
//
// Scheduling is a two-phase, glitch-free, colored push-pull graph
// (the Reactively/alien-signals shape):
//
//   • WRITE (push, no compute): a changed signal marks its direct
//     observers DIRTY and their transitive observers CHECK ("a source
//     *might* have changed"), and queues any reachable effects. Nothing
//     recomputes yet.
//   • FLUSH (pull, in dependency order): each queued effect is settled
//     via `updateIfNecessary` — a CHECK node first asks each source to
//     settle, recomputing itself only if a source actually changed; a
//     DIRTY node recomputes. A derived that recomputes to an equal value
//     does NOT propagate, so downstream stays put.
//
// This guarantees an effect runs exactly once per logical change, never
// observes a half-updated graph (the diamond glitch), and each node
// recomputes at most once per cycle (no exponential blow-up on stacked
// diamonds). Dependencies are re-collected each run, so conditional reads
// (`cond() ? a() : b()`) never leak stale subscriptions.
//
// Mirrors `crates/sui-signals` in intent; note the JS scheduler is now
// glitch-free where the Rust SSR mirror has not yet adopted the same
// algorithm — client/server derived parity (a tracked condition) needs
// the Rust side to follow.

import { findNearestBoundary } from './error-boundary'
import { onCleanup } from './effect-scope'

/** Legacy alias retained for the public `Signal.__subs` field shape. */
type Subscriber = () => void
type Cleanup = () => void

// Node colors. Annotated `number` (not literal) so control-flow analysis
// doesn't assume a narrowed literal survives a mutating call — settling a
// source inside the CHECK loop can flip `state` to DIRTY out of band.
const CLEAN: number = 0
const CHECK: number = 1
const DIRTY: number = 2

/** A node that can be read (and thus observed): signals and deriveds. */
interface SourceNode {
  observers: Set<Computation>
}

/** A reactive computation: a derived (memoized value) or an effect. */
interface Computation extends SourceNode {
  fn: () => unknown
  state: number
  /** Forward edges — the sources this computation read on its last run. */
  sources: SourceNode[]
  /** Custom teardown thunks (effect cleanups, selector bucket removals). */
  cleanups: Cleanup[]
  readonly isEffect: boolean
  active: boolean
  /** Discriminates a computation from a bare signal source. */
  readonly computation: true
  /** Derived: the memoized value. */
  value?: unknown
  /** Effect: the error-boundary host captured at creation. */
  host?: Node | null
}

interface SignalNode<T> extends SourceNode {
  value: T
}

let CURRENT_OBSERVER: Computation | null = null
let BATCH_DEPTH = 0
let QUEUE: Computation[] = []
let RUNNING = false

// Host stack — pushed/popped by the component factory wrapper so any
// effect created during a component's setup captures `this` as its
// host for error-boundary routing. Outside any setup the stack is
// empty and effects fall through to console.error on throw.
const HOST_STACK: Node[] = []

/** Push a host on the stack while `fn` runs. Used by the component
 *  factory wrapper so effects/deriveds created during setup capture
 *  it. */
export function withHost<T>(host: Node, fn: () => T): T {
  HOST_STACK.push(host)
  try {
    return fn()
  } finally {
    HOST_STACK.pop()
  }
}

function currentHost(): Node | null {
  return HOST_STACK.length > 0 ? HOST_STACK[HOST_STACK.length - 1]! : null
}

/**
 * A reactive signal — a callable getter paired with a setter and a
 * subscribe handle.
 *
 * Reading inside an `effect` or `derived` automatically registers a
 * dependency. Writing marks dependents and flushes effects once (or
 * coalesces the flush to the end of an enclosing `batch`).
 *
 * @see signal to construct one; derived for memoized reads; effect for
 *   side effects.
 */
export interface Signal<T> {
  /** Read the current value (and track in any active reactive scope). */
  (): T
  /**
   * Write a new value. Accepts either the next value directly or an
   * updater function that receives the previous value.
   */
  set(value: T | ((prev: T) => T)): void
  /** package-private — the set of computations observing this source. */
  __subs: Set<Subscriber>
  /** package-private — current value without tracking (patch applier, benches). */
  __peek(): T
}

/**
 * The one signal type-guard. A signal is a callable getter carrying a `.set`
 * writer. This is the single source of truth — controller-resolver, mount,
 * ref, and the mount-state proxy all import it rather than re-deriving the
 * shape (which had drifted into 2-check and 3-check copies).
 */
export function isSignal(v: unknown): v is Signal<unknown> {
  return (
    typeof v === 'function' &&
    'set' in (v as object) &&
    typeof (v as unknown as { set: unknown }).set === 'function'
  )
}

// ── Graph plumbing ───────────────────────────────────────────────────────────

/** Subscribe the current observer (if any) to `src`, recording both edges. */
function track(src: SourceNode): void {
  const obs = CURRENT_OBSERVER
  if (obs === null) return
  if (src.observers.has(obs)) return
  src.observers.add(obs)
  obs.sources.push(src)
}

/**
 * Mark `node` at least `state` (CHECK or DIRTY). On the first transition
 * out of CLEAN we propagate CHECK to observers (deriveds) or queue the
 * effect; later upgrades (CHECK→DIRTY) don't re-propagate — observers are
 * already CHECK and effects already queued.
 */
function stale(node: Computation, state: number): void {
  if (node.state >= state) return
  const wasClean = node.state === CLEAN
  node.state = state
  if (node.isEffect) {
    if (wasClean && node.active) QUEUE.push(node)
  } else if (wasClean) {
    for (const obs of node.observers) stale(obs, CHECK)
  }
}

function isComputation(src: SourceNode): src is Computation {
  return (src as Computation).computation === true
}

/** Run and clear a node's custom teardown thunks. */
function runCleanups(node: Computation): void {
  const cs = node.cleanups
  if (cs.length === 0) return
  node.cleanups = []
  for (let i = 0; i < cs.length; i++) {
    try {
      cs[i]!()
    } catch (e) {
      console.error('[sui] cleanup threw:', e)
    }
  }
}

/** Drop this computation from every source it was subscribed to. */
function unlinkSources(node: Computation): void {
  const srcs = node.sources
  for (let i = 0; i < srcs.length; i++) srcs[i]!.observers.delete(node)
  node.sources = []
}

/** Recompute a derived; if its value changed, mark observers DIRTY. */
function refreshDerived(node: Computation): void {
  runCleanups(node)
  unlinkSources(node)
  // Optimistic CLEAN before running, so a (pathological) re-mark during
  // the run is not clobbered after it.
  node.state = CLEAN
  const prev = CURRENT_OBSERVER
  CURRENT_OBSERVER = node
  let next: unknown
  try {
    next = node.fn()
  } finally {
    CURRENT_OBSERVER = prev
  }
  if (!Object.is(next, node.value)) {
    node.value = next
    // Value changed: upgrade observers so they recompute. Direct bump —
    // no recursion, no enqueue: every reachable effect was already queued
    // during the write's mark phase, and observers are already CHECK. Skip
    // the in-flight reader (CURRENT_OBSERVER): it is reading our fresh value
    // right now and must not be re-dirtied by it.
    for (const obs of node.observers) {
      if (obs !== CURRENT_OBSERVER && obs.state < DIRTY) obs.state = DIRTY
    }
  }
}

/** Run an effect body, capturing any returned cleanup. Errors route to
 *  the nearest error boundary (or console) — one bad effect never breaks
 *  the graph. */
function runEffect(node: Computation): void {
  runCleanups(node)
  unlinkSources(node)
  node.state = CLEAN
  const prev = CURRENT_OBSERVER
  CURRENT_OBSERVER = node
  try {
    const c = node.fn()
    if (typeof c === 'function') node.cleanups.push(c as Cleanup)
  } catch (e) {
    const boundary = node.host ? findNearestBoundary(node.host) : null
    if (boundary) {
      boundary.catch(e, 'effect')
    } else {
      console.error('[sui] effect threw:', e)
    }
  } finally {
    CURRENT_OBSERVER = prev
  }
}

/**
 * Settle a DERIVED to CLEAN, recomputing only if a source actually
 * changed. The heart of glitch-freedom: a CHECK node pulls each source
 * first, so it never recomputes against a half-updated graph, and
 * recomputes at most once. Effects are settled via `runReaction`, never
 * here (nothing reads an effect, so an effect is never a source).
 */
function updateIfNecessary(node: Computation): void {
  if (node.state === CLEAN) return
  if (node.state === CHECK) {
    const srcs = node.sources
    for (let i = 0; i < srcs.length; i++) {
      const s = srcs[i]!
      if (isComputation(s)) {
        updateIfNecessary(s)
        // s recomputing to a changed value upgrades us to DIRTY.
        if (node.state === DIRTY) break
      }
    }
  }
  if (node.state === DIRTY) refreshDerived(node)
  // A spurious re-dirty during recompute (a source read mid-run) is
  // resolved here: the fresh value already incorporates it.
  node.state = CLEAN
}

/**
 * Process a queued effect: settle its sources, then run it iff a source
 * actually changed (a CHECK with no changed source is suppressed). Unlike
 * a derived, we do NOT force CLEAN afterward — a write performed during
 * the run re-enqueues the effect and leaves it DIRTY, so a self-updating
 * effect drives to its fixpoint instead of stalling after one pass.
 */
function runReaction(node: Computation): void {
  if (!node.active) return
  if (node.state === CHECK) {
    const srcs = node.sources
    for (let i = 0; i < srcs.length; i++) {
      const s = srcs[i]!
      if (isComputation(s)) {
        updateIfNecessary(s)
        if (node.state === DIRTY) break
      }
    }
  }
  if (node.state === DIRTY) runEffect(node)
  else node.state = CLEAN
}

/** Runaway backstop — far above any real cascade (which settles in a
 *  handful of rounds). Trips only when a reaction keeps scheduling itself
 *  with no fixpoint (e.g. an effect writing a signal it reads, unbounded). */
const MAX_FLUSH_ROUNDS = 100_000

/** Drain the effect queue. Re-entrant writes just enqueue; the outermost
 *  flush loop picks them up. */
function flush(): void {
  if (RUNNING) return
  RUNNING = true
  let rounds = 0
  try {
    while (QUEUE.length > 0) {
      if (++rounds > MAX_FLUSH_ROUNDS) {
        QUEUE = []
        throw new Error(
          `[sui] reactivity: aborting flush after ${MAX_FLUSH_ROUNDS} rounds — ` +
            'a reaction keeps scheduling itself (likely an effect writing a ' +
            'signal it reads with no stable fixpoint).',
        )
      }
      const batchRun = QUEUE
      QUEUE = []
      for (let i = 0; i < batchRun.length; i++) runReaction(batchRun[i]!)
    }
  } finally {
    RUNNING = false
  }
}

// ── Public primitives ─────────────────────────────────────────────────────────

/**
 * Create a fine-grained reactive signal.
 *
 * The returned function is the reader: call it inside a `derived` or
 * `effect` to subscribe. `.set(next)` writes a new value (skipped via
 * `Object.is` if unchanged), marks dependents, and flushes dependent
 * effects exactly once. Writes inside `batch` defer the flush to the
 * end of the outermost batch.
 *
 * @param initial Starting value the reader returns until the first set.
 * @returns A signal: callable getter + `.set` writer.
 *
 * @example
 * ```ts
 * const count = signal(0)
 * effect(() => console.log('count:', count()))
 * count.set(1)              // → 'count: 1'
 * count.set((n) => n + 1)   // → 'count: 2'
 * ```
 *
 * @see derived for computed values; effect for side effects; batch to
 *   coalesce multi-signal writes.
 */
export function signal<T>(initial: T): Signal<T> {
  const node: SignalNode<T> = { value: initial, observers: new Set() }

  const sig = (() => {
    track(node)
    return node.value
  }) as Signal<T>

  sig.set = (next) => {
    const v =
      typeof next === 'function'
        ? (next as (prev: T) => T)(node.value)
        : next
    if (Object.is(v, node.value)) return
    node.value = v
    // Mark phase (no recompute). Iterating directly is safe: the
    // observer set only mutates during a recompute, which happens in
    // flush, after this loop.
    for (const obs of node.observers) stale(obs, DIRTY)
    if (BATCH_DEPTH === 0) flush()
  }
  sig.__subs = node.observers as unknown as Set<Subscriber>
  sig.__peek = () => node.value

  return sig
}

/**
 * A memoized computed — reads its dependencies on first access and
 * recomputes lazily when any of them change. Behaves as a read-only
 * signal: callable getter, callable from inside other reactive scopes.
 * Calling `.set` on a derived throws.
 */
export interface Derived<T> extends Signal<T> {}

/**
 * Create a memoized computed value. `fn` runs on first read to seed the
 * cache, then recomputes on demand when a tracked dependency changes.
 *
 * Recompute is pull-based: a dependency change marks the derived dirty
 * but does not recompute it until something reads it (or a downstream
 * effect is flushed). Results are referentially memoized — when a recompute
 * yields an `Object.is`-equal value, downstream observers are NOT notified
 * (so a derived returning the same ref doesn't churn its effects). With no
 * observers, a derived stays lazy. Dependencies are re-collected each run,
 * so conditional reads (`if cond() { a() } else { b() }`) never leak stale
 * subscriptions.
 *
 * @param fn Pure read function; tracked reads inside register as deps.
 * @returns A signal-like reader. Calling `.set` throws.
 *
 * @example
 * ```ts
 * const first = signal('Ada')
 * const last = signal('Lovelace')
 * const full = derived(() => `${first()} ${last()}`)
 * effect(() => console.log(full())) // → 'Ada Lovelace'
 * first.set('Grace')                 // → 'Grace Lovelace'
 * ```
 *
 * @see signal for sources; effect for side effects.
 */
export function derived<T>(fn: () => T): Derived<T> {
  const node: Computation = {
    fn: fn as () => unknown,
    state: DIRTY,
    sources: [],
    observers: new Set(),
    cleanups: [],
    isEffect: false,
    active: true,
    computation: true,
    value: undefined,
  }

  const reader = (() => {
    track(node)
    updateIfNecessary(node)
    return node.value as T
  }) as Derived<T>
  reader.set = () => {
    throw new Error('cannot set on a derived; mutate its dependencies instead')
  }
  reader.__subs = node.observers as unknown as Set<Subscriber>
  reader.__peek = () => {
    updateIfNecessary(node)
    return node.value as T
  }

  return reader
}

/**
 * Run `fn` once eagerly, then re-run on every tracked dependency
 * change. Returns a disposer that unsubscribes and stops further runs.
 *
 * `fn` may return a cleanup callback. The cleanup runs once before each
 * re-run (so users can detach listeners, abort fetches, etc.) and once
 * on disposal. Errors thrown inside `fn` or its cleanup are logged and
 * swallowed — one bad effect never breaks the reactive graph.
 *
 * Dependencies are re-collected each run: signals read in run N but not in
 * run N+1 are unsubscribed. Disposal proactively unsubscribes from every
 * source the effect was tracking.
 *
 * @param fn Side-effecting function; tracked reads register as deps.
 *   May return an optional cleanup callback.
 * @returns A disposer that detaches the effect.
 *
 * @example
 * ```ts
 * const url = signal('/api/feed')
 * const dispose = effect(() => {
 *   const ctrl = new AbortController()
 *   fetch(url(), { signal: ctrl.signal }).catch(() => {})
 *   return () => ctrl.abort()  // cleanup runs before next re-run
 * })
 * // later...
 * dispose()
 * ```
 *
 * @see signal, derived; batch to coalesce writes that would otherwise
 *   re-fire the effect multiple times.
 */
export function effect(fn: () => void | (() => void)): () => void {
  const node: Computation = {
    fn: fn as () => unknown,
    state: DIRTY,
    sources: [],
    observers: new Set(),
    cleanups: [],
    isEffect: true,
    active: true,
    computation: true,
    // Capture host at creation so subsequent re-runs (which happen
    // outside any `withHost` scope) still know which boundary subtree
    // they belong to.
    host: currentHost(),
  }

  // Eager initial run. Writes triggered during it schedule + flush
  // through the re-entrancy-guarded loop.
  runEffect(node)

  return () => {
    if (!node.active) return
    node.active = false
    runCleanups(node)
    unlinkSources(node)
    node.state = CLEAN
  }
}

/**
 * Membership selector — a derived-shaped accessor returning whether
 * `key` matches `source()`'s current value. The point: when the
 * source changes, only the two affected buckets (the OLD value and
 * the NEW value) re-notify their subscribers. Other keys are left
 * alone — no re-evaluation, no DOM thrash.
 *
 * The Krausest motivation: SUI's bench has
 *
 *   class:danger={row.id === selected()}
 *
 * For 1000 rows this puts 1000 effects in `selected`'s subscriber
 * set; a `selected.set(newId)` notifies all 1000, every one
 * re-evaluates the comparison. With `selector`:
 *
 *   const isSelected = selector(selected)
 *   // template: class:danger={isSelected(row.id)}
 *
 * Each row's effect subscribes to its OWN per-id bucket inside the
 * selector. `selected.set(newId)` notifies only the previously-
 * selected row's bucket and the newly-selected row's bucket — two
 * effects fire, not a thousand.
 *
 * @example
 * ```ts
 * const selected = signal<string | null>(null)
 * const isSelected = selector(selected)
 *
 * for (const item of items()) {
 *   effect(() => {
 *     // re-runs only when item.id's selection membership flips.
 *     if (isSelected(item.id)) console.log('me!')
 *   })
 * }
 * ```
 */
export function selector<T>(source: Signal<T>): (key: T) => boolean {
  const buckets = new Map<T, Set<Computation>>()
  let current = source.__peek()
  // One ambient effect subscribes to source. On change, mark exactly
  // the two affected buckets (old + new) and advance current.
  const stopSelector = effect(() => {
    const next = source()
    if (Object.is(next, current)) return
    notifyBucket(buckets.get(current))
    notifyBucket(buckets.get(next))
    current = next
  })
  // Enroll the ambient effect's teardown with the current scope so it stops on
  // unmount — like live()'s effect (live.ts onCleanup(stop)). Without this it
  // stays subscribed to `source` forever, accumulating one dead observer per
  // selector() call across navigations.
  onCleanup(stopSelector)
  return (key: T): boolean => {
    const obs = CURRENT_OBSERVER
    if (obs !== null) {
      let bucket = buckets.get(key)
      if (!bucket) {
        bucket = new Set()
        buckets.set(key, bucket)
      }
      if (!bucket.has(obs)) {
        const b = bucket
        b.add(obs)
        obs.cleanups.push(() => b.delete(obs))
      }
    }
    return Object.is(key, current)
  }
}

function notifyBucket(set: Set<Computation> | undefined): void {
  if (!set || set.size === 0) return
  // Snapshot — re-tracking during flush mutates bucket membership.
  for (const obs of [...set]) stale(obs, DIRTY)
}

/**
 * Group multiple signal writes into one notification cycle. Dependent
 * effects are marked during `fn` and flushed exactly once when the
 * outermost `batch` returns.
 *
 * Nests safely — only the outermost batch flushes. Used by the patch
 * applier to keep multi-op SSE patches atomic against derived/effects.
 *
 * @param fn Code that writes one or more signals.
 *
 * @example
 * ```ts
 * const x = signal(0), y = signal(0)
 * effect(() => console.log(x() + y()))   // logs 0
 * batch(() => {
 *   x.set(1)
 *   y.set(2)
 * })                                     // logs 3 once (not 1, then 3)
 * ```
 *
 * @see applyPatches which uses batch to apply many patches atomically.
 */
export function batch(fn: () => void): void {
  BATCH_DEPTH++
  try {
    fn()
  } finally {
    BATCH_DEPTH--
    if (BATCH_DEPTH === 0) flush()
  }
}

/**
 * Read signals without subscribing the current observer to them.
 * Useful inside effects/derived when a value is consulted but
 * shouldn't trigger re-runs (timestamps, RNG seeds, peeking at peers).
 *
 * @param fn Code that reads signals; reads inside are not tracked.
 * @returns Whatever `fn` returns.
 *
 * @example
 * ```ts
 * const count = signal(0), label = signal('hits')
 * effect(() => {
 *   const stamp = untrack(() => Date.now())     // no dep on Date
 *   console.log(`${label()}: ${count()} @${stamp}`)
 * })
 * ```
 *
 * @see effect, derived.
 */
export function untrack<T>(fn: () => T): T {
  const prev = CURRENT_OBSERVER
  CURRENT_OBSERVER = null
  try {
    return fn()
  } finally {
    CURRENT_OBSERVER = prev
  }
}

/**
 * A signal that lags `input` by `ms` milliseconds. Each write to
 * `input` restarts the timer; the returned signal only updates after
 * `ms` have elapsed without further writes.
 *
 * Useful for search-as-you-type, where you want to debounce the input
 * before firing a query (`live(() => ship.search(query()))` reading
 * the debounced signal will only re-run once the user stops typing).
 *
 * @example
 * ```ts
 * const query = signal('')
 * const debounced = debounce(query, 250)
 * effect(() => console.log('search:', debounced()))
 * query.set('h'); query.set('he'); query.set('hel') // only 'hel' fires
 * ```
 */
export function debounce<T>(input: Signal<T>, ms: number): Signal<T> {
  const out = signal(untrack(() => input()))
  let timer: ReturnType<typeof setTimeout> | null = null

  effect(() => {
    const value = input()
    if (timer !== null) clearTimeout(timer)
    timer = setTimeout(() => {
      timer = null
      out.set(value)
    }, ms)
    return () => {
      if (timer !== null) {
        clearTimeout(timer)
        timer = null
      }
    }
  })

  return out
}
