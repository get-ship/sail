// host.ts — flat host-inject PORT.
//
// Named providers the host puts on controller / mount context. Empty by
// design; each provider augments its OWN key via declaration merging:
//
//   declare module '@ship/sail-runtime' {
//     interface Host { api: Api; demo: Demo }
//   }
//
// THERE IS NO nested bag (`clients`) and NO framework `ship` alias. Usage
// is the declaration:
//
//   1. An app manifest maps a free name to an ESM import.
//   2. The compiler scans the template for each manifest key it references
//      and types it as `Host['<name>']` on the generated `Scope`.
//   3. At mount the host supplies the values flat —
//      `mount(target, payload, { api, demo, route, navigate })` — or via
//      `{ providers: { api, demo } }` which flattens onto the same surface.
//   4. Controllers and templates read bare names: `ctx.api`, `demo.addTodo()`.
//
// Framework injects `route` + `navigate` stay separate (deck host_context);
// they are not Host members. The runtime names no host product shape.

// eslint-disable-next-line @typescript-eslint/no-empty-object-type
export interface Host {}
