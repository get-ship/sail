// Overlay position is CSS `position-anchor` / `position-area` (Job A).
// This action is inert: it does not write geometry. Kept as a name so
// leftover `use:anchor` stamps do not throw until Job E deletes them.

import type { Action } from '../action'

export type AnchorPlacement =
  | 'top'
  | 'top-end'
  | 'bottom'
  | 'bottom-end'
  | 'left'
  | 'left-end'
  | 'right'
  | 'right-end'

export const anchor: Action<AnchorPlacement | undefined> = (_node, _placement) => {
  return {}
}
