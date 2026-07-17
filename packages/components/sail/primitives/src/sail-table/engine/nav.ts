// sail-table engine — keyboard navigation model (Backbone #1, Slice 4, Feature #14).
//
// Owns the focus MODEL (`nav.active` = { rowId, columnId }); the view owns DOM
// focus (roving-tabindex / aria-activedescendant derived from `nav.active`). The
// engine NEVER touches the DOM. `nav.onKeydown(e)` returns true when it handled
// the key (the view then preventDefaults) and false when it did not (the browser
// keeps the key — e.g. Tab EXITS the grid per the ARIA grid pattern).
//
// Keymap (navMode 'row' | 'cell'):
//   ArrowUp/Down            move rows (over the displayed order)
//   ArrowLeft/Right         cell mode: move columns; tree row: expand/collapse when
//                           getCanExpand, else move
//   Home / End              first / last column, same row
//   Ctrl+Home / Ctrl+End    grid corners (top-left / bottom-right)
//   PageUp / PageDown       page the viewport (a viewport-page of rows)
//   Tab / Shift+Tab         EXIT → false (browser tabs away)
//   Space                   toggle selection at the active cell → true
//   Enter / F2              enter edit on the active cell (if editable)
//   Escape                  cancel edit
//   typeahead(ch)           jump to the next row whose active-column value starts with ch
//   nav is SUSPENDED during edit (the input owns the keys) — onKeydown returns false.
//
// Virtualization seam: moving focus to an off-window row calls `scrollToIndex`
// BEFORE focus so the view can reassert DOM focus post-recycle from `nav.active`.
//
// PURE TypeScript — imports `{ signal, derived }` from `@ship/sail-runtime` ONLY.
// No DOM, no SDK.
//
// Authority: build-spec §Features #14 ("Keyboard-nav hooks"), §"API sketch"
// (the `nav` surface), Slice 4.

import { derived, signal, type Derived } from '@ship/sail-runtime'
import { buildTreeNav } from './nav-tree'
import type { ColumnApi, Row } from './types'

export type NavMode = 'row' | 'cell'
export type NavDir =
  | 'up'
  | 'down'
  | 'left'
  | 'right'
  | 'home'
  | 'end'
  | 'pageup'
  | 'pagedown'
  | 'ctrl-home'
  | 'ctrl-end'

export interface NavActive {
  rowId: string | null
  columnId: string | null
}

export interface NavApi {
  mode: Derived<NavMode>
  active: Derived<NavActive>
  move: (dir: NavDir) => void
  focusCell: (rowId: string, columnId: string) => void
  activate: () => void
  selectAtActive: (opts?: { range?: boolean; additive?: boolean }) => void
  typeahead: (ch: string) => void
  onKeydown: (e: KeyboardEvent) => boolean
}

export interface NavInputs<T> {
  navMode: NavMode
  /** The displayed (post sort/filter/page) rows — the nav grid. */
  rows: Derived<Row<T>[]>
  /** The visible leaf columns — the nav grid's columns. */
  visibleLeafColumns: Derived<ColumnApi<T>[]>
  /** Suspend nav while a cell edits (the input owns the keys). */
  isEditingActive: () => boolean
  /** Toggle selection at a row (Space). */
  toggleRowSelected: (id: string, v?: boolean, opts?: { range?: boolean; additive?: boolean }) => void
  /** Enter edit on the active cell (Enter / F2). */
  startEditing: (rowId: string, columnId: string) => void
  cancelEdit: () => void
  /** Rows-per-viewport-page (PageUp/PageDown). Defaults to 10. */
  pageSize?: () => number
  /** Off-window scroll seam — called BEFORE moving focus to a row's index. */
  scrollToIndex?: (index: number) => void
}

/** Number of rows a PageUp/PageDown jumps when no explicit page size is given. */
const DEFAULT_PAGE = 10

export function buildNav<T>(inputs: NavInputs<T>): NavApi {
  const {
    rows,
    visibleLeafColumns,
    isEditingActive,
    toggleRowSelected,
    startEditing,
    cancelEdit,
  } = inputs
  const modeSig = signal<NavMode>(inputs.navMode)
  const rowIdSig = signal<string | null>(null)
  const colIdSig = signal<string | null>(null)
  const pageOf = inputs.pageSize ?? (() => DEFAULT_PAGE)

  const mode = derived<NavMode>(() => modeSig())
  const active = derived<NavActive>(() => ({ rowId: rowIdSig(), columnId: colIdSig() }))

  const rowIndex = (id: string | null): number =>
    id == null ? -1 : rows().findIndex((r) => r.id === id)
  const colIndex = (id: string | null): number =>
    id == null ? -1 : visibleLeafColumns().findIndex((c) => c.id === id)

  /** Move focus to a row index, scrolling it into view first (the seam). */
  function focusRowAt(index: number, columnId: string | null): void {
    const all = rows()
    if (all.length === 0) return
    const clamped = Math.max(0, Math.min(index, all.length - 1))
    inputs.scrollToIndex?.(clamped)
    rowIdSig.set(all[clamped]!.id)
    if (columnId != null) colIdSig.set(columnId)
  }

  function focusColAt(index: number): void {
    const cols = visibleLeafColumns()
    if (cols.length === 0) return
    const clamped = Math.max(0, Math.min(index, cols.length - 1))
    colIdSig.set(cols[clamped]!.id)
  }

  function focusCell(rowId: string, columnId: string): void {
    rowIdSig.set(rowId)
    colIdSig.set(columnId)
  }

  /** The active row, if resolvable in the displayed set. */
  const activeRow = (): Row<T> | undefined => {
    const i = rowIndex(rowIdSig())
    return i >= 0 ? rows()[i] : undefined
  }

  function move(dir: NavDir): void {
    const ri = rowIndex(rowIdSig())
    const ci = colIndex(colIdSig())
    switch (dir) {
      case 'up':
        focusRowAt((ri < 0 ? 0 : ri) - 1, colIdSig())
        return
      case 'down':
        focusRowAt((ri < 0 ? 0 : ri) + 1, colIdSig())
        return
      case 'left':
        focusColAt((ci < 0 ? 0 : ci) - 1)
        return
      case 'right':
        focusColAt((ci < 0 ? 0 : ci) + 1)
        return
      case 'home':
        focusColAt(0)
        return
      case 'end':
        focusColAt(visibleLeafColumns().length - 1)
        return
      case 'pageup':
        focusRowAt((ri < 0 ? 0 : ri) - pageOf(), colIdSig())
        return
      case 'pagedown':
        focusRowAt((ri < 0 ? 0 : ri) + pageOf(), colIdSig())
        return
      case 'ctrl-home':
        focusRowAt(0, visibleLeafColumns()[0]?.id ?? null)
        return
      case 'ctrl-end':
        focusRowAt(rows().length - 1, visibleLeafColumns().at(-1)?.id ?? null)
        return
    }
  }

  function activate(): void {
    const r = rowIdSig()
    const c = colIdSig()
    if (r != null && c != null) startEditing(r, c)
  }

  function selectAtActive(opts?: { range?: boolean; additive?: boolean }): void {
    const r = rowIdSig()
    if (r != null) toggleRowSelected(r, undefined, opts)
  }

  /** First-letter jump: from the row AFTER the active one (wrapping), find the
   *  next row whose active-column value starts with `ch` (case-insensitive). */
  function typeahead(ch: string): void {
    const col = colIdSig()
    if (col == null || ch.length === 0) return
    const needle = ch.toLowerCase()
    const all = rows()
    const start = rowIndex(rowIdSig())
    for (let step = 1; step <= all.length; step++) {
      const i = (start + step) % all.length
      const v = String(all[i]!.getValue(col) ?? '').toLowerCase()
      if (v.startsWith(needle)) {
        focusRowAt(i, col)
        return
      }
    }
  }

  /** Move row focus to a row by id (no-op when the id is not displayed). */
  function focusRowId(id: string): void {
    const i = rowIndex(id)
    if (i >= 0) focusRowAt(i, colIdSig())
  }

  // ARIA-tree row nav (navMode:'row' Left/Right + Home/End) — see `nav-tree.ts`.
  const tree = buildTreeNav<T>({
    activeRow,
    rows,
    move,
    focusRowAt: (i) => focusRowAt(i, colIdSig()),
    focusRowId,
  })

  function onKeydown(e: KeyboardEvent): boolean {
    // Suspended during edit — the input owns the keys (Escape still cancels).
    if (isEditingActive()) {
      if (e.key === 'Escape') {
        cancelEdit()
        return true
      }
      return false
    }
    const ctrl = e.ctrlKey || e.metaKey
    switch (e.key) {
      case 'Tab':
        return false // EXIT the grid (roving-tabindex hands off to the browser)
      case 'ArrowUp':
        move('up')
        return true
      case 'ArrowDown':
        move('down')
        return true
      case 'ArrowRight':
        return modeSig() === 'row' ? tree.arrow('right') : (move('right'), true)
      case 'ArrowLeft':
        return modeSig() === 'row' ? tree.arrow('left') : (move('left'), true)
      case 'Home':
        // Row mode + tree row: first VISIBLE row (ARIA-tree). Else column ends.
        if (!(ctrl === false && modeSig() === 'row' && tree.rowEnd('home'))) {
          move(ctrl ? 'ctrl-home' : 'home')
        }
        return true
      case 'End':
        if (!(ctrl === false && modeSig() === 'row' && tree.rowEnd('end'))) {
          move(ctrl ? 'ctrl-end' : 'end')
        }
        return true
      case 'PageUp':
        move('pageup')
        return true
      case 'PageDown':
        move('pagedown')
        return true
      case ' ':
      case 'Spacebar':
        selectAtActive()
        return true
      case 'Enter':
      case 'F2':
        activate()
        return true
      case 'Escape':
        cancelEdit()
        return true
      default:
        return false // unmapped — not handled
    }
  }

  return { mode, active, move, focusCell, activate, selectAtActive, typeahead, onKeydown }
}
