/**
 * Findings list grouping — by severity (primary inbox) and by rule.
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

export function severityLabel(severity: string): string {
  if (severity === 'error') return 'Errors'
  if (severity === 'warning') return 'Warnings'
  if (severity === 'info') return 'Info'
  if (severity === 'hint') return 'Hints'
  return severity
}

export function severityToneOf(severity: string): string {
  if (severity === 'error') return 'destructive'
  if (severity === 'warning') return 'warning'
  return 'muted'
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
    const ia = SEVERITY_ORDER.indexOf(a)
    const ib = SEVERITY_ORDER.indexOf(b)
    return (ia < 0 ? 99 : ia) - (ib < 0 ? 99 : ib) || a.localeCompare(b)
  })
  return keys.map((severity) => {
    const items = map.get(severity)!
    return {
      severity,
      label: severityLabel(severity),
      tone: severityToneOf(severity),
      count: items.length,
      items,
    }
  })
}
