// SailSignal — ReScript bindings for @ship/sail-runtime's reactive core.
//
// The bakeoff law: ReScript BINDS the runtime signal graph, it never
// reimplements one. Every binding here resolves to the same TS source the
// other tracks use (`packages/sail/runtime/src/signal.ts` via the
// `@ship/sail-runtime/internal` specifier, aliased by the vitest config).
//
// A sail signal is a CALLABLE GETTER paired with a `.set` writer. bsc types
// the callable as an abstract `t<'v>`; reading needs to *call* the value,
// which external syntax can't express — hence the tiny `%%raw`/`%raw` read
// shim below (3 lines, zero boxing: it compiles to `s => s()`).
//
// GOTCHA pinned during this track: `external read: ... = "read"` binds a
// GLOBAL `read`, not this module's raw export — cross-module callers then
// emit a bare undefined `read(...)`. Binding the shim as a `%raw` LET keeps
// it a real exported symbol other modules import.

type t<'v>

@module("@ship/sail-runtime/internal")
external make: 'v => t<'v> = "signal"

let read: t<'v> => 'v = %raw("function read(s) {
  return s()
}")

@module("@ship/sail-runtime/internal")
external derived: (unit => 'v) => t<'v> = "derived"

@send
external set: (t<'v>, 'v) => unit = "set"

@module("@ship/sail-runtime/internal")
external effect: (unit => unit) => (unit => unit) = "effect"

@module("@ship/sail-runtime/internal")
external batch: (unit => 'a) => 'a = "batch"

@module("@ship/sail-runtime/internal")
external untrack: (unit => 'a) => 'a = "untrack"
