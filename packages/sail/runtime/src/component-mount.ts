// component-mount.ts — the mount-form of `component()`.
//
// `component(setup)` — the single-argument (function) overload — wraps a setup
// function as the default export of a compiled *tagless* SAIL component: a
// component that is MOUNTED into a host node rather than REGISTERED as a custom
// element. The sail-compiler emits every tagless `.sail` (a deck page, the
// synthetic SSR render) as:
//
//   import { component, … } from "@ship/sail-runtime"
//   export default component((ctx) => { …; return { …$scope, __mount(c) {…} } })
//
// `attach()` (in @ship/sail-dom) reads `module.default.setup(ctx)` to seed scope
// and then walks SSR DOM. This is the contract glue between those two halves.
//
// The two-argument `component(tag, def)` overload — in component-factory.ts —
// REGISTERS a custom element instead. Mount vs. register is the call shape; a
// tag (a hyphenated name) is the only thing that turns a component into a
// registered custom element. With no tag, a component is mounted, not defined.

import type { ShipHost, Clients } from './ship'
// SAIL-local structural `RouteSignals` (the deck-side `RouteSignals` is
// structurally compatible — both import the same `Signal` from
// `@ship/sail-runtime`; deck's `route` factory return satisfies this type at the
// assignment site). SAIL names no host: route/navigate arrive via the seam.
import type { RouteSignals } from './route-signals'

export interface MountContext {
  ship: ShipHost
  clients: Clients
  route: RouteSignals
  navigate: (path: string) => void
}

export type MountSetup<Bindings = Record<string, unknown>> = (
  // The mount-form accepts an untyped context so the synthetic emitter can pass
  // through the mount-time props seam (component mounts carry a `ctx.props`
  // field the setup reads before calling the scope factory). The browser-mount
  // path constructs a real `MountContext` and casts back inside `mount.ts`
  // where it has the typed values.
  ctx: any,
) => Bindings

export interface MountModule<Bindings = Record<string, unknown>> {
  setup: MountSetup<Bindings>
}
