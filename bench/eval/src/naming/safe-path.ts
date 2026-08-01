/**
 * Path boundary checks — reject sibling-prefix escapes (/lab vs /lab2).
 */

import { relative, resolve, sep } from 'node:path'

/** True iff `candidate` is `root` or a path strictly under `root`. */
export function isPathInside(root: string, candidate: string): boolean {
  const r = resolve(root)
  const c = resolve(candidate)
  if (c === r) return true
  const rel = relative(r, c)
  // outside or equal-with-escape
  if (!rel || rel === '..' || rel.startsWith(`..${sep}`) || rel.startsWith('..\\')) {
    return false
  }
  // absolute rel means different drive / escaped on some platforms
  if (rel.startsWith(sep) || /^[a-zA-Z]:/.test(rel)) return false
  return true
}
