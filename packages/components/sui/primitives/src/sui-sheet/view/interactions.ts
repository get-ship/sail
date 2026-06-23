// sui-sheet view — interaction wiring (the surface ↔ engine bridge).
//
// The controller's `mount()` installs ONE set of handlers (pointer / keyboard /
// dblclick / copy / paste / fill-handle drag) that translate DOM events into
// engine mutator calls and re-emit the advertised events. Kept out of the
// controller so the controller stays the thin scope+lifecycle orchestrator and
// each stays under the file cap.
//
// All mutations run inside the on:* handlers (Cardinal 3); the engine stays
// pure. The engine reaches the view via `prop:sheet`; here we read it back from
// the scope signal.

import { bindNativeEvent } from '@ship/sui-runtime'
import { rangeToTsv, tsvToEdits } from '../engine/clipboard'
import { fromA1, toA1 } from '../engine/a1'
import { makeRange } from '../engine/range'
import type { Sheet } from '../engine/create-sheet'
import type { CellAddress } from '../engine/types'

/** Pull the A1 of the cell under a pointer event from its `data-a1` host. */
export function a1FromEvent(e: Event): string | null {
  const path = e.composedPath()
  for (const node of path) {
    if (node instanceof HTMLElement && node.dataset.a1) return node.dataset.a1
  }
  return null
}

/** A printable single character (the "start typing to edit" trigger). */
export function isPrintable(e: KeyboardEvent): boolean {
  return e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey
}

/** Map a keydown to a `moveActive` direction (or null for a non-nav key). */
export function navDir(e: KeyboardEvent): Parameters<Sheet['moveActive']>[0] | null {
  switch (e.key) {
    case 'ArrowUp':
      return 'up'
    case 'ArrowDown':
      return 'down'
    case 'ArrowLeft':
      return 'left'
    case 'ArrowRight':
      return 'right'
    case 'Home':
      return e.ctrlKey || e.metaKey ? 'ctrl-home' : 'home'
    case 'End':
      return e.ctrlKey || e.metaKey ? 'ctrl-end' : 'end'
    case 'Tab':
      return 'tab'
    case 'Enter':
      return 'enter'
    default:
      return null
  }
}

/** Emit `sui-selection-change` with the current active + ranges (A1 strings). */
export function emitSelection(host: HTMLElement, sheet: Sheet): void {
  const sel = sheet.selection()
  host.emit?.('sui-selection-change', {
    active: { row: sel.active.row, col: sel.active.col },
    a1: toA1(sel.active),
    ranges: sel.ranges.map((r) => r.toA1()),
  })
}

/** Begin editing at `cell` (or the active cell), emitting `sui-cell-activate`. */
export function activateEdit(host: HTMLElement, sheet: Sheet, seed?: string): void {
  const cell = sheet.activeCell()
  sheet.startEditing(cell, seed)
  host.emit?.('sui-cell-activate', { a1: toA1(cell), row: cell.row, col: cell.col })
}

/** Commit the open edit, emitting `sui-cell-commit` with the before/after. */
export function commitEdit(
  host: HTMLElement,
  sheet: Sheet,
  advance: 'down' | 'right' | 'none',
): void {
  const editing = sheet.editing()
  if (!editing) return
  const a1 = toA1(editing.cell)
  const before = sheet.getCell(a1)().raw
  sheet.commitEdit({ advance })
  const after = sheet.getCell(a1)().raw
  host.emit?.('sui-cell-commit', { a1, before, after })
}

/** Copy the active selection's first range to the DataTransfer as TSV. Returns
 *  the source top-left so a later paste can re-base relative formula refs. */
export function copySelection(sheet: Sheet, dt: DataTransfer | null): CellAddress | null {
  const range = sheet.selection().ranges[0]
  if (!range || !dt) return null
  dt.setData('text/plain', rangeToTsv(sheet, range))
  const tl = range.normalized().start
  dt.setData('application/x-sui-sheet-anchor', toA1(tl))
  return tl
}

/** Paste TSV from the DataTransfer at the active cell, re-basing relative refs
 *  from the recorded source anchor. */
export function pasteAt(sheet: Sheet, dt: DataTransfer | null): void {
  if (!dt) return
  const tsv = dt.getData('text/plain')
  if (!tsv) return
  const anchorAttr = dt.getData('application/x-sui-sheet-anchor')
  let source: CellAddress | undefined
  if (anchorAttr) {
    try {
      source = fromA1(anchorAttr)
    } catch {
      source = undefined
    }
  }
  const edits = tsvToEdits(tsv, sheet.activeCell(), source)
  if (edits.length > 0) sheet.setCells(edits)
}

/** Series-fill from the active selection's first range out to (and including)
 *  the drop cell, then re-select the union range. Returns false when there is
 *  no source range or the drop is inside it (no extension). */
export function fillTo(sheet: Sheet, dropA1: string): boolean {
  const from = sheet.selection().ranges[0]
  if (!from) return false
  const drop = fromA1(dropA1)
  const n = from.normalized()
  // Extend along the dominant axis of the drag (whichever exceeds the source).
  const target =
    Math.abs(drop.row - n.end.row) >= Math.abs(drop.col - n.end.col)
      ? makeRange(n.start, { row: drop.row, col: n.end.col })
      : makeRange(n.start, { row: n.end.row, col: drop.col })
  if (target.width() <= n.width() && target.height() <= n.height()) return false
  sheet.fillRange(from, target, 'series')
  sheet.setSelection([target])
  return true
}

/** The scroll seam: a virtualizer's `scrollToIndex` per axis. */
export interface ScrollAxes {
  rowToIndex(i: number): void
  colToIndex(i: number): void
}

/** The context the interaction wiring needs from the controller scope. */
export interface InteractionContext {
  /** The live engine (read fresh per event so a prop:sheet swap is honored). */
  sheetOf(): Sheet | null
  /** The focusable grid element (returns focus to it after a commit). */
  focusGrid(): void
  scroll: ScrollAxes
}

/** Install ALL surface handlers on the host's shadow root. Cells live in the
 *  shadow tree; one delegated listener per event routes by target. Every
 *  mutation runs inside a handler (Cardinal 3); cleanups auto-enroll via
 *  `bindNativeEvent` against the active effect-scope. */
export function wireSheetInteractions(host: HTMLElement, ctx: InteractionContext): void {
  const sr = host.shadowRoot
  if (!sr) return
  const { sheetOf, focusGrid } = ctx

  const addrAt = (s: Sheet, a1: string): CellAddress => s.getCell(a1)().address

  // Pointer drag (select OR fill) — transient document-less listeners that
  // self-remove on pointerup.
  const dragWithin = (onMove: (a1: string) => void, onUp: () => void): void => {
    const move = (ev: Event): void => {
      const a1 = a1FromEvent(ev)
      if (a1) onMove(a1)
    }
    const up = (): void => {
      sr.removeEventListener('pointermove', move)
      sr.removeEventListener('pointerup', up)
      onUp()
    }
    sr.addEventListener('pointermove', move)
    sr.addEventListener('pointerup', up)
  }

  const onPointerDown = (e: Event): void => {
    const s = sheetOf()
    if (!s) return
    if ((e.target as HTMLElement).closest('[part="fill-handle"]')) {
      let last: string | null = null
      dragWithin(
        (a1) => (last = a1),
        () => {
          if (last) {
            fillTo(s, last)
            emitSelection(host, s)
          }
        },
      )
      return
    }
    const a1 = a1FromEvent(e)
    if (!a1) return
    const shift = (e as MouseEvent).shiftKey
    s.setActiveCell(addrAt(s, a1), shift ? { extend: true } : undefined)
    emitSelection(host, s)
    focusGrid()
    dragWithin(
      (cur) => s.setActiveCell(addrAt(s, cur), { extend: true }),
      () => emitSelection(host, s),
    )
  }

  const onDblClick = (e: Event): void => {
    const s = sheetOf()
    if (!s) return
    const a1 = a1FromEvent(e)
    if (a1) s.setActiveCell(addrAt(s, a1))
    activateEdit(host, s)
  }

  const onKeyDown = (e: Event): void => {
    const s = sheetOf()
    if (!s) return
    if (s.editing()) return // editor owns the keystrokes
    if ((e.target as HTMLElement).classList?.contains('cell-editor')) return
    const ke = e as KeyboardEvent
    const dir = navDir(ke)
    if (dir) {
      e.preventDefault()
      if (ke.key === 'Tab') s.moveActive(ke.shiftKey ? 'left' : 'right')
      else if (ke.key === 'Enter') s.moveActive(ke.shiftKey ? 'up' : 'down')
      else s.moveActive(dir, ke.shiftKey ? { extend: true } : undefined)
      emitSelection(host, s)
      const a = s.activeCell()
      ctx.scroll.rowToIndex(Math.max(0, a.row - s.centerRowRange().start))
      ctx.scroll.colToIndex(Math.max(0, a.col - s.centerColRange().start))
      return
    }
    if (isPrintable(ke)) {
      e.preventDefault()
      activateEdit(host, s, ke.key)
      return
    }
    if (ke.key === 'Delete' || ke.key === 'Backspace') {
      e.preventDefault()
      const range = s.selection().ranges[0]
      if (range) s.clearCells(range, 'values')
    }
  }

  const onCopy = (e: Event): void => {
    const s = sheetOf()
    if (!s) return
    copySelection(s, (e as ClipboardEvent).clipboardData)
    e.preventDefault()
  }
  const onPaste = (e: Event): void => {
    const s = sheetOf()
    if (!s) return
    pasteAt(s, (e as ClipboardEvent).clipboardData)
    e.preventDefault()
  }

  const isEditor = (e: Event): HTMLInputElement | null => {
    const el = e.target as HTMLElement
    return el.classList?.contains('cell-editor') ? (el as HTMLInputElement) : null
  }
  const onEditorInput = (e: Event): void => {
    const s = sheetOf()
    const input = isEditor(e)
    if (s && input) s.updateDraft(input.value)
  }
  const onEditorKey = (e: Event): void => {
    const s = sheetOf()
    if (!s || !isEditor(e)) return
    const ke = e as KeyboardEvent
    if (ke.key === 'Enter') {
      e.preventDefault()
      commitEdit(host, s, ke.shiftKey ? 'none' : 'down')
      emitSelection(host, s)
      focusGrid()
    } else if (ke.key === 'Tab') {
      e.preventDefault()
      commitEdit(host, s, 'right')
      emitSelection(host, s)
      focusGrid()
    } else if (ke.key === 'Escape') {
      e.preventDefault()
      s.cancelEdit()
      focusGrid()
    }
  }

  bindNativeEvent(sr, 'pointerdown', onPointerDown)
  bindNativeEvent(sr, 'dblclick', onDblClick)
  bindNativeEvent(sr, 'keydown', onKeyDown)
  bindNativeEvent(sr, 'keydown', onEditorKey) // routed by target
  bindNativeEvent(sr, 'copy', onCopy)
  bindNativeEvent(sr, 'paste', onPaste)
  bindNativeEvent(sr, 'input', onEditorInput)
}
