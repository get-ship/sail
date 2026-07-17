// sui-table engine — the one-direction derived row chain (Backbone #1, Slices 1b/2).
//
// Builds the row half of the signal graph from the build-spec:
//
//   data ─► coreRows ─►(global)─ preColumnFiltered ─►(column)─ filtered
//            ─►(sort)─ sorted ─►(group+expand)─ grouped(=prePagination) ─►(page)─ paginated
//
// Each stage is a `derived()` reading the prior stage + its own state slice, so
// touching one slice recomputes only the *suffix* (the suffix-only contract
// create-table.test.ts proves). This module owns `coreRows` + the sort + the
// minimal group stage; the filter stages live in `filtering.ts` and the page
// slice in `pagination.ts`, wired together by the assembler.
//
// PURE TypeScript — imports `derived` from `@ship/sail-runtime` and the typed
// comparators from Slice 1a. No DOM, no SDK.
//
// Authority: build-spec §"Signal graph (one-direction derived chain)",
// §Features #2 (numeric-aware comparator), §Features #6/#7 (group rows feed the
// selection cascade), Slices 1b/2.

import { derived, type Derived, type Signal } from '@ship/sail-runtime'
import { detectSortType, resolveComparator, type SortType } from './comparators'
import { buildAccessors, makeRow, type Accessor, type RowBindings } from './row-api'
import type { CellValue, ColumnDef, Row, SortState } from './types'

function isEmpty(v: unknown): boolean {
  return v == null || v === ''
}

export interface PipelineInputs<T> {
  /** Live data source — a raw array is lifted to a signal by the assembler. */
  data: Signal<T[]>
  /** Flat list of leaf columns. */
  columns: ColumnDef<T>[]
  getRowId: (row: T, index: number) => string
  /** Tree source — a row's child records (Feature #13 sub-rows). */
  getSubRows?: (row: T) => T[] | undefined
  /** Late-bound handles (selection + expansion) the rows route through. */
  bindings: RowBindings<T>
}

export interface CoreModel<T> {
  coreRows: Derived<Row<T>[]>
  accessors: Map<string, Accessor<T>>
}

/**
 * Build `coreRows` — each original lifted to a `Row<T>` keyed by `getRowId`.
 * When `getSubRows` is present each row carries its recursively-built child
 * rows (depth-tagged, parent-linked) so the grouping stage can flatten the tree
 * depth-first. With no `getSubRows` the rows are flat leaves (top-level only).
 */
export function buildCore<T>(inputs: PipelineInputs<T>): CoreModel<T> {
  const { data, columns, getRowId, getSubRows, bindings } = inputs
  const accessors = buildAccessors(columns)
  const coreRows = derived<Row<T>[]>(() => {
    if (!getSubRows) {
      // Flat rows: every row's sibling band is the full root list (posinset/setsize).
      const flat: Row<T>[] = []
      data().forEach((original, index) => {
        flat.push(
          makeRow(original, index, getRowId, accessors, bindings, {
            subRows: [],
            depth: 0,
            siblings: () => flat,
          }),
        )
      })
      return flat
    }
    const build = (records: T[], depth: number, parent?: Row<T>): Row<T>[] => {
      const rows: Row<T>[] = []
      records.forEach((original, index) => {
        const childRecords = getSubRows(original)
        // Siblings band = the rows array at THIS level (parent's subRows / the root
        // list). Late-bound so a row can reference the array still being filled.
        const row = makeRow(original, index, getRowId, accessors, bindings, {
          subRows: [],
          ...(parent ? { parent } : {}),
          depth,
          siblings: () => rows,
        })
        if (childRecords && childRecords.length > 0) {
          row.subRows.push(...build(childRecords, depth + 1, row))
        }
        rows.push(row)
      })
      return rows
    }
    return build(data(), 0)
  })
  return { coreRows, accessors }
}

/**
 * The sort stage over `filteredRows`. Empty sorting passes the input through
 * (array identity preserved). The multi-key stable sort resolves each active
 * column's typed comparator over its CURRENT values (numeric-aware → '10' after
 * '2'), then applies them in priority order; the first non-zero wins.
 *
 * Manual mode (`manual === true`) SKIPS the client sort entirely — the `data` IS
 * the server-sorted page — while `sorting` still feeds the header chevrons via
 * the column-api (the slice is read there, not here).
 */
export function buildSortStage<T>(
  filteredRows: Derived<Row<T>[]>,
  sorting: Signal<SortState[]>,
  manual = false,
): Derived<Row<T>[]> {
  return derived<Row<T>[]>(() => {
    const rows = filteredRows()
    if (manual) return rows // server already sorted — never client-sort the page
    const order = sorting()
    if (order.length === 0) return rows
    return sortRows(rows, order)
  })
}

/** Stable multi-key sort (see comparators.ts for the typed comparison). */
export function sortRows<T>(rows: Row<T>[], order: SortState[]): Row<T>[] {
  const cmps = new Map<string, ReturnType<typeof resolveComparator>>()
  for (const s of order) {
    const values: CellValue[] = rows.map((r) => r.getValue(s.id))
    const type: SortType = detectSortType(values)
    cmps.set(s.id, resolveComparator(type))
  }

  const copy = rows.slice()
  copy.sort((a, b) => {
    for (const s of order) {
      const av = a.getValue(s.id)
      const bv = b.getValue(s.id)
      const ae = isEmpty(av)
      const be = isEmpty(bv)
      if (ae || be) {
        if (ae && be) continue
        return ae ? 1 : -1
      }
      const cmp = cmps.get(s.id)!(av, bv)
      if (cmp !== 0) return s.desc ? -cmp : cmp
    }
    return 0
  })
  return copy
}
