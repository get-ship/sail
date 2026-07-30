import type { HostTooling } from '../host.ts'
import type { GatesGrade } from '../types.ts'
import { collectSailFiles } from './compile.ts'

export interface GatesGradeOpts {
  host: HostTooling
  workdir: string
  allowSkip?: boolean
}

/**
 * Mechanical quality gates: unresolved lint (check errors), layout findings,
 * compose findings (when present), and [sail:fix] repair count signal.
 */
export async function gradeGates(opts: GatesGradeOpts): Promise<GatesGrade> {
  const allowSkip = opts.allowSkip ?? false
  if (!opts.host.sailBin) {
    if (allowSkip) {
      return {
        ok: true,
        lintErrors: 0,
        layoutFindings: 0,
        composeFindings: 0,
        repairCount: 0,
        skipped: true,
        detail: 'sail binary not available — gates skipped',
      }
    }
    return {
      ok: false,
      lintErrors: 1,
      layoutFindings: 0,
      composeFindings: 0,
      repairCount: 0,
      detail: 'sail binary not available',
    }
  }

  const files = collectSailFiles(opts.workdir)
  let lintErrors = 0
  let composeFindings = 0
  let repairCount = 0
  let layoutFindings = 0
  let hostFailures = 0
  const details: string[] = []

  for (const f of files) {
    const check = await opts.host.check(f)
    // Host crash / empty response with no usable diagnostics → gate failure
    // (mirrors compile grader).
    if (check.error && !check.raw) {
      hostFailures++
      lintErrors++
      details.push(check.error)
      continue
    }
    for (const d of check.diagnostics) {
      if (d.severity === 'error') lintErrors++
      if (d.rule?.includes('compose') || d.message?.includes('compose')) {
        composeFindings++
      }
    }
    repairCount += check.autoFixed?.length ?? 0

    const layout = await opts.host.layout(f)
    if (layout.error) details.push(layout.error)
    layoutFindings += layout.findings.length
  }

  // Gates pass when no lint/host errors. Layout findings are advisory (spike).
  const ok = lintErrors === 0 && hostFailures === 0
  return {
    ok,
    lintErrors,
    layoutFindings,
    composeFindings,
    repairCount,
    detail: details.length ? details.join('; ') : undefined,
  }
}
