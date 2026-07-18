// ship.ts — the framework's host seam.
//
// The SAIL runtime is host-agnostic: it names no Ship concepts. `ShipHost` is
// an empty interface that the app (deck / `@ship/sdk-client`) augments via
// declaration merging to the concrete `ShipSDK`:
//
//   declare module '@ship/sail-runtime' {
//     interface ShipHost extends ShipSDK {}
//   }
//
// `ResolverContext.ship` / `MountContext.ship` are typed as `ShipHost`, so the
// host-provision seam stays typed while the runtime stays free of any host
// shape. The concrete `ShipSDK` + its data types live in `@ship/sdk-client`
// (`ship-sdk.ts`) — the framework deliberately knows nothing about them.
//
// `ship` is a SEAM member only. It is NOT a member of `ControllerScope`: the
// generic controller scope names no host product (#105). A Ship host still
// provides the value — it rides `clients.ship`, which the mount path spreads
// into scope — and Ship-side authoring types it by augmenting `Clients`, not
// by the framework injecting a `ship` key into every controller.

// eslint-disable-next-line @typescript-eslint/no-empty-object-type
export interface ShipHost {}

// `Clients` moved to `./clients` — it is the abstract data PORT, not part of
// the Ship host seam. Re-exported here so the seam files that need both keep
// one import site.
export type { Clients } from './clients'
