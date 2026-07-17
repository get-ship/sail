// sail-board — pure keyboard-operability resolver.
//
// Maps a key (+ focused node id + current scale) to a board action and applies
// it through the BoardModel. Kept pure (no DOM) so it is unit-testable the same
// way the rest of the board engine is. The controller resolves the focused node
// from the event target and forwards here.

import type { BoardModel } from './board-model'

/** World-space nudge per arrow press, in screen pixels (divided by scale). */
export const ARROW_STEP = 10

export interface BoardKey {
  key: string
  shiftKey?: boolean
}

/** True when the key was a board action (caller should preventDefault). */
export function applyBoardKey(
  board: BoardModel,
  key: BoardKey,
  focusedId: string | null,
  scale: number,
): boolean {
  if (key.key === 'Escape') {
    board.clearSelection()
    board.cancelEdge()
    return true
  }
  if (key.key === 'Enter' || key.key === ' ') {
    if (!focusedId) return false
    if (key.shiftKey) board.toggleSelection(focusedId)
    else board.setSelection([focusedId])
    return true
  }
  const dir =
    key.key === 'ArrowLeft' ? { x: -1, y: 0 }
    : key.key === 'ArrowRight' ? { x: 1, y: 0 }
    : key.key === 'ArrowUp' ? { x: 0, y: -1 }
    : key.key === 'ArrowDown' ? { x: 0, y: 1 }
    : null
  if (!dir || !focusedId) return false
  const step = ARROW_STEP / (scale || 1)
  const dx = dir.x * step
  const dy = dir.y * step
  if (board.selectedNodes().includes(focusedId)) board.moveSelectionBy(dx, dy)
  else board.moveNodeBy(focusedId, dx, dy)
  return true
}
