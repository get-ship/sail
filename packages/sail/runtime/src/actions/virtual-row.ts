import type { Action } from '../action'

/**
 * Virtual geometry is `bind(signal)` (controller), not `--row-y` from
 * an action. These stamps stay as names until Job E deletes widget-tied
 * `use:`. They do not write style.
 */

export const virtualRow: Action<number | null | undefined> = (_node, _initial) => {
  return { update: (_next: number | null | undefined) => {} }
}

export const virtualSpacer: Action<number | null | undefined> = (_node, _initial) => {
  return { update: (_next: number | null | undefined) => {} }
}

export const cellPad: Action<number | null | undefined> = (_node, _initial) => {
  return { update: (_next: number | null | undefined) => {} }
}
