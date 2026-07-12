// effect-scope — mount-time cleanup ownership.
//
// Pieces of the runtime that allocate long-lived resources during a
// view's `setup()` — `live()` opens an EventSource, `data()` opens an
// SSE subscription, custom hooks may install global listeners — need
// a way to register a cleanup that fires when the enclosing view
// unmounts. Without this, every `mount()` call leaks the resources
// allocated by its setup body. Tabs OOM'd from exactly this class
// of leak before commit `87cf3223`.
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
    },
    get disposed() {
      return disposed
    },
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
  if (owner) owner.cleanup(() => child.dispose())
  return child
}

/** Run `body` inside a fresh, STANDALONE scope; return its result plus the
 *  scope's disposer.
 *
 *  For the control-flow primitives (`bindFor`, `bindIf`, `bindAwait`): each
 *  rendered child (a row, a branch) must own the effects created while
 *  rendering it, so they tear down when THAT child is removed / swapped — not
 *  when the enclosing reconciler effect re-runs. Rendering the child through
 *  `renderInScope` enrolls its effects on the child's own scope (via each
 *  `effect()`'s `onCleanup`), which the primitive disposes on removal (and on
 *  its own teardown). Standalone — NOT a child of the current scope — because
 *  the primitive owns the child's disposal explicitly; the reconciler effect's
 *  per-run scope must not capture it. */
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

