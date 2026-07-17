// sui-table engine — the ORTHOGONAL column lane (Backbone #1, Slice 3a).
//
// Features #8-#11 (sizing/resize, ordering, visibility, pinning) live OUTSIDE
// the row pipeline. This lane reads the column state slices (`columnSizing`,
// `columnSizingInfo`, `columnOrder`, `columnVisibility`, `columnPinning`) and
// derives the presentational surface: `visibleLeafColumns` (order → visibility
// → pin-bucket), `leftLeafColumns`/`rightLeafColumns`, the resolved sizing map,
// `getTotalSize`, and the per-column `getStart`/`getSize` offsets. Because no
// row-pipeline derived reads these slices, a `setColumnSizing` write recomputes
// ONLY the column lane — the orthogonality proof columns.test.ts asserts.
//
// One sizing source: `resolvedSizing` is the single resolved px map; the header
// reads it via `getSize()` and the body reads the SAME number via the
// `--col-<id>-size` custom-property value. They cannot drift.
//
// PURE TypeScript — imports `derived` from `@ship/sail-runtime` ONLY. No DOM, no
// SDK.
//
// Authority: build-spec §Features #8 (resize onChange/onEnd, flexible px/'auto'/
// grow, getTotalSize, min/max clamp + autofit, ONE `--col-<id>-size` source),
// #9 (ordering, partial orders), #10 (visibility), #11 (pinning: getStart from
// the resolved sizing map, getIsPinned/getPinnedIndex/getIsLastColumn),
// §"Signal graph" ("ORTHOGONAL (presentational, NOT in the row chain)"),
// Slice 3a.

import { derived, type Derived, type Signal } from '@ship/sail-runtime'
import type {
  ColumnDef,
  ColumnPinningState,
  ColumnSizingState,
  VisibilityState,
} from './types'

/** Default px width when a column declares no explicit `size`. */
const DEFAULT_SIZE = 150
/** Resolved px for a 'auto'-sized column (content-fit is a view concern; the
 *  engine resolves a deterministic numeric placeholder so the sizing map is a
 *  pure number map and `getTotalSize` is well-defined). */
const AUTO_SIZE = 150
/** Resolved px contributed per `grow` unit before viewport distribution (the
 *  view spreads remaining width across grow columns; the engine seeds a numeric
 *  base so the map stays numeric and `> 0`). */
const GROW_BASE = 150

export interface ColumnLaneState {
  columnSizing: Signal<ColumnSizingState>
  columnOrder: Signal<string[]>
  columnVisibility: Signal<VisibilityState>
  columnPinning: Signal<ColumnPinningState>
}

export interface ColumnLane<T> {
  /** Visible leaves: order → visibility → pin-bucket (left ⧺ center ⧺ right). */
  visibleLeafColumns: Derived<ColumnDef<T>[]>
  leftLeafColumns: Derived<ColumnDef<T>[]>
  rightLeafColumns: Derived<ColumnDef<T>[]>
  /** The SINGLE resolved px map every sizing read derives from. */
  resolvedSizing: Derived<ColumnSizingState>
  getTotalSize: Derived<number>
  // Per-column reactive reads (track the lane when called in a derived/template).
  getSize: (id: string) => number
  getStart: (id: string, pos: 'left' | 'right') => number
  getIsVisible: (id: string) => boolean
  getIsPinned: (id: string) => 'left' | 'right' | false
  getPinnedIndex: (id: string) => number
  getIsLastColumn: (id: string, pos: 'left' | 'right' | false) => boolean
  // Size custom-property source (one var feeds header + body).
  getColumnSizeVar: (id: string) => string
  getColumnSizeVarValue: (id: string) => string
}

/** Resolve a column def's declared `size`/`grow` to a numeric px base. */
function baseSize<T>(def: ColumnDef<T>): number {
  if (typeof def.size === 'number') return def.size
  if (def.size === 'auto') return AUTO_SIZE
  if (typeof def.grow === 'number' && def.grow > 0) return GROW_BASE * def.grow
  return DEFAULT_SIZE
}

/** Clamp a px size to a column's min/max bounds (both optional). */
function clampSize<T>(def: ColumnDef<T> | undefined, px: number): number {
  let v = px
  if (def?.minSize != null && v < def.minSize) v = def.minSize
  if (def?.maxSize != null && v > def.maxSize) v = def.maxSize
  return v
}

/**
 * Build the orthogonal column lane over the leaf defs + the column state
 * slices. The deriveds read ONLY the column slices, so writing one (e.g.
 * `columnSizing`) recomputes the lane and nothing in the row pipeline.
 */
export function buildColumnLane<T>(
  columns: ColumnDef<T>[],
  state: ColumnLaneState,
): ColumnLane<T> {
  const byId = new Map(columns.map((c) => [c.id, c]))
  const defOrder = columns.map((c) => c.id)

  // ── Resolved sizing map: declared base, overridden by columnSizing, clamped ─
  const resolvedSizing = derived<ColumnSizingState>(() => {
    const overrides = state.columnSizing()
    const map: ColumnSizingState = {}
    for (const def of columns) {
      const override = overrides[def.id]
      const px = override != null ? override : baseSize(def)
      map[def.id] = clampSize(def, px)
    }
    return map
  })

  // ── Ordering: columnOrder ids first (in order), absent ids keep def position ─
  const orderedIds = derived<string[]>(() => {
    const order = state.columnOrder()
    if (order.length === 0) return defOrder
    const seen = new Set<string>()
    const out: string[] = []
    for (const id of order) {
      if (byId.has(id) && !seen.has(id)) {
        out.push(id)
        seen.add(id)
      }
    }
    for (const id of defOrder) {
      if (!seen.has(id)) out.push(id)
    }
    return out
  })

  // ── Visibility: drop ids set to false ──────────────────────────────────────
  const isVisible = (id: string): boolean => state.columnVisibility()[id] !== false

  // ── Pin buckets: left ⧺ center ⧺ right over the visible, ordered leaves ─────
  const buckets = derived<{
    left: ColumnDef<T>[]
    center: ColumnDef<T>[]
    right: ColumnDef<T>[]
  }>(() => {
    const pinning = state.columnPinning()
    const leftSet = pinning.left
    const rightSet = pinning.right
    const left: ColumnDef<T>[] = []
    const center: ColumnDef<T>[] = []
    const right: ColumnDef<T>[] = []
    // Center keeps the resolved order; left/right keep PIN order (not column order).
    const visibleOrdered = orderedIds()
      .map((id) => byId.get(id)!)
      .filter((def) => isVisible(def.id))
    const visibleIds = new Set(visibleOrdered.map((d) => d.id))
    for (const def of visibleOrdered) {
      if (leftSet.includes(def.id) || rightSet.includes(def.id)) continue
      center.push(def)
    }
    for (const id of leftSet) {
      if (visibleIds.has(id)) left.push(byId.get(id)!)
    }
    for (const id of rightSet) {
      if (visibleIds.has(id)) right.push(byId.get(id)!)
    }
    return { left, center, right }
  })

  const leftLeafColumns = derived<ColumnDef<T>[]>(() => buckets().left)
  const rightLeafColumns = derived<ColumnDef<T>[]>(() => buckets().right)
  const visibleLeafColumns = derived<ColumnDef<T>[]>(() => {
    const b = buckets()
    return [...b.left, ...b.center, ...b.right]
  })

  // ── Total size + per-column reads — all from the resolved sizing map ────────
  const getTotalSize = derived<number>(() => {
    const sizing = resolvedSizing()
    let total = 0
    for (const def of visibleLeafColumns()) total += sizing[def.id] ?? 0
    return total
  })

  const getSize = (id: string): number => resolvedSizing()[id] ?? 0

  const getStart = (id: string, pos: 'left' | 'right'): number => {
    const pinning = state.columnPinning()
    const sizing = resolvedSizing()
    const lane = pos === 'left' ? pinning.left : pinning.right
    let offset = 0
    for (const pinned of lane) {
      if (pinned === id) return offset
      offset += sizing[pinned] ?? 0
    }
    return offset
  }

  const getIsVisible = (id: string): boolean => isVisible(id)

  const getIsPinned = (id: string): 'left' | 'right' | false => {
    const pinning = state.columnPinning()
    if (pinning.left.includes(id)) return 'left'
    if (pinning.right.includes(id)) return 'right'
    return false
  }

  const getPinnedIndex = (id: string): number => {
    const pinning = state.columnPinning()
    const li = pinning.left.indexOf(id)
    if (li !== -1) return li
    const ri = pinning.right.indexOf(id)
    if (ri !== -1) return ri
    return -1
  }

  const getIsLastColumn = (id: string, pos: 'left' | 'right' | false): boolean => {
    const pinning = state.columnPinning()
    if (pos === 'left') return pinning.left[pinning.left.length - 1] === id
    if (pos === 'right') return pinning.right[pinning.right.length - 1] === id
    const visible = visibleLeafColumns()
    return visible[visible.length - 1]?.id === id
  }

  const getColumnSizeVar = (id: string): string => `--col-${id}-size`
  const getColumnSizeVarValue = (id: string): string => `${getSize(id)}px`

  return {
    visibleLeafColumns,
    leftLeafColumns,
    rightLeafColumns,
    resolvedSizing,
    getTotalSize,
    getSize,
    getStart,
    getIsVisible,
    getIsPinned,
    getPinnedIndex,
    getIsLastColumn,
    getColumnSizeVar,
    getColumnSizeVarValue,
  }
}

/** Clamp helper exposed for the assembler's `setColumnSizing` write path so the
 *  committed map is clamped at write time too (the resolved map clamps reads;
 *  this keeps the stored slice honest for the `onStateChange` snapshot). */
export function clampSizingPatch<T>(
  columns: ColumnDef<T>[],
  patch: ColumnSizingState,
): ColumnSizingState {
  const byId = new Map(columns.map((c) => [c.id, c]))
  const out: ColumnSizingState = {}
  for (const [id, px] of Object.entries(patch)) {
    out[id] = clampSize(byId.get(id), px)
  }
  return out
}
