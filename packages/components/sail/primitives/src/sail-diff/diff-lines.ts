/** Pure LCS + unified-diff parsers for sail-diff line rows. */

export type DiffKind = 'add' | 'del' | 'ctx' | 'hunk'

export interface DiffRawLine {
  kind: DiffKind
  text: string
}

export interface DiffLine {
  id: number
  key: string
  kind: DiffKind
  text: string
  sign: string
  label: string
}

const SIGNS: Record<DiffKind, string> = { add: '+', del: '-', hunk: '@', ctx: ' ' }
const LABELS: Record<DiffKind, string> = {
  add: 'added',
  del: 'removed',
  hunk: 'hunk',
  ctx: 'context',
}

/** LCS-based line diff of original `a` vs revised `b`. */
export function diffKinds(a: string, b: string): DiffRawLine[] {
  const A = a.split('\n')
  const B = b.split('\n')
  const n = A.length
  const m = B.length
  const dp: number[][] = Array.from({ length: n + 1 }, () => new Array(m + 1).fill(0))
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      dp[i]![j] = A[i] === B[j] ? dp[i + 1]![j + 1]! + 1 : Math.max(dp[i + 1]![j]!, dp[i]![j + 1]!)
    }
  }
  const out: DiffRawLine[] = []
  let i = 0
  let j = 0
  while (i < n && j < m) {
    if (A[i] === B[j]) {
      out.push({ kind: 'ctx', text: A[i]! })
      i++
      j++
    } else if (dp[i + 1]![j]! >= dp[i]![j + 1]!) {
      out.push({ kind: 'del', text: A[i]! })
      i++
    } else {
      out.push({ kind: 'add', text: B[j]! })
      j++
    }
  }
  while (i < n) out.push({ kind: 'del', text: A[i++]! })
  while (j < m) out.push({ kind: 'add', text: B[j++]! })
  return out
}

/** Parse a raw unified-diff string into typed lines. */
export function parseUnified(value: string): DiffRawLine[] {
  return value.split('\n').map((line) => {
    if (line.startsWith('@@')) return { kind: 'hunk' as DiffKind, text: line }
    if (line.startsWith('+')) return { kind: 'add' as DiffKind, text: line.slice(1) }
    if (line.startsWith('-')) return { kind: 'del' as DiffKind, text: line.slice(1) }
    return { kind: 'ctx' as DiffKind, text: line.startsWith(' ') ? line.slice(1) : line }
  })
}

/** Resolve from/to/value into rendered row models (value wins over from/to). */
export function buildDiffLines(from: string, to: string, value: string): DiffLine[] {
  const raw = value ? parseUnified(value) : from || to ? diffKinds(from, to) : []
  return raw.map((l, i) => ({
    id: i,
    key: `${i}:${l.kind}`,
    kind: l.kind,
    text: l.text,
    sign: SIGNS[l.kind],
    label: `${LABELS[l.kind]}: ${l.text}`,
  }))
}
