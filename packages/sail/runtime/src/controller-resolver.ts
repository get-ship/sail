// controller-resolver.ts — runtime backing for the typed controller contract.
//
// Takes a `ControllerDef` (the ONE setup function) and the resolver
// context, builds the ambient route scope (flat providers + route/navigate
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

import type { ControllerDef, ControllerScope } from './controller'
import type {
  ResolverContext,
  ResolvedBindings,
} from './controller-resolver-types'

export type { ResolverContext, ResolvedBindings } from './controller-resolver-types'

/** Framework-owned keys on ResolverContext — not host providers. */
const RESERVED_CTX_KEYS = new Set([
  'route',
  'navigate',
  'providers',
  'scope',
  'stateKeys',
  'host',
  'props',
])

/**
 * Collect flat named providers from a resolver / mount context:
 * top-level non-reserved keys plus an optional `providers` bag.
 */
export function collectProviders(
  ctx: Record<string, unknown>,
): Record<string, unknown> {
  const out: Record<string, unknown> = {}
  const bag = ctx['providers']
  if (bag && typeof bag === 'object') {
    Object.assign(out, bag as Record<string, unknown>)
  }
  for (const [k, v] of Object.entries(ctx)) {
    if (RESERVED_CTX_KEYS.has(k)) continue
    if (v !== undefined) out[k] = v
  }
  return out
}

/**
 * Resolve a controller def into the flat bindings shape `hydrate` can
 * merge into the template scope.
 *
 * @param def The controller's setup function.
 * @param ctx The setup-time context (flat providers, `route`, `navigate`,
 *            `scope`, `stateKeys`).
 * @returns A flat record of bindings keyed by name — the def's return, verbatim.
 */
export function resolveController(
  def: ControllerDef<object>,
  ctx: ResolverContext,
): ResolvedBindings {
  // Flat host inject: named providers are top-level on ctx (and/or under
  // `providers`, which flattens). No nested `clients` bag, no `ship` alias.
  const providers = collectProviders(ctx as unknown as Record<string, unknown>)
  const baseScope: ControllerScope<object> = {
    ...providers,
    // `?? {}` is a WEAK guard, not a safety net. It is cast to `RouteSignals`
    // but has none of its members, so `route.params.id()` still throws — and
    // on the real mount path it never even applies: the compiled module writes
    // `$scope.route = route` unconditionally, so a present-but-undefined
    // `route` key inside `ctx.scope` spreads OVER this default below. It only
    // helps callers that pass no `ctx.scope` (unit tests, mostly).
    route: ctx.route ?? {},
    navigate: ctx.navigate ?? ((_path: string) => {}),
    ...(ctx.scope ?? {}),
  } as ControllerScope<object>

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
 */
export function controller<Ctx, SC extends Record<string, unknown>>(
  fn: (init: ControllerScope<Ctx>, host?: Element) => SC,
): ControllerExport<Ctx & SC> & { __def: (init: ControllerScope<Ctx>, host?: Element) => SC } {
  return {
    setup: (ctx) => resolveController(fn as ControllerDef<object>, ctx),
    __def: fn,
  }
}
