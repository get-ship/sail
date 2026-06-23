// sui-table engine — reactive state seed (Backbone #1, Slice 1b).
//
// Builds the full `ReactiveTableState`: one `Signal` per `TableState` slice,
// seeded from `initialState` over a baseline of inert defaults. The assembler
// owns these signals; each `set*`/`toggle*` writes its slice inside `batch()`.
// A slice present in a (future) controlled `state` option is read-only to the
// engine — that wiring lands with manual mode; this seed is the uncontrolled
// baseline every slice reads from now.
//
// PURE TypeScript — imports `signal` from `@ship/sui-runtime` only.
//
// Authority: build-spec §"API sketch" (`initialState` UNCONTROLLED seed,
// `state` CONTROLLED), §"Signal graph" (each slice is a Signal), §"Features"
// (the slice → feature mapping), Slice 1b (`state.sorting` is `SortState[]`).

import { signal } from '@ship/sui-runtime'
import type { ReactiveTableState, TableState } from './types'

/** The inert baseline — every feature off, every collection empty. */
export function defaultTableState(): TableState {
  return {
    sorting: [],
    columnFilters: [],
    globalFilter: '',
    grouping: [],
    expanded: {},
    // pageSize 0 = no pagination slice: the full row set flows to the
    // virtualizer, which is the reference grid's default scaling strategy.
    // Pagination is opt-in (`setPageSize(n)` / `initialState.pagination`).
    pagination: { pageIndex: 0, pageSize: 0 },
    rowSelection: { mode: 'include', ids: new Set<string>() },
    selectionAnchor: null,
    columnSizing: {},
    columnSizingInfo: {
      columnId: null,
      startSize: 0,
      deltaOffset: 0,
      isResizing: false,
    },
    columnOrder: [],
    columnVisibility: {},
    columnPinning: { left: [], right: [] },
    rowPinning: { top: [], bottom: [] },
    editing: null,
  }
}

/**
 * Build the reactive state: every `TableState` slice lifted to its own
 * `Signal`, seeded from `initial` (a partial override of the inert baseline).
 */
export function buildState(initial?: Partial<TableState>): ReactiveTableState {
  const seed = { ...defaultTableState(), ...initial }
  // Per-slice Signal construction. Typed via the keyed mapped type so each
  // signal's value type matches its slice exactly.
  const state = {} as ReactiveTableState
  for (const key of Object.keys(seed) as (keyof TableState)[]) {
    // The cast is sound: seed[key] is TableState[key]; the target slot is
    // Signal<TableState[key]>. TS cannot follow the per-key correlation
    // across a dynamic loop, so we assert the matched pair.
    ;(state as Record<string, unknown>)[key] = signal(seed[key])
  }
  return state
}
