// sui-table engine — grouping + expansion mutators (Backbone #1, Slice 3b).
//
// The `set*`/`toggle*` writers for the grouping + expansion slices, factored out
// of the assembler. Every writer mutates its Signal inside `batch()` then fires
// the matching `on*Change` + `onStateChange` (Cardinal 2 & 3). Expansion is the
// late-bound handle the rows read through (`getIsExpanded`/`toggleExpanded`).
//
// PURE TypeScript — imports `batch` from `@ship/sail-runtime` ONLY.
//
// Authority: build-spec §Features #5/#13 (setGrouping/toggleGrouping +
// setExpanded/toggleExpanded/toggleAllExpanded; `expanded: true` = expand-all),
// §"Mutation discipline", Slice 3b.

import { batch } from '@ship/sail-runtime'
import type { ExpandedState, ReactiveTableState, TableState } from './types'

export interface RowMutatorCallbacks {
  onGroupingChange?: (g: string[]) => void
  onExpandedChange?: (e: ExpandedState) => void
  onStateChange?: (s: TableState) => void
}

export interface RowMutators {
  setGrouping: (g: string[] | ((prev: string[]) => string[])) => void
  toggleGrouping: (id: string) => void
  setExpanded: (u: ExpandedState | ((prev: ExpandedState) => ExpandedState)) => void
  toggleExpanded: (id: string, v?: boolean) => void
  toggleAllExpanded: (v?: boolean) => void
  /** The late-bound expansion handle the rows bind to. */
  isExpanded: (id: string) => boolean
}

export function buildRowMutators(
  state: ReactiveTableState,
  snapshot: () => TableState,
  cb: RowMutatorCallbacks,
): RowMutators {
  const fireGrouping = (): void => {
    cb.onGroupingChange?.(state.grouping())
    cb.onStateChange?.(snapshot())
  }
  const fireExpanded = (): void => {
    cb.onExpandedChange?.(state.expanded())
    cb.onStateChange?.(snapshot())
  }

  function setGrouping(g: string[] | ((prev: string[]) => string[])): void {
    batch(() => {
      const next = typeof g === 'function' ? g(state.grouping()) : g
      state.grouping.set(next)
    })
    fireGrouping()
  }
  function toggleGrouping(id: string): void {
    setGrouping((prev) => (prev.includes(id) ? prev.filter((g) => g !== id) : [...prev, id]))
  }

  function setExpanded(u: ExpandedState | ((prev: ExpandedState) => ExpandedState)): void {
    batch(() => {
      const next = typeof u === 'function' ? u(state.expanded()) : u
      state.expanded.set(next)
    })
    fireExpanded()
  }
  function toggleExpanded(id: string, v?: boolean): void {
    setExpanded((prev) => {
      if (prev === true) {
        // Expand-all → opening a specific id is a no-op; closing one demotes to
        // an explicit map (id off) — but only `false` collapses from expand-all.
        if (v === false) return { [id]: false }
        return true
      }
      const cur = prev[id] === true
      const next = v ?? !cur
      return { ...prev, [id]: next }
    })
  }
  function toggleAllExpanded(v?: boolean): void {
    setExpanded((prev) => {
      const on = v ?? prev !== true
      return on ? true : {}
    })
  }

  const isExpanded = (id: string): boolean => {
    const e = state.expanded()
    return e === true ? true : e[id] === true
  }

  return {
    setGrouping,
    toggleGrouping,
    setExpanded,
    toggleExpanded,
    toggleAllExpanded,
    isExpanded,
  }
}
