/**
 * Fix review session — machine auto-fixes only (real rewrites, not suggest-only).
 * Pure helpers; store holds the session state.
 */
import { applyFixToSource, fileBase } from './findings'
import type { Finding, FixReviewItem } from './types'

export type { FixReviewItem }

/** True when applyFixToSource rewrites source bytes. */
export function isMachineFixable(source: string, finding: Finding): boolean {
  if (!source) return false
  const result = applyFixToSource(source, finding)
  return result.ok && result.text !== source
}

/** Findings with a known machine rewrite against the loaded source. */
export function machineFixableFindings(
  findings: Finding[],
  sourceByPath: Map<string, string>,
): Finding[] {
  return findings.filter((f) => {
    const src = sourceByPath.get(f.path) ?? ''
    return isMachineFixable(src, f)
  })
}

/** @deprecated use machineFixableFindings — kept name for call-site clarity */
export function fixableFindings(
  findings: Finding[],
  sourceByPath: Map<string, string>,
): Finding[] {
  return machineFixableFindings(findings, sourceByPath)
}

export function buildFixReviewItems(
  findings: Finding[],
  sourceByPath: Map<string, string>,
): FixReviewItem[] {
  const out: FixReviewItem[] = []
  for (const f of machineFixableFindings(findings, sourceByPath)) {
    const src = sourceByPath.get(f.path) ?? ''
    const lines = src.split('\n')
    const before = (lines[f.line - 1] ?? '').trimEnd() || f.message
    const applied = applyFixToSource(src, f)
    if (!applied.ok || applied.text === src) continue
    const nextLines = applied.text.split('\n')
    const after = (nextLines[f.line - 1] ?? applied.text).trimEnd()
    out.push({
      findingId: f.id,
      rule: f.rule,
      message: f.message,
      fileSpan: f.fileSpan || `${fileBase(f.path)}:${f.line}`,
      path: f.path,
      line: f.line,
      before,
      after,
      accepted: true,
      machine: true,
    })
  }
  return out
}

/**
 * Apply accepted review items. High line numbers first per path so
 * earlier rewrites don't shift later spans.
 */
export function applyAcceptedReviews(
  items: FixReviewItem[],
  findings: Finding[],
  sourceByPath: Map<string, string>,
): {
  sourceByPath: Map<string, string>
  applied: number
  appliedIds: string[]
  changedPaths: string[]
} {
  const accepted = items.filter((i) => i.accepted && i.machine)
  const byPath = new Map<string, FixReviewItem[]>()
  for (const item of accepted) {
    const list = byPath.get(item.path) ?? []
    list.push(item)
    byPath.set(item.path, list)
  }

  const next = new Map(sourceByPath)
  const appliedIds: string[] = []
  const changedPaths: string[] = []
  let applied = 0

  for (const [path, pathItems] of byPath) {
    pathItems.sort((a, b) => b.line - a.line)
    let text = next.get(path) ?? ''
    let pathChanged = false
    for (const item of pathItems) {
      const finding = findings.find((f) => f.id === item.findingId)
      if (!finding) continue
      const result = applyFixToSource(text, finding)
      if (result.ok && result.text !== text) {
        text = result.text
        applied++
        appliedIds.push(item.findingId)
        pathChanged = true
      }
    }
    if (pathChanged) {
      next.set(path, text)
      changedPaths.push(path)
    }
  }

  return { sourceByPath: next, applied, appliedIds, changedPaths }
}

export function countAccepted(items: FixReviewItem[]): number {
  return items.filter((i) => i.accepted).length
}
