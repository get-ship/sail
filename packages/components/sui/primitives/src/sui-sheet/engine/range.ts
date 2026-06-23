// sui-sheet engine — the CellRange value type (range algebra).
//
// A rectangular cell range carrying its corner addresses + the methods the
// selection/fill/clipboard modules and the view read: `normalized()` (top-left /
// bottom-right regardless of drag direction), `contains`, `width`/`height`
// (inclusive spans), `toA1()` (`'A1:C3'`), and `cells()` iteration. The factory
// is the ONE place a range is constructed so every consumer shares the algebra.
//
// PURE TypeScript — the A1 projection comes from `a1.ts`.
//
// Authority: build-spec §"Authoritative Cell / Selection / Range / DAG API"
// (CellRange members), §Features #4 (range selection).

import { toA1 } from './a1'
import type { CellAddress, CellRange } from './types'

/** Construct a `CellRange` from two corner addresses (any drag direction). */
export function makeRange(start: CellAddress, end: CellAddress): CellRange {
  const s: CellAddress = { row: start.row, col: start.col }
  const e: CellAddress = { row: end.row, col: end.col }

  const range: CellRange = {
    start: s,
    end: e,
    normalized(): CellRange {
      const top = Math.min(s.row, e.row)
      const bottom = Math.max(s.row, e.row)
      const left = Math.min(s.col, e.col)
      const right = Math.max(s.col, e.col)
      return makeRange({ row: top, col: left }, { row: bottom, col: right })
    },
    contains(a: CellAddress): boolean {
      const top = Math.min(s.row, e.row)
      const bottom = Math.max(s.row, e.row)
      const left = Math.min(s.col, e.col)
      const right = Math.max(s.col, e.col)
      return a.row >= top && a.row <= bottom && a.col >= left && a.col <= right
    },
    width(): number {
      return Math.abs(e.col - s.col) + 1
    },
    height(): number {
      return Math.abs(e.row - s.row) + 1
    },
    toA1(): string {
      const n = range.normalized()
      const a = toA1({ row: n.start.row, col: n.start.col })
      const b = toA1({ row: n.end.row, col: n.end.col })
      return a === b ? a : `${a}:${b}`
    },
    *cells(): Iterable<CellAddress> {
      const n = range.normalized()
      for (let r = n.start.row; r <= n.end.row; r++) {
        for (let c = n.start.col; c <= n.end.col; c++) {
          yield { row: r, col: c }
        }
      }
    },
  }
  return range
}

/** Coerce a plain `{start,end}` (e.g. from the controlled-state seed or a test)
 *  into a full `CellRange` with methods. Idempotent on an already-made range. */
export function ensureRange(r: CellRange | { start: CellAddress; end: CellAddress }): CellRange {
  if (typeof (r as CellRange).normalized === 'function') return r as CellRange
  return makeRange(r.start, r.end)
}
