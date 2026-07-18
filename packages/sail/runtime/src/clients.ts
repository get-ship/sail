// clients.ts — SAIL-local data/ops PORT.
//
// The keyed, host-injected data/ops clients a view declares via
// `<client name="X" />`. This is the abstract port: the runtime declares the
// seam, providers declare the types, hosts supply the values at mount.
//
// Same decoupling move as `./route-signals` (which replaced the
// `@ship/deck-router` import): the type a controller's scope member is
// annotated with is owned by SAIL, not by a host product. `Clients` used to
// live in `./ship` alongside `ShipHost`, which pinned the generic controller
// scope to the Ship host module. Nothing about the shape changed in the move —
// only who owns it.
//
// Empty here by design. Each provider augments its OWN key via declaration
// merging:
//
//   declare module '@ship/sail-runtime' {
//     interface Clients { ship: ShipSDK }
//   }
//
// The runtime never names a host shape; the compiler legalizes each declared
// name in `<data source>`; the mount seam (`./mount`) supplies the concrete
// map from the OUTSIDE, and a host's concrete client map satisfies this
// interface structurally at that assignment site.
// eslint-disable-next-line @typescript-eslint/no-empty-object-type
export interface Clients {}
