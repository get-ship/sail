// sui-table engine — ARIA-tree row navigation (Backbone #1, Feature #14, #240).
//
// The navMode:'row' Left/Right + Home/End behavior for a TREE/treegrid, factored
// out of `nav.ts` so the dispatcher stays under the file cap. Gated on
// getCanExpand()/depth so a FLAT table (depth-0 leaves, no subRows) falls through
// to the ordinary column move — flat-table row nav is UNAFFECTED.
//
// PURE TypeScript — no runtime imports (the focus signals live in `nav.ts`; this
// module only reads/derives over the passed-in context). No DOM, no SDK.
//
// Authority: build-spec §Features #14 ("Keyboard-nav hooks"); #240 follow-up 3
// (Left collapse-or-parent · Right expand-or-first-child · Home/End first/last
// visible row).

import type { Row } from './types'

/** The focus-model surface the tree-nav helpers drive (owned by `nav.ts`). */
export interface TreeNavCtx<T> {
  /** The active row, if resolvable in the displayed set. */
  activeRow: () => Row<T> | undefined
  /** The displayed (post sort/filter/page) rows — the nav grid. */
  rows: () => Row<T>[]
  /** Ordinary column move (the FLAT-table fall-through for Left/Right). */
  move: (dir: 'left' | 'right') => void
  /** Move row focus to a clamped row index (keeps the active column). */
  focusRowAt: (index: number) => void
  /** Move row focus to a row by id (no-op when the id is not displayed). */
  focusRowId: (id: string) => void
}

export interface TreeNav {
  /** navMode:'row' Left/Right (expand/collapse/descend/ascend, else column move). */
  arrow: (dir: 'left' | 'right') => boolean
  /** navMode:'row' Home/End → first/last VISIBLE row; false when not a tree row. */
  rowEnd: (dir: 'home' | 'end') => boolean
}

/** A row participates in tree nav when it can expand OR sits below the root
 *  (depth > 0). A FLAT table's rows are depth-0 leaves → false → column move. */
export function isTreeRow<T>(row: Row<T> | undefined): boolean {
  return !!row && (row.getCanExpand() || row.depth > 0)
}

export function buildTreeNav<T>(ctx: TreeNavCtx<T>): TreeNav {
  /**
   * ARIA-tree Left/Right in navMode:'row':
   *   Right — collapsed node → EXPAND; expanded node → first CHILD; leaf → no-op.
   *   Left  — expanded node → COLLAPSE; collapsed-or-leaf → move to PARENT.
   * A FLAT row (not a tree row) falls through to the ordinary column move.
   */
  function arrow(dir: 'left' | 'right'): boolean {
    const row = ctx.activeRow()
    if (!isTreeRow(row) || !row) {
      ctx.move(dir) // FLAT row — ordinary column move
      return true
    }
    if (dir === 'right') {
      if (row.getCanExpand()) {
        if (!row.getIsExpanded()) {
          row.toggleExpanded(true) // collapsed → expand in place
          return true
        }
        const first = row.subRows[0] // expanded → first child
        if (first) ctx.focusRowId(first.id)
        return true
      }
      return true // tree leaf — no children to descend into (no-op)
    }
    // Left: expanded node → collapse; collapsed-or-leaf → move to parent.
    if (row.getCanExpand() && row.getIsExpanded()) {
      row.toggleExpanded(false)
      return true
    }
    const parent = row.getParentRow()
    if (parent) ctx.focusRowId(parent.id)
    return true // depth-0 collapsed root with no parent: handled no-op
  }

  /**
   * Row-mode Home/End. In a TREE grid (the active row participates in a tree)
   * they jump to the first / last VISIBLE row. In a FLAT grid they return false
   * so the dispatcher falls back to the column ends (legacy behavior).
   */
  function rowEnd(dir: 'home' | 'end'): boolean {
    if (!isTreeRow(ctx.activeRow())) return false
    ctx.focusRowAt(dir === 'home' ? 0 : ctx.rows().length - 1)
    return true
  }

  return { arrow, rowEnd }
}
