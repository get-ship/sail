// sail-table engine — Row<T> factory (Backbone #1, Slice 1b core + Slice 2 selection).
//
// Wraps an original record into the `Row<T>` API shape keyed by the REQUIRED
// `getRowId`. Slice 1b landed the load-bearing core (`id`/`index`/`original`/
// `getValue`). Slice 2 wires the SELECTION members (`getIsSelected`/
// `getIsSomeSelected`/`getCanSelect`/`toggleSelected`) and a minimal GROUP row
// (synthetic parent over leaf children with aggregate-free grouping value) so
// the selection group-cascade + tri-state contract is real. Grouping proper
// (aggregation, footers, deep trees) lands in Slice 3b.
//
// Rows read the selection API through a late-bound `bindings` handle: the
// assembler builds selection AFTER the row pipeline (selection scopes the
// displayed order), then populates `bindings.selection`. Row methods are
// invoked from `on:*`/template — after construction — so the late binding is
// resolved by call time.
//
// PURE TypeScript — no runtime imports (reactivity lives in the pipeline
// computeds that build the row arrays + the selection deriveds).
//
// Authority: build-spec §"Authoritative Row / Cell / Column / Header API",
// §Features #7 (selection cascade + tri-state), Slice 2.

import type { CellBinding } from './cell-api'
import type { Cell, CellValue, ColumnDef, Row } from './types'

/** How a column extracts its value from an original record. */
export type Accessor<T> = (row: T) => CellValue

/** Late-bound handles the row methods route through (selection in Slice 2,
 *  expansion in Slice 3b, cells in Slice 4). */
export interface RowBindings<T> {
  selection?: {
    isSelected: (id: string) => boolean
    canSelectId: (id: string) => boolean
    toggleRowSelected: (
      id: string,
      v?: boolean,
      opts?: { range?: boolean; additive?: boolean },
    ) => void
  }
  expansion?: {
    isExpanded: (id: string) => boolean
    toggleExpanded: (id: string, v?: boolean) => void
  }
  /** Cells — `getVisibleCells(row)` maps visibleLeafColumns to Cell<T> (Slice 4). */
  cells?: CellBinding<T>
}

/** The visible-cell list for a row through the late-bound cell handle. */
function visibleCellsOf<T>(row: Row<T>, bindings: RowBindings<T>): Cell<T>[] {
  return bindings.cells?.getVisibleCells(row) ?? []
}

/**
 * Build the accessor map for a flat column list. Only accessor columns
 * (`accessorKey`) and display columns carrying an `accessorFn` are valued; a
 * column with neither yields no entry (its `getValue` returns undefined).
 */
export function buildAccessors<T>(columns: ColumnDef<T>[]): Map<string, Accessor<T>> {
  const map = new Map<string, Accessor<T>>()
  for (const def of columns) {
    if ('accessorKey' in def && def.accessorKey != null) {
      const key = def.accessorKey
      map.set(def.id, (row: T) => (row as Record<string, unknown>)[key] as CellValue)
    } else if ('accessorFn' in def && typeof def.accessorFn === 'function') {
      const fn = def.accessorFn
      map.set(def.id, (row: T) => fn(row))
    }
  }
  return map
}

/** Tree shape passed when a row carries sub-rows (Feature #13 expansion). */
export interface RowTree<T> {
  /** Pre-built child rows (recursion builds these depth-first). */
  subRows: Row<T>[]
  /** The parent row (undefined at the top level). */
  parent?: Row<T>
  /** The depth in the tree (top-level = 0). */
  depth: number
  /** The row's sibling band (its parent's subRows, or the root list at depth 0).
   *  Late-bound (returns a function) so depth-0 rows can reference the root array
   *  that is still being built. Drives `getPosInSet`/`getSetSize`. */
  siblings?: () => Row<T>[]
}

/**
 * The leaf-selection tri-state + cascade a BRANCH carries (a regular tree row
 * with subRows, or a synthetic group row): selection reflects the SELECTABLE
 * leaf descendants — fully selected when every selectable leaf is selected, tri-state
 * when some-but-not-all are; toggling cascades to every selectable leaf. One source
 * of truth shared by `makeRow` (tree branch) and `makeGroupRow` (Feature #7).
 */
export function branchSelection<T>(
  leafRows: () => Row<T>[],
  bindings: RowBindings<T>,
): {
  getIsSelected: () => boolean
  getIsSomeSelected: () => boolean
  getCanSelect: () => boolean
  toggleSelected: (v?: boolean) => void
} {
  const selectable = (): Row<T>[] =>
    leafRows().filter((l) => bindings.selection?.canSelectId(l.id) ?? true)
  const allSelected = (): boolean => {
    const sel = selectable()
    return sel.length > 0 && sel.every((l) => l.getIsSelected())
  }
  return {
    getIsSelected: allSelected,
    getIsSomeSelected: () => {
      const sel = selectable()
      return sel.some((l) => l.getIsSelected()) && !sel.every((l) => l.getIsSelected())
    },
    getCanSelect: () => selectable().length > 0,
    toggleSelected: (v?: boolean) => {
      const target = v ?? !allSelected()
      for (const l of selectable()) l.toggleSelected(target)
    },
  }
}

/**
 * Wrap one original record into a `Row<T>`. `index` is the row's position in
 * the core (pre-transform) order. `accessors` is the shared column→value map.
 * `bindings.selection`/`bindings.expansion` (late-bound) route the selection +
 * expansion members; until populated they report the "feature off" defaults.
 *
 * `tree` carries the sub-row structure for tree data (`getSubRows`): when a row
 * has children it `getCanExpand()`s and `getLeafRows()`/`getParentRows()` walk
 * the structure. A leaf row (no `tree`) keeps the flat-row defaults.
 */
export function makeRow<T>(
  original: T,
  index: number,
  getRowId: (row: T, index: number) => string,
  accessors: Map<string, Accessor<T>>,
  bindings: RowBindings<T>,
  tree?: RowTree<T>,
): Row<T> {
  const id = getRowId(original, index)
  const subRows = tree?.subRows ?? []
  const depth = tree?.depth ?? 0

  const getValue = (colId: string): CellValue => {
    const accessor = accessors.get(colId)
    return accessor ? accessor(original) : undefined
  }

  const collectLeaves = (): Row<T>[] => {
    if (subRows.length === 0) return [row]
    const out: Row<T>[] = []
    for (const child of subRows) out.push(...child.getLeafRows())
    return out
  }

  // Siblings band — parent's subRows, or the root list (late-bound at depth 0).
  const siblingsOf = (): Row<T>[] => {
    const explicit = tree?.siblings?.()
    if (explicit) return explicit
    const parent = tree?.parent
    return parent ? parent.subRows : [row]
  }
  const getPosInSet = (): number => {
    const i = siblingsOf().findIndex((s) => s.id === id)
    return i < 0 ? 1 : i + 1
  }
  const getSetSize = (): number => {
    const n = siblingsOf().length
    return n === 0 ? 1 : n
  }

  // A BRANCH (has subRows) carries the leaf tri-state + cascade; a LEAF keeps the
  // flat single-row selection contract (no tri-state, direct toggle). `subRows` is
  // mutated AFTER construction (children push in), so `isBranch` is read lazily at
  // call time — never captured at build time.
  const branchSel = branchSelection<T>(collectLeaves, bindings)
  const isBranch = (): boolean => subRows.length > 0

  const row: Row<T> = {
    id,
    index,
    depth,
    original,
    getValue,
    getVisibleCells: () => visibleCellsOf(row, bindings),
    subRows,
    getCanExpand: () => subRows.length > 0,
    getIsExpanded: () => bindings.expansion?.isExpanded(id) ?? false,
    toggleExpanded: (v?: boolean) => bindings.expansion?.toggleExpanded(id, v),
    getParentRow: () => tree?.parent,
    getParentRows: () => {
      const chain: Row<T>[] = []
      let p = tree?.parent
      while (p) {
        chain.unshift(p)
        p = p.getParentRow()
      }
      return chain
    },
    getPosInSet,
    getSetSize,
    getLeafRows: collectLeaves,
    // A branch reflects its leaf descendants (tri-state + cascade); a leaf selects
    // itself directly through the late-bound selection handle.
    getIsSelected: () =>
      isBranch() ? branchSel.getIsSelected() : bindings.selection?.isSelected(id) ?? false,
    getIsSomeSelected: () => (isBranch() ? branchSel.getIsSomeSelected() : false),
    getCanSelect: () =>
      isBranch() ? branchSel.getCanSelect() : bindings.selection?.canSelectId(id) ?? true,
    toggleSelected: (v?: boolean, opts?: { range?: boolean }) =>
      isBranch()
        ? branchSel.toggleSelected(v)
        : bindings.selection?.toggleRowSelected(id, v, opts),
    getIsPinned: () => false,
    pin: () => {},
    getIsGrouped: () => false,
    getGroupingValue: (colId: string) => getValue(colId),
  }
  return row
}

/** Inputs for a synthetic group row (Slice 3b — aggregation + nesting). */
export interface GroupRowInputs<T> {
  groupColId: string
  groupingValue: unknown
  /** Direct children: leaf rows OR nested group rows (multi-level grouping). */
  subRows: Row<T>[]
  /** The flat leaf descendants (the aggregation domain). */
  leafRows: Row<T>[]
  index: number
  depth: number
  bindings: RowBindings<T>
  /** Per-column aggregate over the leaf rows: colId → aggregated CellValue. */
  aggregate: (colId: string) => CellValue
  /** The group's sibling band (the sibling group rows at this level). Late-bound
   *  so the level builder can reference the array still being assembled. Drives
   *  `getPosInSet`/`getSetSize` for `aria-posinset`/`aria-setsize`. */
  siblings?: () => Row<T>[]
}

/**
 * Build a synthetic group row keyed by `groupingValue` for `groupColId`. The
 * group row has no original record; `getValue(colId)` returns the grouping
 * value for the group column and the column AGGREGATE (sum/min/…) for an
 * aggregating column. Selection cascades to the leaf descendants (all →
 * selected; some → tri-state). Expansion is keyed by the group row's id through
 * the late-bound expansion handle so a collapsed group hides its children.
 */
export function makeGroupRow<T>(inputs: GroupRowInputs<T>): Row<T> {
  const { groupColId, groupingValue, subRows, leafRows, index, depth, bindings, aggregate } =
    inputs
  const id = `group:${depth}:${groupColId}:${String(groupingValue)}`
  // The leaf tri-state + cascade — one source of truth shared with tree branches.
  const sel = branchSelection<T>(() => leafRows, bindings)

  // Siblings band — the sibling group rows at this level (drives posinset/setsize).
  const siblingsOf = (): Row<T>[] => inputs.siblings?.() ?? [group]
  const getPosInSet = (): number => {
    const i = siblingsOf().findIndex((s) => s.id === id)
    return i < 0 ? 1 : i + 1
  }
  const getSetSize = (): number => {
    const n = siblingsOf().length
    return n === 0 ? 1 : n
  }

  const group: Row<T> = {
    id,
    index,
    depth,
    original: undefined as unknown as T,
    getValue: (colId: string) =>
      colId === groupColId ? (groupingValue as CellValue) : aggregate(colId),
    getVisibleCells: () => visibleCellsOf(group, bindings),
    subRows,
    getCanExpand: () => subRows.length > 0,
    getIsExpanded: () => bindings.expansion?.isExpanded(id) ?? true,
    toggleExpanded: (v?: boolean) => bindings.expansion?.toggleExpanded(id, v),
    getParentRow: () => undefined,
    getParentRows: () => [],
    getPosInSet,
    getSetSize,
    getLeafRows: () => leafRows,
    getIsSelected: sel.getIsSelected,
    getIsSomeSelected: sel.getIsSomeSelected,
    getCanSelect: sel.getCanSelect,
    toggleSelected: (v?: boolean) => sel.toggleSelected(v),
    getIsPinned: () => false,
    pin: () => {},
    getIsGrouped: () => true,
    getGroupingValue: (colId: string) =>
      colId === groupColId ? groupingValue : undefined,
  }
  return group
}
