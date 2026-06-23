// sui-sheet engine — the cell editing model (backs sui-cell-commit).
//
// `editing = { cell, draft } | null`. `startEditing` opens an editor at a cell
// (or the active cell), seeded with the first typed char; `updateDraft` tracks
// keystrokes; `commitEdit` writes the draft as the cell's value (a formula draft
// — leading `=` — is stored as `formula`, the literal path keeps the eval for
// the formula slice) and advances per `{advance}`; `cancelEdit` discards. Every
// mutator batches the slice write. Nav is suspended while `editing != null`
// (the controller consults `editing()` before routing arrows).
//
// PURE TypeScript — `batch` from `@ship/sui-runtime` ONLY.
//
// Authority: build-spec §Features #10 (editing model), §"API sketch"
// (startEditing/updateDraft/commitEdit/cancelEdit), §Events (sui-cell-commit).

import { batch } from '@ship/sui-runtime'
import { toA1 } from './a1'
import type { InteractionState } from './state'
import type { CellAddress, CellData } from './types'

export interface EditingApi {
  startEditing(active: CellAddress, a?: CellAddress, seed?: string): void
  updateDraft(text: string): void
  /** Returns the committed edit (a1 + draft) so the assembler can write the
   *  cell + fire onCellChange, plus the advance direction. */
  commitEdit(opts?: { advance?: 'down' | 'right' | 'none' }): {
    a1: string
    cell: CellAddress
    data: CellData
    advance: 'down' | 'right' | 'none'
  } | null
  cancelEdit(): void
}

export function buildEditing(state: InteractionState): EditingApi {
  const startEditing = (active: CellAddress, a?: CellAddress, seed?: string): void => {
    const cell = a ?? active
    batch(() => state.editing.set({ cell, draft: seed ?? '' }))
  }

  const updateDraft = (text: string): void => {
    const cur = state.editing()
    if (!cur) return
    batch(() => state.editing.set({ cell: cur.cell, draft: text }))
  }

  const commitEdit = (
    opts?: { advance?: 'down' | 'right' | 'none' },
  ): ReturnType<EditingApi['commitEdit']> => {
    const cur = state.editing()
    if (!cur) return null
    const advance = opts?.advance ?? 'down'
    const data = draftToCellData(cur.draft)
    batch(() => state.editing.set(null))
    return { a1: toA1(cur.cell), cell: cur.cell, data, advance }
  }

  const cancelEdit = (): void => {
    if (state.editing() == null) return
    batch(() => state.editing.set(null))
  }

  return { startEditing, updateDraft, commitEdit, cancelEdit }
}

/** Map a committed draft string to `CellData`. A leading `=` is a formula
 *  (stored for the formula slice to parse/eval); a numeric draft becomes a
 *  number literal; everything else is a string literal; empty clears the cell. */
export function draftToCellData(draft: string): CellData {
  const trimmed = draft.trim()
  if (trimmed === '') return {}
  if (trimmed.startsWith('=')) return { formula: trimmed }
  const num = Number(trimmed)
  if (trimmed !== '' && !Number.isNaN(num) && /^-?\d*\.?\d+$/.test(trimmed)) {
    return { value: num }
  }
  if (trimmed === 'TRUE') return { value: true }
  if (trimmed === 'FALSE') return { value: false }
  return { value: draft }
}
