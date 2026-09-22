// controller-resolver-types.ts — shared types for the controller +
// hook resolvers. Lives in its own file to break the cycle between
// `controller-resolver.ts` (which delegates to `resolveHook`) and
// `controller-hook-resolver.ts` (which needs `ResolverContext`).
//
// Slice 2 — SAIL-local `RouteSignals` (see ./route-signals for the
// decoupling rationale). Replaces the `@ship/deck-router` import.
import type { RouteSignals } from './route-signals'
import type { Host } from './host'

/**
 * Setup-time context passed to the resolver. Framework injects
 * (`route` / `navigate`) plus flat named providers (Host members).
 * There is no nested `clients` bag and no `ship` alias — providers
 * sit top-level on this object (and/or under optional `providers`,
 * which the resolver flattens).
 */
export type ResolverContext = {
  /**
   * Reactive URL proxy (`RouteSignals`). Plain `Record<string,string>` is
   * accepted for host tests that pass a flat map; production mounts supply
   * `RouteSignals`.
   */
  route?: RouteSignals | Record<string, string>
  navigate?: (path: string) => void
  /**
   * Optional bag of named providers — flattened onto scope the same way
   * top-level Host keys are. Prefer top-level `{ api, demo }` at the
   * mount call; this bag exists for hosts that already group them.
   */
  providers?: Host & Record<string, unknown>
  /**
   * Template-declared signals (and any other ambient bindings) that
   * are in scope by the time the controller resolver runs. State
   * signals (the writable subset) are identified by `stateKeys`.
   */
  scope?: Record<string, unknown>
  /**
   * Names of `scope` entries that are template `<state>` declarations.
   * Assignments through the writable proxy in `on`/`mount`/`unmount`
   * call `.set()` on these; assignments to other keys warn and are
   * dropped. Defaults to "every callable signal in scope" when
   * omitted, which is fine for unit tests but lossy for real hydrate
   * (since live envelope signals are also writable in shape).
   */
  stateKeys?: ReadonlySet<string>
  /**
   * The custom-element host (light DOM). Controllers use it for
   * imperative reach (focus an input, mount a third-party editor into
   * an opt-in shadow containment root). Not used for shared-state
   * publication — that's scope-homed via `useStore` / `provideContext`
   * (see `./store` / `./effect-scope`).
   */
  host?: Element
} & Host

/**
 * The flat bindings record returned by `resolveController`. Each entry
 * is something the template scope can consume directly:
 *
 *  - derived keys → callable memoized getter
 *  - on keys → handler functions (no state arg; mutator pre-bound)
 *  - use keys → namespaced sub-bindings (one record per hook)
 */
export type ResolvedBindings = Record<string, unknown>
