// sui-table engine — manual / server-mode seam (Backbone #1, Slice 4).
//
// Manual mode is a TRANSFORM SKIP — the engine NEVER calls a backend. Two rules
// live here:
//
//   1. Controlled-slice rule. A slice present in `options.state` is CONTROLLED:
//      the engine READS it (so header chevrons / page controls track it) and
//      NEVER writes it — a mutator fires the matching `on*Change` for the
//      consumer to write, then the consumer's write flows back in. A slice
//      ABSENT from `state` is INTERNAL: the engine owns the Signal (seeded by
//      `initialState`). `resolveSlice` picks the right Signal + write strategy.
//
//   2. Pipeline-prefix validation. Manual stages MUST form a prefix of the
//      pipeline order filter → sort → group → paginate (you cannot client-filter
//      a server-paginated page). Grouping may be skipped (it is often disabled);
//      any OTHER gap before a later manual stage is a foot-gun and WARNS. In
//      practice manualPagination implies manualSorting + manualFiltering.
//
// PURE TypeScript — imports `{ batch }` from `@ship/sui-runtime` ONLY. No DOM,
// no SDK, no network, no server-events.
//
// Authority: build-spec §"Manual / server mode (the seam — preserves Cardinal
// 4)" (per-feature manual* = transform skip; controlled-slice rule; pipeline-
// prefix rule with a WARN on a non-prefix combo), Slice 4.

import { batch, type Signal } from '@ship/sui-runtime'
import type { ReactiveTableState, TableState } from './types'

/**
 * Apply the consumer's CONTROLLED slices onto the reactive state: each slice
 * present in `controlled` REPLACES the engine's internal signal with the
 * consumer's signal, so every reader (row pipeline, column-api, footer) reads
 * the controlled value uniformly. Returns the set of controlled slice keys so
 * the mutators can SKIP writing them (firing `on*Change` instead). Mutating the
 * `state` object in place is intentional — the assembler owns it pre-return.
 */
export function applyControlledState(
  state: ReactiveTableState,
  controlled: Partial<ReactiveTableState> | undefined,
): Set<keyof TableState> {
  const keys = new Set<keyof TableState>()
  if (!controlled) return keys
  for (const key of Object.keys(controlled) as (keyof TableState)[]) {
    const sig = controlled[key]
    if (sig) {
      ;(state as Record<string, unknown>)[key] = sig
      keys.add(key)
    }
  }
  return keys
}

/** The controlled slices a consumer may hand the engine (engine-read-only). */
export interface ControlledState {
  sorting?: Signal<unknown>
  columnFilters?: Signal<unknown>
  globalFilter?: Signal<unknown>
  grouping?: Signal<unknown>
  expanded?: Signal<unknown>
  pagination?: Signal<unknown>
  rowSelection?: Signal<unknown>
  columnSizing?: Signal<unknown>
  columnOrder?: Signal<unknown>
  columnVisibility?: Signal<unknown>
  columnPinning?: Signal<unknown>
}

/** A resolved slice: the Signal to READ, and how to WRITE (or NOT) on a mutate. */
export interface ResolvedSlice<V> {
  /** The signal the engine reads (controlled signal if present, else internal). */
  read: Signal<V>
  /** Is this slice controlled (engine-read-only)? */
  controlled: boolean
  /** Apply a next value: internal → write the signal in batch; controlled → no
   *  write (the caller fires the on*Change instead). Returns whether it wrote. */
  write: (next: V) => boolean
}

/**
 * Resolve one state slice: if the consumer passed a controlled Signal for it,
 * the engine reads THAT signal and never writes it (a mutate fires the callback
 * for the consumer to write); otherwise the engine owns the `internal` signal.
 */
export function resolveSlice<V>(
  controlled: Signal<unknown> | undefined,
  internal: Signal<V>,
): ResolvedSlice<V> {
  if (controlled) {
    const read = controlled as Signal<V>
    return {
      read,
      controlled: true,
      write: () => false, // engine never writes a controlled slice
    }
  }
  return {
    read: internal,
    controlled: false,
    write: (next: V) => {
      batch(() => internal.set(next))
      return true
    },
  }
}

/** The manual-mode flags that gate the row pipeline. */
export interface ManualFlags {
  manualFiltering?: boolean
  manualSorting?: boolean
  manualGrouping?: boolean
  manualPagination?: boolean
}

/**
 * Validate the manual-flag combination forms a valid pipeline prefix
 * (filter → sort → group → paginate). Grouping may be skipped; any other gap
 * before a later manual stage is non-prefix and warns. Returns the warning
 * string when invalid (and `undefined` when valid) so the caller can emit it.
 */
export function manualPrefixWarning(flags: ManualFlags): string | undefined {
  // Required ordered stages (grouping is the one optional middle stage).
  const order: { name: string; manual: boolean; optional: boolean }[] = [
    { name: 'manualFiltering', manual: flags.manualFiltering === true, optional: false },
    { name: 'manualSorting', manual: flags.manualSorting === true, optional: false },
    { name: 'manualGrouping', manual: flags.manualGrouping === true, optional: true },
    { name: 'manualPagination', manual: flags.manualPagination === true, optional: false },
  ]

  // Walk the order: once a REQUIRED stage is not manual, a later manual stage is
  // a non-prefix violation. An optional (grouping) gap does not arm the break.
  let gapHit = false
  for (const stage of order) {
    if (stage.manual) {
      if (gapHit) {
        return `sui-table: manual mode flags must form a prefix of filter → sort → group → paginate — "${stage.name}" is set but an earlier stage is still client-side. (manualPagination implies manualSorting + manualFiltering.)`
      }
    } else if (!stage.optional) {
      gapHit = true
    }
  }
  return undefined
}

/** Emit the prefix warning to the console if the combo is non-prefix. */
export function warnOnNonPrefix(flags: ManualFlags): void {
  const msg = manualPrefixWarning(flags)
  if (msg) console.warn(msg)
}
