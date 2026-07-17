// controller-resolver.ts — runtime backing for the typed controller contract.
//
// Takes a `ControllerDef` (the ONE setup function) and the resolver
// context, builds the ambient view scope (ship/clients/route/navigate
// + template-declared signals), and calls it. Whatever the function
// returns IS the flat bindings record `hydrate()` merges into the
// template scope — signals, `derived()` values, plain helpers, and
// event handlers alike. No section-walking, no writable-state proxy:
// the function closes over its own signals and mutates them directly,
// the single idiom `controller_state_set_call` (sail-lint) enforces.
//
// Lifecycle (`mount(fn)` / `onCleanup(fn)`) is ambient, not resolved
// here — those calls register on whatever `EffectScope` is current
// (see `./effect-scope`) while `def` runs; the host (`mount.ts`,
// `compose.ts`) wraps the call to `setup()` in `withScope(scope, …)`
// and later calls `scope.runMounts(hostElement)` once the DOM is
// connected.

import type { ControllerDef, ViewScope } from './controller'
import type {
  ResolverContext,
  ResolvedBindings,
} from './controller-resolver-types'

export type { ResolverContext, ResolvedBindings } from './controller-resolver-types'

/**
 * Resolve a controller def into the flat bindings shape `hydrate` can
 * merge into the template scope.
 *
 * @param def The controller's setup function.
 * @param ctx The setup-time context (`ship`, `route`, `navigate`, `scope`, `stateKeys`).
 * @returns A flat record of bindings keyed by name — the def's return, verbatim.
 */
export function resolveController(
  def: ControllerDef<object>,
  ctx: ResolverContext,
): ResolvedBindings {
  const baseScope: ViewScope<object> = {
    ship: ctx.ship,
    clients: ctx.clients ?? {},
    ...(ctx.clients ?? {}),
    route: ctx.route ?? {},
    navigate: ctx.navigate ?? ((_path: string) => {}),
    ...(ctx.scope ?? {}),
  } as ViewScope<object>

  return def(baseScope, ctx.host)
}

// ---------------------------------------------------------------------------
// `controller(setup)` — author-facing entry point
// ---------------------------------------------------------------------------

/**
 * The shape `controller(setup)` returns — a setup-shaped export the
 * daemon's hydrate path consumes verbatim. The daemon extracts `.setup`
 * and calls it with a `ResolverContext` (built from the hydrate flow);
 * `setup` runs the resolver and returns flat bindings the template
 * merges into scope.
 */
export interface ControllerExport<Scope = object> {
  /** Setup callback — invoked by hydrate with the resolver context. */
  setup: (ctx: ResolverContext) => ResolvedBindings
  /** Original def (the author's setup function); for tooling — the
   *  synthetic controller recovers its contributed scope TYPE from
   *  this function's return type via `typeof __authored.__def`. */
  __def: ControllerDef<Scope>
}

/**
 * Author-facing helper. Wrap a setup function with `controller(fn)` to
 * get a `{ setup, __def }` export the daemon's hydrate path can call
 * directly, with `Ctx`/`SC` inferred from `fn` — no explicit type
 * argument, no currying.
 *
 * @example
 * ```ts
 * <controller>
 * export default controller((ctx) => {
 *   const items = signal<Item[]>([])
 *   mount((el) => { ... })
 *   onCleanup(() => { ... })
 *   return { items, add: () => items.set([...items(), newItem()]) }
 * })
 * </controller>
 * ```
 */
// CANONICAL and ONLY form: `controller(fn)` with no explicit type argument.
// Both `Ctx` (from `fn`'s param annotation) and `SC` (from its return) are
// inferred → precise, cast-free handler state over `Ctx & SC`, single call,
// no currying. `__def` is added to the return so the synthetic controller
// can recover the setup function's return type via `typeof __authored.__def`
// (slice 2 G4, updated for the function-form def). Typing `fn` DIRECTLY as
// `(init: Ctx, host?: Element) => SC` (not a generic `D extends (...) => SC`)
// is load-bearing: it gives `Ctx`/`SC` real inference positions — `fn`'s
// param annotation drives `Ctx`, its return drives `SC` — so handler state
// over `Ctx & SC` is precise. A `D`-generic param erased those positions
// (nothing infers `Ctx`/`SC` from a bare `def: D`), so `Ctx` fell to
// `unknown`, the contravariant param check failed, and EVERY call silently
// widened — the framework-wide "factory signals untyped in handlers, must
// cast" bug this shape exists to prevent from coming back.
export function controller<Ctx, SC extends Record<string, unknown>>(
  fn: (init: ViewScope<Ctx>, host?: Element) => SC,
): ControllerExport<Ctx & SC> & { __def: (init: ViewScope<Ctx>, host?: Element) => SC } {
  return {
    setup: (ctx) => resolveController(fn as ControllerDef<object>, ctx),
    // `__def` carries the precise literal type of `fn` (via the generic
    // params above) so the synthetic controller can recover the setup
    // function's return type through `typeof __authored.__def extends
    // (...a: never[]) => infer R` (slice 2 G4).
    __def: fn,
  }
}
