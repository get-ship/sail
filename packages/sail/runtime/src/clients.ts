// clients.ts — SAIL-local data/ops PORT.
//
// The keyed, host-injected data/ops clients a view consumes. This is the
// abstract port: the runtime declares the seam, providers declare the types,
// hosts supply the values at mount.
//
// THERE IS NO `<client>` DECLARATION. A .sail file declares nothing to get a
// client — USAGE IS THE DECLARATION. The chain is:
//
//   1. An app manifest (`app.manifest.jsonc`) maps a free name to the real
//      ESM import that provides it
//      (`emit::synthetic::AppManifest`, crates/sail/compiler/src/emit/synthetic/mod.rs).
//   2. The compiler scans the template for each manifest key it actually
//      references (`used_clients`) and imports only those — the per-view
//      resolver. A name the template never mentions costs nothing.
//   3. Every manifest key becomes a `ScopeCapability::client` typed through
//      THIS port as `Clients['<name>']` (`emit::synthetic::scope_capabilities`
//      → `emit::dts::ScopeCapability::client`), written as a member of the
//      generated `Scope`. Reading a name no manifest declares is a type error.
//   4. At mount the host supplies the concrete map —
//      `mount(target, payload, { clients: { api } })` — and `./mount` spreads
//      it into the view scope under both `clients.api` and bare `api`.
//
// The compiler names no client and no provider: the manifest is the only
// place a name is bound to an implementation.
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
// The runtime never names a host shape; the mount seam (`./mount`) supplies
// the concrete map from the OUTSIDE, and a host's concrete client map
// satisfies this interface structurally at that assignment site.
// eslint-disable-next-line @typescript-eslint/no-empty-object-type
export interface Clients {}
