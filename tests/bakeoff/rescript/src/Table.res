// Krausest keyed-table state machine — ReScript port of the reference
// Grain implementation (bakeoff-grain worktree, table_flat.gr).
//
// Unlike the Grain/Rust tracks there is NO wasm and NO ABI here: bsc
// compiles this ordinary mutable-record / flat-array code straight to
// plain JS. Wasm owns ids/order/seed/mutated/selection in the other
// tracks; here the same state simply lives in a module-level record.
//
// State (identical semantics to table_flat.gr):
//   packed = seed * 32 + appendCount   (appendCount capped at 31; low 5 bits)
//   labels are materialized host-side from (seed, appendCount) via labelFor
//   seed chain: s' = s * 48271 mod 100003

type t = {
  mutable ids: array<int>,
  mutable vals: array<int>,
  mutable n: int,
  mutable cap: int,
  mutable nextId: int,
  mutable seed: int,
  mutable sel: int,
}

let initCap = 16384

let t: t = {
  ids: Array.make(initCap, 0),
  vals: Array.make(initCap, 0),
  n: 0,
  cap: initCap,
  nextId: 1,
  seed: 7919,
  sel: 0,
}

let grow = () => {
  let nextCap = t.cap * 2
  let ids2 = Array.make(nextCap, 0)
  let vals2 = Array.make(nextCap, 0)
  for i in 0 to t.n - 1 {
    ids2[i] = t.ids[i]
    vals2[i] = t.vals[i]
  }
  t.ids = ids2
  t.vals = vals2
  t.cap = nextCap
}

let insertOne = () => {
  if t.n >= t.cap {
    grow()
  }
  let id = t.nextId
  let s = mod(t.seed * 48271, 100003)
  t.seed = s
  t.ids[t.n] = id
  t.vals[t.n] = s * 32
  t.nextId = id + 1
  t.n = t.n + 1
}

// ── Ops (krausest vocabulary) ────────────────────────────────────────────────

/** #add — APPEND `count` rows. */
let add = count => {
  let target = t.n + count
  while t.n < target {
    insertOne()
  }
  t.n
}

/** #run — official semantics REPLACES all rows; ids keep counting up. */
let replaceAll = count => {
  t.n = 0
  add(count)
}

/** Every-10th row gains one ' !!!' — pack an append COUNT in the low 5 bits.
 *  ReScript 11 lets are immutable: loop counters ride a mutable cursor
 *  record, exactly like the reference Grain implementation. */
type cursor = {mutable i: int}

let cur: cursor = {i: 0}

let partialUpdate = () => {
  cur.i = 0
  while cur.i < t.n {
    let v = t.vals[cur.i]
    let count = mod(v, 32) + 1
    let countCapped = count > 31 ? 31 : count
    t.vals[cur.i] = v - mod(v, 32) + countCapped
    cur.i = cur.i + 10
  }
  t.n
}

let selectRow = id => {
  t.sel = id
  t.sel
}

let swapRows = (a, b) => {
  let ia = t.ids[a]
  t.ids[a] = t.ids[b]
  t.ids[b] = ia
  let va = t.vals[a]
  t.vals[a] = t.vals[b]
  t.vals[b] = va
  t.n
}

let removeAt = i => {
  for j in i + 1 to t.n - 1 {
    t.ids[j - 1] = t.ids[j]
    t.vals[j - 1] = t.vals[j]
  }
  t.n = t.n - 1
  t.n
}

let clearRows = () => {
  t.n = 0
  t.sel = 0
  t.n
}

// ── Read surface (mirrors the bridge ctl exports) ────────────────────────────

let len = () => t.n
let keyAt = i => t.ids[i]
let valAt = i => t.vals[i]
let selectedId = () => t.sel

/** Official krausest vocabulary: adjective colour noun + ' !!!' × bangs. */
let labelFor = (seed, bangs) => {
  let adj = switch mod(seed, 10) {
  | 0 => "pretty"
  | 1 => "large"
  | 2 => "big"
  | 3 => "small"
  | 4 => "tall"
  | 5 => "short"
  | 6 => "long"
  | 7 => "handsome"
  | 8 => "plain"
  | _ => "quaint"
  }
  let colour = switch mod(seed / 7, 9) {
  | 0 => "red"
  | 1 => "yellow"
  | 2 => "blue"
  | 3 => "green"
  | 4 => "pink"
  | 5 => "brown"
  | 6 => "purple"
  | 7 => "brown"
  | _ => "white"
  }
  let noun = switch mod(seed / 13, 9) {
  | 0 => "table"
  | 1 => "chair"
  | 2 => "house"
  | 3 => "bbq"
  | 4 => "desk"
  | 5 => "car"
  | 6 => "pony"
  | 7 => "cookie"
  | _ => "sandwich"
  }
  let rec bangs_ = n => n <= 0 ? "" : " !!!" ++ bangs_(n - 1)
  adj ++ " " ++ colour ++ " " ++ noun ++ bangs_(bangs)
}

let labelOf = packed => labelFor(packed / 32, mod(packed, 32))

// ReScript 11 auto-exports top-level lets; krausest-vocabulary aliases:
let run = replaceAll
let update = partialUpdate
let select = selectRow
let swaprows = swapRows
let clear = clearRows
