/**
 * Findings list grouping — by rule (primary inbox) and by severity.
 * Rule groups sorted by count desc; items within a group by severity then line.
 */
import type { Finding } from './types'

const SEVERITY_ORDER = ['error', 'warning', 'info', 'hint']

export type SeverityGroup = {
  severity: string
  label: string
  tone: string
  count: number
  items: Finding[]
}

export type RuleGroup = {
  rule: string
  count: number
  errorCount: number
  warningCount: number
  infoCount: number
  /** Machine-fixable count (caller may set; default 0). */
  fixableCount: number
  items: Finding[]
}

export function severityLabel(severity: string): string {
  if (severity === 'error') return 'Errors'
  if (severity === 'warning') return 'Warnings'
  if (severity === 'info') return 'Info'
  if (severity === 'hint') return 'Hints'
  return severity
}

export function severityToneOf(severity: string): string {
  if (severity === 'error') return 'danger'
  if (severity === 'warning') return 'warning'
  return 'muted'
}

function severityRank(severity: string): number {
  const i = SEVERITY_ORDER.indexOf(String(severity || 'error'))
  return i < 0 ? 99 : i
}

/** Stable within-group order: errors → warnings → info, then file:line. */
export function sortFindingsStable(items: Finding[]): Finding[] {
  return [...items].sort((a, b) => {
    const sr = severityRank(a.severity) - severityRank(b.severity)
    if (sr !== 0) return sr
    const path = a.path.localeCompare(b.path)
    if (path !== 0) return path
    return a.line - b.line || a.col - b.col || a.id.localeCompare(b.id)
  })
}

/** Group by severity; errors first, then warnings, then the rest. */
export function groupBySeverity(findings: Finding[]): SeverityGroup[] {
  const map = new Map<string, Finding[]>()
  for (const f of findings) {
    const s = String(f.severity || 'error')
    if (!map.has(s)) map.set(s, [])
    map.get(s)!.push(f)
  }
  const keys = [...map.keys()].sort((a, b) => {
    return severityRank(a) - severityRank(b) || a.localeCompare(b)
  })
  return keys.map((severity) => {
    const items = sortFindingsStable(map.get(severity)!)
    return {
      severity,
      label: severityLabel(severity),
      tone: severityToneOf(severity),
      count: items.length,
      items,
    }
  })
}

/**
 * Group by rule. Groups sorted by count desc (then rule name);
 * items within each group use stable severity order.
 */
export function groupByRule(findings: Finding[]): RuleGroup[] {
  const map = new Map<string, Finding[]>()
  for (const f of findings) {
    const rule = f.rule || 'unknown'
    if (!map.has(rule)) map.set(rule, [])
    map.get(rule)!.push(f)
  }
  const groups: RuleGroup[] = []
  for (const [rule, raw] of map) {
    const items = sortFindingsStable(raw)
    let errorCount = 0
    let warningCount = 0
    let infoCount = 0
    for (const f of items) {
      if (f.severity === 'error') errorCount++
      else if (f.severity === 'warning') warningCount++
      else infoCount++
    }
    groups.push({
      rule,
      count: items.length,
      errorCount,
      warningCount,
      infoCount,
      fixableCount: 0,
      items,
    })
  }
  groups.sort(
    (a, b) => b.count - a.count || a.rule.localeCompare(b.rule),
  )
  return groups
}

/** Flatten rule groups (count-desc) into table row order for createTable grouping. */
export function flattenRuleGroups(groups: RuleGroup[]): Finding[] {
  return groups.flatMap((g) => g.items)
}
