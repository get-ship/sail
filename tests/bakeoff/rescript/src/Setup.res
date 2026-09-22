// Setup — the controller bridge the compiled .sail emits import as their
// virtual `*.sail.controller.ts` file (vitest alias → lib/js/Setup.js).
//
// Default export `{ setup(ctx) → scopeObj }`; the returned object is merged
// into the render's `$scope`. Two fixtures share this module because the
// bakeoff aliases BOTH virtual controller files here, so setup dispatches
// on the host tag carried by `ctx.host`:
//
//   • bakeoff-counter → `{ clicks, add }`   (c1 DOM test)
//   • anything else   → full krausest keyed-table scope:
//     rows/selected are runtime signals (this repo's TS graph, bound in
//     SailSignal.res); every handler mutates the plain-JS Table.res state
//     machine and pushes ONE fresh array identity into `rows`, so the
//     keyed reconciler rewrites only rows whose label actually changed.
//
// Unlike the grain/rust tracks there is NO wasm boundary: handlers call
// bsc-compiled JS directly. The resync mirror logic mirrors
// sail-rust/wasm-bridge.ts resync(): row objects are reused when (id,
// packed) are unchanged.

type row = {"id": int, "label": string}

// Mirror of the last synced row objects + their (id, packed) keys, so
// unchanged rows keep referential identity across ops.
type mirror = {
  mutable rows: array<row>,
  mutable ids: array<int>,
  mutable packed: array<int>,
}

let m: mirror = {rows: [], ids: [], packed: []}

let rowsSignal = SailSignal.make([]: array<row>)
let selected = SailSignal.make(0)

/** One bulk sync per op: rebuild a fresh array identity, reusing unchanged
 *  row objects; then push it through the runtime signal graph. */
let resync = () => {
  let n = Table.len()
  let oldId = m.ids
  let oldPacked = m.packed
  let oldLen = Array.length(oldId)
  // Safe fill default: m.rows may be empty on the very first sync.
  let dummy: row = {"id": 0, "label": ""}
  let out = Array.make(n, dummy)
  let fresh = Array.make(n, dummy)
  let freshId = Array.make(n, 0)
  let freshPacked = Array.make(n, 0)
  for i in 0 to n - 1 {
    let id = Table.keyAt(i)
    let packed = Table.valAt(i)
    let row =
      if i < oldLen && oldId[i] == id && oldPacked[i] == packed {
        m.rows[i]
      } else {
        {"id": id, "label": Table.labelOf(packed)}
      }
    out[i] = row
    fresh[i] = row
    freshId[i] = id
    freshPacked[i] = packed
  }
  m.rows = fresh
  m.ids = freshId
  m.packed = freshPacked
  SailSignal.set(rowsSignal, out)
}

let removeById = id => {
  let n = Table.len()
  let rec find = i =>
    if i >= n {
      false
    } else if Table.keyAt(i) == id {
      ignore(Table.removeAt(i))
      true
    } else {
      find(i + 1)
    }
  if find(0) {
    resync()
  }
}

let swapGuarded = () => {
  if Table.len() > 998 {
    Table.swaprows(1, 998) |> ignore
    resync()
  }
}

// ── Krausest scope (list track) ──────────────────────────────────────────────

// The two fixture scopes have intentionally different shapes; the emit
// consumes them dynamically, so erase to a loose JS object type.
external loosen: 'a => Js.Dict.t<unknown> = "%identity"

let tableScope = () => loosen({
    "rows": () => SailSignal.read(rowsSignal),
    "selected": () => SailSignal.read(selected),
    "run": () => {
      Table.run(1000) |> ignore
      resync()
    },
    "runlots": () => {
      Table.run(10000) |> ignore
      resync()
    },
    "add": () => {
      Table.add(1000) |> ignore
      resync()
    },
    "update": () => {
      Table.update() |> ignore
      resync()
    },
    "clear": () => {
      Table.clear() |> ignore
      resync()
    },
    "swaprows": swapGuarded,
    "select": id => {
      Table.select(id) |> ignore
      SailSignal.set(selected, id)
    },
    "remove": removeById,
})

// ── Counter scope (c1 test) ──────────────────────────────────────────────────

let counterScope = () => {
  let clicks = SailSignal.make(0)
  loosen({
    "clicks": () => SailSignal.read(clicks),
    "add": () => SailSignal.set(clicks, SailSignal.read(clicks) + 1),
  })
}

%%raw(
  "export function hostTag(ctx) {
  const h = ctx == null ? null : ctx.host
  return h != null && h.localName != null ? h.localName : ''
}
"
)
external hostTag: 'ctx => string = "hostTag"
// The emit checks `typeof setup.setup === "function"` before invoking.
let setup = ctx => hostTag(ctx) === "bakeoff-counter" ? counterScope() : tableScope()


%%raw("export default { setup }")
