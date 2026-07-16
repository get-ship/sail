// effect-scope — mount-time cleanup ownership.
//
// Pieces of the runtime that allocate long-lived resources during a
// view's `setup()` — `live()` opens an EventSource, `data()` opens an
// SSE subscription, custom hooks may install global listeners — need
// a way to register a cleanup that fires when the enclosing view
// unmounts. Without this, every `mount()` call leaks the resources
// allocated by its setup body. Tabs OOM'd from exactly this class of
// leak before commit `87cf3223`.
//
// Pattern: `mount.ts` wraps the call to the view module's `setup()`
// in `withScope(scope, () => setup(ctx))`. Allocators inside that
// scope call `onCleanup(fn)` to enroll a teardown function; the
// scope's `dispose()` runs them all in reverse insertion order.
// Outside any scope, `onCleanup` is a no-op (so the API is safe to
// call from non-mount contexts like tests, long-lived globals, or
// SSR shims).
//
// Nesting: scopes can nest. `withScope(parent, () => withScope(child,
// () => …))` enrolls inner cleanups on `child`. When `parent`
// disposes, `child` disposes too — `createChildScope` registers the
// child's `dispose` as a cleanup of the parent automatically. Matches
// Solid's `createRoot` / Vue's `effectScope` semantics.
//
// Stays free of any DOM / runtime concerns so it can be imported
// from `live.ts`, `data.ts`, hooks, etc. without circular deps.
//
// Owner graph: every EffectScope (except the synthetic one returned
// from `createRootScope()`) has a `parent` pointer, set at creation
// by `createChildScope`. The chain is the lookup path for the
// scope-homed context API (see `store.ts` — `useStore(def)` walks
// `scope.parent` until it finds an instance or hits the root). The
// root scope is identified by `parent === null` AND it being the
// only one with no owner; the runtime holds no global "root" (a
// per-request SSR scope is a fresh root that disposes with the
// request).
//
// Per-scope context map: each scope carries a `Map<symbol, unknown>`
// keyed by the store/context `id`. Map, not object-with-`__sui_*`
// keys: insertion is O(1) on the hot path, the symbol key can't
// collide with anything an author names, and walking the chain
// stays tight (parent → grandparent → …) instead of dragging DOM
// shadow-root hops behind it.

/** A unit of resource ownership. Cleanups registered while the scope
 *  is the current scope (via `withScope`) run when the scope is
 *  disposed. */
export interface EffectScope {
  /** Register a teardown function. If `dispose` already ran, runs the
   *  cleanup immediately so the resource releases. */
  cleanup(fn: () => void): void
  /** Tear down. Runs every cleanup in reverse insertion order. Errors
   *  in any one cleanup are logged and don't abort the rest. Calling
   *  `dispose` more than once is a no-op. */
  dispose(): void
  /** Becomes true after `dispose` has run. */
  readonly disposed: boolean
  /** Parent scope — `null` for a root. Read-only; set by
   *  `createChildScope` at construction. Walking `parent` is the
   *  scope-homed `useStore` / `provideContext` lookup. */
  readonly parent: EffectScope | null
  /** Per-scope key→value map. Store instances (from `store.ts`)
   *  live here keyed by the store's `id` symbol; arbitrary context
   *  values from `provideContext` keyed by the caller's symbol.
   *  Internal — direct reads bypass the type-narrowing that
   *  `useStore` / `injectContext` provide, so call those instead. */
  readonly contexts: Map<symbol, unknown>
}

let CURRENT_SCOPE: EffectScope | null = null

/** Run `body` with `scope` as the current scope. Cleanups registered
 *  inside `body` enroll on `scope`. Restores the prior scope on
 *  return, including on throw. Nestable. */
export function withScope<R>(scope: EffectScope, body: () => R): R {
  const prev = CURRENT_SCOPE
  CURRENT_SCOPE = scope
  try {
    return body()
  } finally {
    CURRENT_SCOPE = prev
  }
}

/** Register a teardown function with the current scope. No-op when
 *  there is no current scope (i.e. called outside any `withScope`). */
export function onCleanup(fn: () => void): void {
  const scope = CURRENT_SCOPE
  if (!scope) return
  scope.cleanup(fn)
}

/** The active scope, or `null` if no `withScope` is currently on the
 *  stack. Diagnostic / advanced use only — most callers should use
 *  `onCleanup` directly. */
export function currentScope(): EffectScope | null {
  return CURRENT_SCOPE
}

/** Create a fresh, top-level scope. Most allocators don't need this —
 *  they call `onCleanup` to enroll on whatever scope `mount()` set
 *  up. Reach for `createScope` only when you're authoring a HOST
 *  (`mount.ts` is the canonical example) that owns a scope. */
export function createScope(): EffectScope {
  const cleanups: Array<() => void> = []
  let disposed = false
  const contexts = new Map<symbol, unknown>()
  const scope: EffectScope = {
    cleanup(fn) {
      if (disposed) {
        // Late registration after dispose — run immediately so the
        // resource isn't leaked. Matches the `mount.ts` async-mount
        // disposer guard.
        try { fn() } catch (e) { console.error('[sui/scope] late cleanup threw:', e) }
        return
      }
      cleanups.push(fn)
    },
    dispose() {
      if (disposed) return
      disposed = true
      for (let i = cleanups.length - 1; i >= 0; i--) {
        try {
          cleanups[i]!()
        } catch (e) {
          console.error('[sui/scope] cleanup threw:', e)
        }
      }
      cleanups.length = 0
      contexts.clear()
    },
    get disposed() {
      return disposed
    },
    // The `parent` is fixed at construction. A scope created via
    // `createScope()` (not `createChildScope`) is a root — the
    // runtime does not maintain a single global root, so each
    // request/host gets its own. The map is created here so the
    // shape is the same for every scope; root vs. child only
    // differs in `parent`.
    parent: null,
    contexts,
  }
  return scope
}

/** Create a child scope owned by `parent` (or the current scope if
 *  `parent` is omitted). The child auto-disposes when its owner
 *  disposes — registered as a cleanup at creation. Use when you
 *  need a sub-lifecycle inside a mount: e.g. a hook that tears down
 *  its own resources independently of the outer view but should still
 *  clean up if the view unmounts first. */
export function createChildScope(parent?: EffectScope): EffectScope {
  const owner = parent ?? CURRENT_SCOPE
  const child = createScope()
  // The child IS a fresh scope; its `parent` points at the owner
  // (the scope that, if disposed, should take the child with it).
  // We re-define `parent` after construction — the literal returned
  // by `createScope` set it to `null`; the child needs the link.
  // The `contexts` map is fresh per child, so a `provideContext` /
  // `useStore` on the child does not leak into a sibling subtree.
  Object.defineProperty(child, 'parent', {
    value: owner,
    writable: false,
    enumerable: true,
    configurable: false,
  })
  if (owner) owner.cleanup(() => child.dispose())
  return child
}

/** Run `body` inside a fresh, STANDALONE scope; return its result plus the
 *  scope's disposer.
 *
 * For the control-flow primitives (`bindFor`, `bindIf`): each
 * rendered child (a row, a branch) must own the effects created while
 * rendering it, so they tear down when THAT child is removed / swapped — not
 * when the enclosing reconciler effect re-runs. Rendering the child through
 * `renderInScope` enrolls its effects on the child's own scope (via each
 * `effect()`'s `onCleanup`), which the primitive disposes on removal (and on
 * its own teardown). Standalone — NOT a child of the current scope — because
 * the primitive owns the child's disposal explicitly; the reconciler effect's
 * per-run scope must not capture it. */
export function renderInScope<T>(body: () => T): [T, () => void] {
  const scope = createScope()
  const value = withScope(scope, body)
  return [value, () => scope.dispose()]
}

/**
 * `withScope`, but the arguments are PASSED rather than closed over.
 *
 * `withScope(scope, () => fn(a, b))` allocates an arrow every call. That is fine once;
 * it is not fine once per row. `bindFor` was reaching the row render through
 *
 *     renderInScope(() => runProtected(anchor, 'render', () => render(item, i)))
 *
 * which costs FIVE allocations before the render body runs: two call-site arrows, the
 * scope, the returned tuple, and the disposer closure inside `renderInScope`. Threading
 * the args through removes four of the five; the scope is the only one that is real.
 */
export function withScopeArgs<A, B, R>(
  scope: EffectScope,
  fn: (a: A, b: B) => R,
  a: A,
  b: B,
): R {
  const prev = CURRENT_SCOPE
  CURRENT_SCOPE = scope
  try {
    return fn(a, b)
  } finally {
    CURRENT_SCOPE = prev
  }
}

/**
 * Walk the scope chain from `start` toward the root, calling `visit`
 * at each node (including `start`). Stops when `visit` returns
 * something other than `undefined` — that value is returned — or
 * the chain runs out, in which case the `fallback` is returned.
 *
 * Internal: the scope-homed `useStore` and `injectContext` both
 * resolve with this primitive. The walk is parent-only (not
 * DFS siblings); matches the model — a scope only sees what its
 * ancestors provided.
 */
export function walkScope<R>(
  start: EffectScope | null,
  visit: (scope: EffectScope) => R | undefined,
  fallback: R,
): R {
  let cur: EffectScope | null = start
  while (cur) {
    const v = visit(cur)
    if (v !== undefined) return v
    cur = cur.parent
  }
  return fallback
}

/**
 * Key for an arbitrary value held in a scope's `contexts` map. Use
 * a `Symbol(...)` so the key can't collide with anything an author
 * names. The non-store context API (`provideContext` /
 * `injectContext`) is a thin layer over the same map stores use —
 * reach for it when you need to share a value that ISN'T a
 * setup-style store (a single ref, a logger handle, an ad-hoc
 * client). For stateful shared logic, `defineStore` is the answer.
 */
export type ContextKey<T> = symbol & { readonly __ctx?: T }

/** Mint a typed context key. The phantom `__ctx` is for type
 *  inference only — it carries through `provideContext` /
 *  `injectContext` so the read site doesn't need an annotation. */
export function contextKey<T>(description: string): ContextKey<T> {
  return Symbol(description) as ContextKey<T>
}

/** Put `value` on `scope` under `key`. Any descendant that calls
 *  `injectContext(key)` and walks the chain will find it. No-op
 *  when called without a current scope; tests / module-level
 *  helpers can call it explicitly with a scope. */
export function provideContext<T>(key: ContextKey<T>, value: T, scope?: EffectScope): void {
  const target = scope ?? CURRENT_SCOPE
  if (!target) return
  target.contexts.set(key, value)
}

/** Look up a value published via `provideContext`. Walks the
 *  parent chain, returning the nearest match. Returns `null` when
 *  no scope on the chain has a value under `key` — same null-vs-
 *  throw choice the old `useShared` made (callers guard with
 *  `if (v)` / controller-side null checks). The TS narrowing
 *  comes from the `ContextKey<T>` phantom, so the result IS `T`,
 *  not `T | null` — the null collapses only at the call site
 *  where it matters. */
export function injectContext<T>(key: ContextKey<T>, scope?: EffectScope): T | null {
  const start = scope ?? CURRENT_SCOPE
  if (!start) return null
  let cur: EffectScope | null = start
  while (cur) {
    const v = cur.contexts.get(key)
    if (v !== undefined) return v as T
    cur = cur.parent
  }
  return null
}
