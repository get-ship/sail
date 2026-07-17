// controller-resolver-types.ts — shared types for the controller +
// hook resolvers. Lives in its own file to break the cycle between
// `controller-resolver.ts` (which delegates to `resolveHook`) and
// `controller-hook-resolver.ts` (which needs `ResolverContext`).
//
// Slice 2 — SAIL-local `RouteSignals` (see ./route-signals for the
// decoupling rationale). Replaces the `@ship/deck-router` import.
import type { RouteSignals } from './route-signals'
import type { ShipHost, Clients } from './ship'

/**
 * Setup-time context passed to the resolver. Mirrors what `hydrate`
 * passes to a setup function, plus a `scope` field carrying any
 * signals already declared in the template (`<data>`, `<state>`)
 * by the time setup runs.
 */
export interface ResolverContext {
  ship: ShipHost
  /** Host-injected named data/ops clients (the `<client>`-declared map).
   *  Optional through the migration; spread into scope so bare `<name>.foo()`
   *  resolves. */
  clients?: Clients
  /**
   * Reactive URL proxy. Accepts `RouteSignals` (the new shape) or the
   * legacy `Record<string,string>` shape; the latter is retained as a
   * migration shim for callers that haven't moved over yet and emits a
   * `console.warn` from the attach runtime when used.
   */
  route?: RouteSignals | Record<string, string>
  navigate?: (path: string) => void
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
   * (since `<data>` signals are also writable in shape).
   */
  stateKeys?: ReadonlySet<string>
  /**
   * The custom-element host. Useful for controller code that
   * needs to reach its own shadow root (e.g. to imperatively
   * mount a third-party editor, or to focus an input). Not used
   * for shared-state publication — that's scope-homed now via
   * `useStore` / `provideContext` (see `./store` / `./effect-scope`).
   */
  host?: Element
}

/**
 * The flat bindings record returned by `resolveController`. Each entry
 * is something the template scope can consume directly:
 *
 *  - derived keys → callable memoized getter
 *  - on keys → handler functions (no state arg; mutator pre-bound)
 *  - use keys → namespaced sub-bindings (one record per hook)
 */
export type ResolvedBindings = Record<string, unknown>
