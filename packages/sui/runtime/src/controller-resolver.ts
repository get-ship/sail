// controller-resolver.ts — runtime backing for the typed controller contract.
//
// Takes a `ControllerDef`, walks its sections, and returns a flat
// bindings record the existing `hydrate()` setup-merge path can
// consume. The resolver does no DOM work — it builds the read-only /
// writable scope proxies, memoizes deriveds, composes `use:` hooks,
// and wraps `on` handlers.
//
// Per the "state is declarative-only" ADR, the resolver no longer
// constructs state signals from a `def.state` section. State signals
// arrive in `ctx.scope` (built by hydrate from the template's `<state>`
// directives) and the resolver consumes them as-is. The writable proxy
// allows assignments to fall through to those signals' `.set()`.
//
// `ControllerHook` (composed via `use:`) still owns its own state-
// bearing shape — hook resolution lives in `./controller-hook-resolver`.

import type { Signal } from './signal'
import { isSignal } from './signal'
import type {
  ControllerDef,
  ControllerHook,
  ControllerShape,
  MutableState,
  ScopedControllerDef,
} from './controller'
import { resolveHook } from './controller-hook-resolver'
import { createMountState } from './state-proxy'
import { pickShared, provide } from './context'
import type {
  ResolverContext,
  ResolvedBindings,
} from './controller-resolver-types'

export type { ResolverContext, ResolvedBindings } from './controller-resolver-types'

/**
 * Non-enumerable key on resolved bindings carrying the controller's
 * lifecycle hooks + the writable state proxy `mount`/`unmount` need.
 */
export const LIFECYCLE_KEY = '__shipSetLifecycle'

/** Lifecycle bundle stored under `LIFECYCLE_KEY`. */
export interface ControllerLifecycle {
  /** Writable state mutator the lifecycle hooks receive as their first arg. */
  state: Record<string, unknown>
  /** Optional `mount(state, el)` from the controller def. May return a cleanup. */
  mount?: (state: Record<string, unknown>, el: HTMLElement) => (() => void) | void
  /** Optional `unmount(state)` from the controller def. */
  unmount?: (state: Record<string, unknown>) => void
}

/**
 * Resolve a controller def into the flat bindings shape `hydrate` can
 * merge into the template scope.
 *
 * @param def The controller def — object form (tier 2) or function form (tier 3).
 * @param ctx The setup-time context (`ship`, `route`, `navigate`, `scope`, `stateKeys`).
 * @returns A flat record of bindings keyed by name.
 */
export function resolveController(
  def: ControllerDef<object>,
  ctx: ResolverContext,
): ResolvedBindings {
  const shape = normalize(def, ctx)
  const baseScope: Record<string, unknown> = {
    ship: ctx.ship,
    clients: ctx.clients ?? {},
    ...(ctx.clients ?? {}),
    route: ctx.route ?? {},
    navigate: ctx.navigate ?? ((_path: string) => {}),
    ...(ctx.scope ?? {}),
  }

  // `scope:` factory runs first — its returned entries become part of
  // the merged scope before any other section. Idiomatic TypeScript
  // closure: signals, derived, hooks, shared wrappers all live here.
  // Pass ctx.host as second arg so consumers can call useShared(host, tag).
  const scopeAdds = shape.scope
    ? (shape.scope as (s: unknown, h?: Element) => Record<string, unknown>)(baseScope, ctx.host)
    : {}
  Object.assign(baseScope, scopeAdds)

  const stateSignals = pickStateSignals(baseScope, ctx.stateKeys, scopeAdds)
  // The one mount-state builder. Reads return unwrapped values
  // (state.count++); the raw signal is reached via state.$signals.x.
  const writableProxy = createMountState(
    baseScope,
    stateSignals,
  ) as MutableState<Record<string, Signal<unknown>>>

  const declScope: Record<string, unknown> = { ...baseScope }

  const useResults = buildUseResults(shape, declScope, ctx)
  for (const [k, v] of Object.entries(useResults)) {
    declScope[k] = v
  }

  const onHandlers = buildOnHandlers(shape, writableProxy)

  const bindings: ResolvedBindings = {
    ...scopeAdds,
    ...onHandlers,
    ...useResults,
  }

  // Auto-publish any signals marked with `shared(...)` to the host so
  // descendant components can resolve via `useShared(host, '<tag>')`.
  // The host element is the custom element itself (provided via ctx).
  if (Object.keys(scopeAdds).length > 0 && ctx.host) {
    const sharedEntries = pickShared(scopeAdds)
    if (Object.keys(sharedEntries).length > 0) {
      const hostEl = ctx.host as Element
      const tag = hostEl.tagName.toLowerCase()
      provide(hostEl, tag, sharedEntries)
    }
  }

  if (shape.mount || shape.unmount) {
    const lifecycle: Record<string, unknown> = {
      state: writableProxy as Record<string, unknown>,
    }
    if (shape.mount) lifecycle.mount = shape.mount
    if (shape.unmount) lifecycle.unmount = shape.unmount
    // Enumerable so the compiled view's `Object.assign($scope, __r)` carries
    // the lifecycle through to the view module's setup return value, where
    // attach.ts::extractLifecycle reads it. Non-enumerable swallowed mount
    // entirely — confirmed by ship-ui-studio's async mount silently dropping
    // catalog state. No code iterates bindings keys, so safe to expose.
    Object.defineProperty(bindings, LIFECYCLE_KEY, {
      value: lifecycle as unknown as ControllerLifecycle,
      enumerable: true,
      configurable: true,
      writable: false,
    })
  }

  return bindings
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

function normalize(
  def: ControllerDef<object>,
  ctx: ResolverContext,
): ControllerShape<object> {
  if (typeof def === 'function') {
    return def({ ship: ctx.ship, clients: ctx.clients ?? {} })
  }
  return def
}

/**
 * Extract the writable-state subset of an ambient scope. State signals
 * are identified by `stateKeys` (template `<state>` directives) AND the
 * keys returned by the controller's `scope:` factory — both are author-
 * declared writable signals. Without `stateKeys` and without `scopeAdds`
 * we fall back to the "any callable value with a `.set` method" heuristic.
 */
function pickStateSignals(
  scope: Record<string, unknown>,
  stateKeys?: ReadonlySet<string>,
  scopeAdds?: Record<string, unknown>,
): Record<string, Signal<unknown>> {
  const out: Record<string, Signal<unknown>> = {}
  const haveStateKeys = stateKeys !== undefined
  const scopeAddKeys = scopeAdds ? Object.keys(scopeAdds) : []
  if (haveStateKeys || scopeAddKeys.length > 0) {
    if (stateKeys) {
      for (const k of stateKeys) {
        const v = scope[k]
        if (isSignal(v)) out[k] = v
      }
    }
    for (const k of scopeAddKeys) {
      if (k in out) continue
      const v = scope[k]
      if (isSignal(v)) out[k] = v
    }
    return out
  }
  for (const [k, v] of Object.entries(scope)) {
    if (isSignal(v)) out[k] = v
  }
  return out
}



function buildOnHandlers(
  shape: ControllerShape<object>,
  writableProxy: MutableState<Record<string, Signal<unknown>>>,
): Record<string, (...args: unknown[]) => unknown> {
  const section = shape.on ?? {}
  const out: Record<string, (...args: unknown[]) => unknown> = {}
  for (const [k, fn] of Object.entries(section)) {
    out[k] = (...args: unknown[]) =>
      (fn as (state: unknown, ...rest: unknown[]) => unknown)(writableProxy, ...args)
  }
  return out
}

function buildUseResults(
  shape: ControllerShape<object>,
  declScope: Record<string, unknown>,
  ctx: ResolverContext,
): Record<string, ResolvedBindings> {
  if (shape.use === undefined) return {}
  const useSection =
    typeof shape.use === 'function'
      ? (shape.use as (s: unknown) => Record<string, ControllerHook>)(declScope)
      : (shape.use as Record<string, ControllerHook>)
  const out: Record<string, ResolvedBindings> = {}
  for (const [k, hookShape] of Object.entries(useSection)) {
    out[k] = resolveHook(hookShape, ctx, declScope)
  }
  return out
}

// ---------------------------------------------------------------------------
// `controller(def)` — author-facing entry point
// ---------------------------------------------------------------------------

/**
 * The shape `controller(def)` returns — a setup-shaped export the
 * daemon's hydrate path consumes verbatim. The daemon extracts `.setup`
 * and calls it with a `ResolverContext` (built from the hydrate flow);
 * `setup` runs the resolver and returns flat bindings the template
 * merges into scope.
 */
export interface ControllerExport<Scope = object> {
  /** Setup callback — invoked by hydrate with the resolver context. */
  setup: (ctx: ResolverContext) => ResolvedBindings
  /** Original def; for tooling. */
  __def: ControllerDef<Scope>
}

/**
 * Author-facing helper. Wrap a `ControllerDef` with `controller({...})`
 * to:
 *
 *   1. Get section signatures inferred without writing the generic
 *      parameter explicitly.
 *   2. Get a `{ setup, __def }` export the daemon's hydrate path can
 *      call directly.
 *
 * @example
 * ```ts
 * // tier 2 — object form
 * // <state name="count" value="0" /> in the template
 * export default controller({
 *   derived: { doubled: ({ count }) => count() * 2 },
 *   on:      { incr: (s) => { s.count++ } },
 * })
 * ```
 *
 * @example
 * ```ts
 * // tier 3 — function form
 * export default controller(({ ship }) => ({
 *   on: { refresh: () => ship.agents.refresh() },
 * }))
 * ```
 */
// CANONICAL form: `controller(def)` with no explicit type argument. Both
// `Ctx` (from the `scope` factory's param annotation) and `SC` (from its
// return) are inferred → precise, cast-free handler state over `Ctx & SC`,
// single call, no currying. This overload is listed first and matches calls
// with no type arguments; the explicit-`<Scope>` overloads below are the
// deprecated legacy/curried bridges (an explicit single `<Scope>` can't bind
// this overload's two inferred params, so resolution falls through to them).
// `__def` is added to the canonical return so the synthetic controller
// can recover the scope-factory return type via
// `typeof __authored.__def.scope` (slice 2 G4). The original `D` generic
// preserves the precise literal type of `def` (including the inferred
// `Ctx` + `SC`).
export function controller<Ctx, SC extends Record<string, unknown>, D extends ScopedControllerDef<Ctx, SC> = ScopedControllerDef<Ctx, SC>>(
  def: D,
): ControllerExport<Ctx & SC> & { __def: D }
// Legacy form `controller<Scope>(def)` — for controllers with NO `scope:`
// factory (only `on`/`mount`). Their scope is entirely template-declared, so
// the compiler-emitted `Scope` is already precise (no casts). A controller
export function controller<Scope>(def: ControllerDef<Scope>): ControllerExport<Scope>
// Implementation signature: `any` so it satisfies both overloads (the
// overloads are the real public contract). The `__def` field carries
// the original `def` (its precise literal type) so the synthetic
// controller can recover the scope-factory return type via
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function controller<D extends ControllerDef<any> = ControllerDef<object>>(
  def: D,
): ControllerExport<object> & { __def: D } {
  return {
    setup: (ctx) => resolveController(def as ControllerDef<object>, ctx),
    // `__def` carries the precise literal type of `def` (via the `D`
    // generic) so the synthetic controller can recover the
    // scope-factory return type through
    // `typeof __authored.__def extends { scope?: infer F }` (slice 2
    // G4). The `controller<Scope>(def)` legacy overload's
    // `ControllerDef<Scope>` is still the public type for callers
    // who read `__def` without the literal-narrowing pattern.
    __def: def,
  }
}
