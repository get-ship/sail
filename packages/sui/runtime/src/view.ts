// view() — wraps a setup function as the default export of a compiled
// SUI view module.
//
// The sui-compiler emits every `.sui` view as:
//
//   import { view, … } from "@ship/sui-runtime"
//   export default view((ctx) => { …; return { …$scope, __mount(c) {…} } })
//
// `attach()` (in @ship/sui-dom) reads `module.default.setup(ctx)` to seed
// scope and then walks SSR DOM using the sibling `bindings` manifest.
// This factory is the contract glue between those two halves.

import type { ShipHost, Clients } from './ship'
// Slice 2 — SUI-local structural `RouteSignals` (replaces the
// `@ship/deck/router` import). The deck-side `RouteSignals` is
// structurally compatible (both import the same `Signal` from
// `@ship/sui-runtime`); the deck `route` factory's return value
// satisfies this structural type at the assignment site.
import type { RouteSignals } from './route-signals'

export interface ViewContext {
  ship: ShipHost
  clients: Clients
  route: RouteSignals
  navigate: (path: string) => void
}

export type ViewSetup<Bindings = Record<string, unknown>> = (
  // Slice 2 — the runtime's view() accepts an untyped context so the
  // synthetic emitter can pass through the mount-time props seam
  // (component-mode mounts carry a `ctx.props` field the view-shaped
  // setup reads via `ctx.props` before calling the scope factory).
  // The browser-mount path still constructs a real `ViewContext` and
  // casts back inside `mount.ts` where it has the typed values.
  ctx: any,
) => Bindings

export interface ViewDef<Bindings = Record<string, unknown>> {
  setup: ViewSetup<Bindings>
}

export function view<Bindings = Record<string, unknown>>(
  setup: ViewSetup<Bindings>,
): ViewDef<Bindings> {
  return { setup }
}
