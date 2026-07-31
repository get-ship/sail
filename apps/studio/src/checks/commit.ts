/**
 * Push applied fix text into the edit buffer (+ optional disk write).
 * Checks store owns sourceByPath; edit store owns the open document.
 */
import {
  getEditState,
  getOpsClient,
  setSourceText,
} from '@ship/studio/edit'

function pathsMatch(a: string, b: string): boolean {
  if (a === b) return true
  const na = a.replace(/\\/g, '/')
  const nb = b.replace(/\\/g, '/')
  return na.endsWith('/' + nb) || nb.endsWith('/' + na) || na.endsWith(nb) || nb.endsWith(na)
}

/** Update every open edit doc that matches path; fire-and-forget writeSource. */
export function commitAppliedSource(path: string, text: string): void {
  const edit = getEditState()
  const targets = new Set<string>()
  if (edit.docs.has(path)) targets.add(path)
  for (const p of edit.docs.keys()) {
    if (pathsMatch(p, path)) targets.add(p)
  }
  if (edit.activePath && pathsMatch(edit.activePath, path)) {
    targets.add(edit.activePath)
  }
  for (const p of targets) setSourceText(p, text)

  const ops = getOpsClient()
  if (ops?.writeSource) {
    const writePath = targets.size > 0 ? [...targets][0]! : path
    void ops.writeSource(writePath, text).catch(() => {
      /* offline / denied — buffer still holds the fix */
    })
  }
}

export function commitAppliedSources(sourceByPath: Map<string, string>, paths: string[]): void {
  for (const path of paths) {
    const text = sourceByPath.get(path)
    if (text != null) commitAppliedSource(path, text)
  }
}
