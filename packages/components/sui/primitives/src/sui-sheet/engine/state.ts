// sui-sheet engine — reactive cell model + state slices (the dirty-suffix core).
//
// The literal-path reactive store: one `Signal<CellData | undefined>` PER
// authored/touched cell so a write to `A1` notifies only `A1`'s subscribers —
// an unrelated `Z9` derived is a cache hit (the dirty-suffix discipline, proven
// by the recompute-counter spec). A workbook-identity change reconciles the per-
// cell signals against the new sheet. The interaction slices (selection /
// editing / activeSheetId) live here too; they are ORTHOGONAL to the cell store
// (presentational, not in the recalc chain).
//
// PURE TypeScript — imports `signal`/`derived`/`untrack` from `@ship/sui-runtime`.
//
// Authority: build-spec §"Signal graph" ("touching one slice recomputes only the
// suffix"), §"Backbone — createSheet()" (signals from the runtime ONLY),
// §Features #4/#10 (selection + editing slices), §States (loading/error reflect).

import { derived, isSignal, signal, type Derived, type Signal } from '@ship/sui-runtime'
import { fromA1, toA1 } from './a1'
import type {
  CellAddress,
  CellData,
  SelectionState,
  SheetDef,
  WorkbookDef,
} from './types'

/** Lift a raw value OR a live Signal/Derived to a read-tracked derived. */
export function liftWorkbook(
  src: WorkbookDef | Signal<WorkbookDef> | Derived<WorkbookDef>,
): Derived<WorkbookDef> {
  if (isSignal(src)) return derived(() => (src as Signal<WorkbookDef>)())
  if (typeof src === 'function') return derived(() => (src as Derived<WorkbookDef>)())
  return derived(() => src)
}

/** Reflect a consumer flag (plain value or live signal) — the engine REFLECTS
 *  loading/error; it never owns the source. */
export function reflect<V>(src: V | Signal<V> | undefined, fallback: V): Derived<V> {
  return derived<V>(() => {
    if (src === undefined) return fallback
    return isSignal(src) ? (src as Signal<V>)() : (src as V)
  })
}

/** The per-cell reactive store for the active sheet. Each cell maps to its own
 *  LOCAL-OVERRIDE signal (`undefined` = fall through to the authored workbook
 *  cell). A local `setCell` touches ONLY that cell's signal (the dirty-suffix
 *  guarantee — an unrelated cell's derived is a cache hit); a workbook-IDENTITY
 *  swap is picked up because the resolved-cell derived reads `activeSheet()`
 *  directly for the authored fallback — no standing effect, no reconcile pass. */
export interface CellStore {
  /** The memoized per-cell override signal (created on first read). */
  cellSignal(a1: string): Signal<CellData | undefined>
  /** The resolved authored+override CellData for a cell (read-tracked: tracks
   *  both the local override AND the active-sheet workbook identity). */
  read(a1: string): CellData | undefined
  /** Write one cell's local override (the only mutation entry). */
  write(a1: string, data: CellData | undefined): void
}

export function buildCellStore(activeSheet: Derived<SheetDef | undefined>): CellStore {
  const overrides = new Map<string, Signal<CellData | undefined>>()

  const ensure = (a1: string): Signal<CellData | undefined> => {
    let sig = overrides.get(a1)
    if (!sig) {
      sig = signal<CellData | undefined>(undefined)
      overrides.set(a1, sig)
    }
    return sig
  }

  return {
    cellSignal(a1: string): Signal<CellData | undefined> {
      return ensure(a1)
    },
    read(a1: string): CellData | undefined {
      // A local override wins; otherwise the authored workbook cell. Reading
      // `activeSheet()` here is what makes a workbook-signal swap reactive.
      const override = ensure(a1)()
      if (override !== undefined) return override
      return activeSheet()?.cells[a1]
    },
    write(a1: string, data: CellData | undefined): void {
      ensure(a1).set(data)
    },
  }
}

/** The interaction slices (orthogonal to the cell store). Seeded inert; the
 *  selection collapses to `A1` so the active cell is always defined. */
export interface InteractionState {
  selection: Signal<SelectionState>
  editing: Signal<{ cell: CellAddress; draft: string } | null>
  activeSheetId: Signal<string | null>
}

export function buildInteractionState(
  initialActiveSheetId: string | null,
  seed?: Partial<SelectionState>,
): InteractionState {
  return {
    selection: signal<SelectionState>(seed ? coerceSelection(seed) : emptySelection()),
    editing: signal<{ cell: CellAddress; draft: string } | null>(null),
    activeSheetId: signal<string | null>(initialActiveSheetId),
  }
}

const ORIGIN: CellAddress = { row: 0, col: 0 }

/** A collapsed 1×1 selection at the origin (`A1`). */
export function emptySelection(): SelectionState {
  return makeSelection(ORIGIN, ORIGIN, [])
}

function coerceSelection(seed: Partial<SelectionState>): SelectionState {
  const active = seed.active ?? ORIGIN
  const anchor = seed.anchor ?? active
  return makeSelection(active, anchor, seed.ranges ?? [])
}

/** Build a `SelectionState` (the methods are derived from the fields). */
export function makeSelection(
  active: CellAddress,
  anchor: CellAddress,
  ranges: SelectionState['ranges'],
): SelectionState {
  const mode: SelectionState['mode'] =
    ranges.length === 0
      ? 'cell'
      : ranges.length === 1 && ranges[0]!.width() === 1 && ranges[0]!.height() === 1
        ? 'cell'
        : 'range'
  return {
    active,
    anchor,
    ranges,
    mode,
    isEmpty(): boolean {
      return ranges.length === 0
    },
  }
}

/** A1 ⇄ address helpers re-exported so selection/cell-api share one source. */
export { fromA1, toA1 }
