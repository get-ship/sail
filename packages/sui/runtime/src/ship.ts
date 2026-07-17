// ship.ts — the framework's host seam.
//
// The SUI runtime is host-agnostic: it names no Ship concepts. `ShipHost` is
// an empty interface that the app (deck / `@ship/sdk-client`) augments via
// declaration merging to the concrete `ShipSDK`:
//
//   declare module '@ship/sail-runtime' {
//     interface ShipHost extends ShipSDK {}
//   }
//
// `ViewScope.ship` / `ResolverContext.ship` are typed as `ShipHost`, so
// authoring stays fully typed while the runtime stays free of any host shape.
// The concrete `ShipSDK` + its data types live in `@ship/sdk-client`
// (`ship-sdk.ts`) — the framework deliberately knows nothing about them.

// eslint-disable-next-line @typescript-eslint/no-empty-object-type
export interface ShipHost {}

// `Clients` — the keyed, host-injected data/ops clients a view declares via
// `<client name="X" />`. Empty here; each provider augments its OWN key via
// declaration merging:
//
//   declare module '@ship/sail-runtime' {
//     interface Clients { ship: ShipSDK }
//   }
//
// This is the per-view, implementor-named generalization of the single
// `ShipHost` slot (multiple `<client>`s = multiple data sources). The runtime
// still names no host shape — providers supply the type, hosts supply the value
// at mount, and the compiler legalizes each declared name in `<data source>`.
// eslint-disable-next-line @typescript-eslint/no-empty-object-type
export interface Clients {}
