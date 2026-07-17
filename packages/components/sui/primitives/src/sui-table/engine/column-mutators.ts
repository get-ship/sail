// sui-table engine — the column-lane mutators (Backbone #1, Slice 3a).
//
// The write half of the orthogonal column lane: `setColumnSizing` (onChange /
// onEnd resize modes + min/max clamp), `setColumnOrder`, `setColumnVisibility`,
// `setColumnPinning`, `pinColumn`. Every mutator writes its slice inside
// `batch()` then fires the matching `on*Change` callback — except an `onEnd`
// resize, which writes the size (so the live drag is reflected) but DEFERS the
// callback until a later commit (`onChange`) write fires it once.
//
// PURE TypeScript — imports `batch` from `@ship/sail-runtime` only. No DOM, no
// SDK.
//
// Authority: build-spec §Features #8 (resize mode onChange/onEnd: "default
// onChange (live), with an onEnd (commit) option", min/max clamp), #9
// (ordering), #10 (visibility), #11 (pinning), §"Signal graph" (mutation
// discipline: write in batch + fire on*Change), Slice 3a.

import { batch } from '@ship/sail-runtime'
import { clampSizingPatch } from './columns'
import type {
  ColumnDef,
  ColumnPinningState,
  ColumnSizingState,
  ReactiveTableState,
  TableState,
  VisibilityState,
} from './types'

export interface ColumnMutatorDeps<T> {
  columns: ColumnDef<T>[]
  state: ReactiveTableState
  snapshot: () => TableState
  callbacks: {
    onColumnSizingChange?: (s: ColumnSizingState) => void
    onColumnOrderChange?: (o: string[]) => void
    onColumnVisibilityChange?: (v: VisibilityState) => void
    onColumnPinningChange?: (p: ColumnPinningState) => void
    onStateChange?: (s: TableState) => void
  }
}

export interface ColumnMutators {
  setColumnSizing: (
    u: ColumnSizingState | ((prev: ColumnSizingState) => ColumnSizingState),
    opts?: { mode?: 'onChange' | 'onEnd' },
  ) => void
  setColumnOrder: (o: string[]) => void
  setColumnVisibility: (
    u: VisibilityState | ((prev: VisibilityState) => VisibilityState),
  ) => void
  setColumnPinning: (
    u: ColumnPinningState | ((prev: ColumnPinningState) => ColumnPinningState),
  ) => void
  toggleVisibility: (id: string, v?: boolean) => void
  pinColumn: (id: string, pos: 'left' | 'right' | false) => void
}

/** Build the column-lane mutators bound to the reactive state + callbacks. */
export function buildColumnMutators<T>(deps: ColumnMutatorDeps<T>): ColumnMutators {
  const { columns, state, snapshot, callbacks } = deps

  // ── Sizing (resize) ─────────────────────────────────────────────────────────
  function setColumnSizing(
    u: ColumnSizingState | ((prev: ColumnSizingState) => ColumnSizingState),
    opts?: { mode?: 'onChange' | 'onEnd' },
  ): void {
    const next = typeof u === 'function' ? u(state.columnSizing()) : u
    const merged = clampSizingPatch(columns, { ...state.columnSizing(), ...next })
    // Both modes WRITE the size (the live drag must reflect onEnd too); only the
    // CALLBACK is deferred for onEnd until a subsequent onChange commit.
    batch(() => state.columnSizing.set(merged))
    if (opts?.mode === 'onEnd') return
    callbacks.onColumnSizingChange?.(state.columnSizing())
    callbacks.onStateChange?.(snapshot())
  }

  // ── Order ─────────────────────────────────────────────────────────────────
  function setColumnOrder(o: string[]): void {
    batch(() => state.columnOrder.set(o))
    callbacks.onColumnOrderChange?.(state.columnOrder())
    callbacks.onStateChange?.(snapshot())
  }

  // ── Visibility ──────────────────────────────────────────────────────────────
  function setColumnVisibility(
    u: VisibilityState | ((prev: VisibilityState) => VisibilityState),
  ): void {
    const next = typeof u === 'function' ? u(state.columnVisibility()) : u
    batch(() => state.columnVisibility.set({ ...state.columnVisibility(), ...next }))
    callbacks.onColumnVisibilityChange?.(state.columnVisibility())
    callbacks.onStateChange?.(snapshot())
  }
  const toggleVisibility = (id: string, v?: boolean): void => {
    const cur = state.columnVisibility()[id] !== false
    setColumnVisibility({ [id]: v ?? !cur })
  }

  // ── Pinning ─────────────────────────────────────────────────────────────────
  function setColumnPinning(
    u: ColumnPinningState | ((prev: ColumnPinningState) => ColumnPinningState),
  ): void {
    const next = typeof u === 'function' ? u(state.columnPinning()) : u
    batch(() => state.columnPinning.set(next))
    callbacks.onColumnPinningChange?.(state.columnPinning())
    callbacks.onStateChange?.(snapshot())
  }
  function pinColumn(id: string, pos: 'left' | 'right' | false): void {
    const cur = state.columnPinning()
    // Remove from BOTH buckets first (re-pin moves it across the boundary).
    const left = cur.left.filter((x) => x !== id)
    const right = cur.right.filter((x) => x !== id)
    if (pos === 'left') left.push(id)
    else if (pos === 'right') right.push(id)
    setColumnPinning({ left, right })
  }

  return {
    setColumnSizing,
    setColumnOrder,
    setColumnVisibility,
    setColumnPinning,
    toggleVisibility,
    pinColumn,
  }
}
