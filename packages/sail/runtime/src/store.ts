// store — scope-homed shared state.
//
// A store is a unit of stateful logic. The shape it returns is
// whatever the author defines — typically `{ ...signals,
// ...derived, ...actions }`. Signals hold state, derived compute
// from them, actions are plain functions that close over the
// signals. The shape is up to the author; the runtime's only
// concern is (a) where the instance lives, (b) when it gets
// created, and (c) how SSR hands its values to the client.
//
// Where it lives — on an EffectScope. NOT an ESM module singleton.
// Two parallel server requests must not share store state, and a
// scoped view subtree must not see a sibling's overrides; the
// scope tree is the only structure that gives both.
//
// When it gets created — on first `useStore(def)` for a given
// `(def, scope-chain-root)`. The lookup walks `scope.parent`
// toward the root. The root (a scope with `parent === null`) is
// where the instance is parked when no ancestor provides one. A
// subtree that needs its own override calls `provideStore(def)`
// to install an instance on the current scope; descendants see
// that one, sibling subtrees keep seeing the root.
//
// Who owns its effects — the instance itself. `setup()` runs inside
// a dedicated child scope of the HOME scope (the scope the instance
// is parked on), exposed as `__scope` on the instance. Cleanups and
// effects registered during setup die when the HOME dies — never
// when the first CALLER unmounts. A store resolved from the root is
// served to every subtree; tying its internals to whichever
// component happened to touch it first would kill them out from
// under everyone else.
//
// How SSR hands values to the client — see `setStoreSSRPayload`.
// The server runs `setup()` per request, lets any effects seed
// the signals to their post-effect values, then serializes the
// per-store `{ name: value, ... }` snapshot keyed by store id.
// The client, BEFORE the first `useStore(def)` for an id, looks
// up the snapshot and re-seeds the signals via `.set()`. A new
// request on the server starts with an empty payload, so two
// concurrent requests don't see each other's state.
//
// No DOM dependency — this module imports nothing DOM-shaped, and
// the scope tree it walks is the same one `mount.ts` builds on
// both browser and SSR (the SSR entry is `server.ts`).

import { createChildScope, createScope, currentScope, withScope, type EffectScope, walkScope } from './effect-scope'
import { isSignal, type Signal } from './signal'

/**
 * A store definition. Stable identity across the app — call
 * `defineStore(...)` once, at module top-level, and import the
 * resulting `StoreDef` anywhere you need `useStore(def)`. The
 * runtime never mutates the `id`; it is the cache key on every
 * scope.
 */
export interface StoreDef<Shape> {
  /** Unique id. String so it round-trips through SSR payloads
   *  and store-name logging; symbols would survive only in
   *  memory. Convention: domain-prefixed, e.g. `cart`, `ui.theme`. */
  readonly id: string
  /**
   * Allocates a fresh instance. Runs at most once per
   * `(def, scope-chain-root)` — the first `useStore` call
   * creates the root instance; subsequent calls on any
   * descendant scope return the cached one. NEVER throws on
   * re-call: the runtime caches the result.
   */
  setup: () => Shape
}

/** Author-facing instance type — the shape `setup` returns,
 *  augmented with the read-side handle to the underlying
 *  EffectScope. Authors ignore the augmented field. */
export type StoreInstance<Shape> = Shape & {
  /** Owning scope. Useful for diagnostics and for code that
   *  needs to schedule cleanup relative to the store's
   *  lifecycle; not normally reached for. */
  readonly __scope: EffectScope
}

/**
 * Define a store. Pure registration: returns a `StoreDef`; no
 * instance is created until something calls `useStore(def)`.
 *
 * @example
 * ```ts
 * const cart = defineStore('cart', () => {
 *   const items = signal<string[]>([])
 *   const count = derived(() => items().length)
 *   return {
 *     items,
 *     count,
 *     add: (id: string) => items.update((xs) => [...xs, id]),
 *   }
 * })
 * ```
 */
export function defineStore<Shape>(id: string, setup: () => Shape): StoreDef<Shape> {
  return { id, setup }
}

// ── SSR payload handshake ────────────────────────────────────────────────

/**
 * The SSR payload: `{ [storeId]: { [signalName]: serializedValue } }`.
 * Signal values must be JSON-serializable. The server builds one
 * per request; the client reads it before its first `useStore` of
 * the same id and seeds the store's signal initial values from it.
 *
 * Kept in module state (not on the scope) so the SSR-render phase
 * can write to it without depending on a specific scope being
 * installed, and so the client can read it before any scope is
 * installed. The server entry clears this between requests.
 */
let SSR_PAYLOAD: Record<string, Record<string, unknown>> | null = null

/**
 * Server-side: install the per-request SSR payload. Replaces any
 * previous payload (a previous request's). The SSR entry calls
 * this once at the top of every request, before mounting.
 */
export function setStoreSSRPayload(payload: Record<string, Record<string, unknown>>): void {
  SSR_PAYLOAD = payload
}

/** Server-side: drop the SSR payload — call at the END of a
 *  request, after the rendered HTML has been emitted. Defends
 *  against a slow request leaving its state in the next one's
 *  payload slot. Idempotent. */
export function clearStoreSSRPayload(): void {
  SSR_PAYLOAD = null
}

/**
 * Read the seed values for `id`. Client-side only — on the
 * server the payload is consumed by the snapshot writer, not by
 * `useStore`. Returns `null` if no payload is installed or the
 * id has no entry. Does NOT mutate the payload; the seed pass
 * is idempotent.
 */
export function readStoreSSRSeed(id: string): Record<string, unknown> | null {
  if (!SSR_PAYLOAD) return null
  return SSR_PAYLOAD[id] ?? null
}

// ── instance cache ────────────────────────────────────────────────────────

/** Symbol per store id, used as the contexts-map key. Strings
 *  would also work; symbols are slightly faster to hash and
 *  can't collide with `provideContext` keys. */
const idKey = (id: string): symbol => Symbol.for(`sail.store.${id}`)

/**
 * Return the existing instance on `scope` for `def`, or `null`
 * if neither this scope nor any ancestor has one. Internal —
 * `useStore` / `provideStore` own the create-on-miss policy.
 */
function lookupInstance<Shape>(def: StoreDef<Shape>, scope: EffectScope): Shape | null {
  const k = idKey(def.id)
  const found = walkScope<unknown>(
    scope,
    (s) => s.contexts.get(k),
    undefined,
  )
  return found === undefined ? null : (found as Shape)
}

/** Find the root of `scope`'s chain — the scope with
 *  `parent === null`. The root is where root-resolved
 *  instances live, so the runtime can share them across all
 *  subtrees. */
function rootOf(scope: EffectScope): EffectScope {
  let cur: EffectScope = scope
  while (cur.parent) cur = cur.parent
  return cur
}

/** The scope used when `useStore` / `provideStore` is called
 *  without a current scope (no `withScope` on the stack). Tests
 *  and module-level bootstrap fall into this case. Lazily
 *  created — there is no global "root" the runtime holds at
 *  rest, so this is purely a fallback for the off-scope path. */
let ANON_ROOT: EffectScope | null = null
function anonRoot(): EffectScope {
  if (!ANON_ROOT || ANON_ROOT.disposed) {
    // `createScope` is pure — no DOM, no signals — so the
    // lazy init is safe in any order.
    ANON_ROOT = createScope()
  }
  return ANON_ROOT
}

// ── public API ───────────────────────────────────────────────────────────

/** Internal: seed a freshly created store's signals from the
 *  SSR payload. Walks the shape, finds every `Signal`, and
 *  calls `.set(payload[name])` if the payload has a value for
 *  that name. Runs synchronously inside `useStore` BEFORE the
 *  result is returned to the consumer — so no effect has had
 *  a chance to subscribe yet, and the first read sees the
 *  seeded value (no flash). */
function seedFromSSR<Shape>(def: StoreDef<Shape>, instance: Shape): void {
  const seed = readStoreSSRSeed(def.id)
  if (!seed) return
  for (const k of Object.keys(seed)) {
    const v = seed[k]
    if (v === undefined) continue
    const entry = (instance as Record<string, unknown>)[k]
    if (isSignal(entry)) {
      // `.set` is signal's write API. The signal's initial
      // value was set by `setup()`; this overrides it before
      // any subscriber attaches.
      ;(entry as Signal<unknown>).set(v)
    }
  }
}

/**
 * Create the instance for `def`, homed on `home`: `setup()` runs
 * inside a fresh child scope of `home`, so every `onCleanup` /
 * effect registered during setup dies exactly when `home` dies
 * (root disposal for root instances, subtree disposal for
 * `provideStore` overrides) — NOT when the first caller's scope
 * does. The child scope rides the instance as non-enumerable
 * `__scope` (see `StoreInstance`); `snapshotStore`'s
 * `Object.keys` walk never sees it.
 */
function instantiate<Shape>(def: StoreDef<Shape>, home: EffectScope): Shape {
  const storeScope = createChildScope(home)
  const instance = withScope(storeScope, () => def.setup())
  seedFromSSR(def, instance)
  if (instance !== null && typeof instance === 'object') {
    Object.defineProperty(instance, '__scope', {
      value: storeScope,
      enumerable: false,
      writable: false,
      configurable: false,
    })
  }
  home.contexts.set(idKey(def.id), instance)
  return instance
}

/**
 * Resolve a store instance. Walks the current scope's parent
 * chain; returns the first instance found. If none is found on
 * the chain, allocates a fresh one on the CHAIN ROOT (so all
 * sibling subtrees see the same root instance — the unit of
 * sharing is the request / mount root, not the local scope).
 *
 * SSR-aware: on the client, if an SSR payload exists for
 * `def.id`, the new instance is seeded from it before being
 * returned. The seed happens before the caller can subscribe,
 * so the first effect that reads the store sees the SSR
 * value, not the `setup()` initial.
 *
 * Never returns `undefined` — the chain root always exists (a
 * per-request SSR scope, the mount scope, or a lazily-created
 * anonymous root in the no-scope case). The type reflects
 * this: `Shape`, not `Shape | undefined`.
 */
export function useStore<Shape>(def: StoreDef<Shape>): Shape {
  const current = currentScope()
  const start = current ?? anonRoot()
  // 1. Walk the chain first — a descendant's `provideStore`
  //    would have planted an instance on an ancestor.
  const found = lookupInstance(def, start)
  if (found !== null) return found
  // 2. No instance on the chain. Create one, homed on the chain
  //    root, so all subtrees of the same request share it and its
  //    internals live as long as the root does.
  return instantiate(def, rootOf(start))
}

/**
 * Force a SCOPED instance on the current scope. Use when a
 * subtree needs to override the root instance — e.g. an
 * editor's preview mode wants a separate cart from the
 * header's. Descendants of THIS scope see the override;
 * sibling subtrees keep the root. Disposing the current
 * scope drops the override and reverts the subtree to the
 * next-up ancestor (the root, in the common case).
 */
export function provideStore<Shape>(def: StoreDef<Shape>): Shape {
  const k = idKey(def.id)
  const current = currentScope()
  if (!current) {
    // No scope: behave like useStore — plant on the anon
    // root, the only anchor we have. Without a scope there
    // is no subtree to "own" the instance, so the override
    // is indistinguishable from a root.
    const start = anonRoot()
    const existing = lookupInstance(def, start)
    if (existing !== null) return existing
    return instantiate(def, start)
  }
  // Idempotent: re-call on the same scope returns the existing
  // override (the SAME shape, with the SAME signals). This is
  // the caller's contract — multiple consumers in a subtree
  // share one override, not N copies.
  const existing = current.contexts.get(k)
  if (existing !== undefined) return existing as Shape
  // No override on this scope yet. Create one — even if an
  // ancestor (typically the chain root) already has an instance
  // for the same id. That's the whole point of `provideStore`:
  // a NEW scope-homed copy that DESCENDANTS of THIS scope will
  // resolve, while sibling subtrees keep the ancestor's
  // instance — homed HERE, so it dies when this subtree does.
  return instantiate(def, current)
}

/** Returns the instance if one is already on the chain, or
 *  `null` if not. Does NOT create. Use when code wants to
 *  react to "is a store mounted yet?" without forcing it. */
export function peekStore<Shape>(def: StoreDef<Shape>): Shape | null {
  const current = currentScope()
  if (!current) {
    const root = ANON_ROOT
    if (!root) return null
    return lookupInstance(def, root)
  }
  return lookupInstance(def, current)
}

// ── snapshot (server-side serialization helper) ───────────────────────────

/**
 * Build a `{ [signalName]: value }` snapshot of every Signal
 * in `instance`. Used by the SSR entry to serialize the
 * request's store state into the HTML payload. Only top-level
 * signals are captured; derived and actions are skipped
 * (derived re-derive from signals, actions don't carry
 * state). Nested stores — an instance whose shape contains
 * another store's instance — are NOT recursively walked; the
 * outer store's owner is responsible for serializing
 * sub-stores it embeds.
 */
export function snapshotStore<Shape>(instance: Shape): Record<string, unknown> {
  const out: Record<string, unknown> = {}
  for (const k of Object.keys(instance as object)) {
    if (k === '__scope') continue
    const v = (instance as Record<string, unknown>)[k]
    if (isSignal(v)) {
      // Read the current value (no tracking — this is a
      // one-shot snapshot, not a reactive read). If a Signal
      // throws on read (it shouldn't — reads are pure), the
      // call site catches and skips.
      try {
        out[k] = (v as Signal<unknown>)()
      } catch {
        // Skip unreadable signals — they aren't part of the
        // payload the client can reconstruct.
      }
    }
  }
  return out
}

/** Internal — exposed for the SSR entry so it can drive the
 *  snapshot pass without re-importing `isSignal`. */
export const _internal = { idKey, rootOf }

// ── SSR bootstrap (client side) ───────────────────────────────────────────

/**
 * Name of the global the SSR entry writes the per-request store
 * payload to. The client entry reads it via `hydrateStoreSSRPayload`.
 * The `<script>` tag with this payload is emitted into the HTML
 * BEFORE the view bundle, so by the time any `useStore` runs the
 * payload is available.
 */
export const SSR_PAYLOAD_GLOBAL = '__SUI_STORES__'

/**
 * Ingest the SSR payload from the global. Client-side: call once
 * at app bootstrap (BEFORE the first `useStore`). Idempotent —
 * safe to call more than once; the second call replaces the
 * current payload (a no-op if no global exists).
 */
export function hydrateStoreSSRPayload(): void {
  if (typeof globalThis === 'undefined') return
  // Narrow via typeof — never inline-assert a fabricated shape
  // on a global. The descriptor check guards against the
  // payload being null / a primitive.
  const raw = (globalThis as unknown as Record<string, unknown>)[SSR_PAYLOAD_GLOBAL]
  if (raw && typeof raw === 'object') {
    SSR_PAYLOAD = raw as Record<string, Record<string, unknown>>
  }
}

/**
 * Server-side: walk `scope`'s contexts map, find every store
 * instance (key description starts with `sail.store.`), snapshot
 * each, and return the full per-request payload. The SSR entry
 * calls this at end-of-request and passes the result to
 * `setStoreSSRPayload` / renders it into the HTML.
 */
export function snapshotScopeStores(scope: EffectScope): Record<string, Record<string, unknown>> {
  const out: Record<string, Record<string, unknown>> = {}
  for (const [sym, val] of scope.contexts) {
    const desc = sym.description
    if (!desc || !desc.startsWith('sail.store.')) continue
    // Store id is everything after the prefix.
    const id = desc.slice('sail.store.'.length)
    if (val && typeof val === 'object') {
      out[id] = snapshotStore(val as object)
    }
  }
  return out
}
