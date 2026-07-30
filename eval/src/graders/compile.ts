import { readdirSync, statSync } from 'node:fs'
import { join } from 'node:path'
import type { HostTooling } from '../host.ts'
import type { CompileGrade, Diagnostic, Trajectory } from '../types.ts'

export interface CompileGradeOpts {
  host: HostTooling
  workdir: string
  trajectory?: Trajectory
  /** When true and sail is missing, return a skipped grade rather than fail. */
  allowSkip?: boolean
}

/** Collect .sail files under dir (skip node_modules / hidden). */
export function collectSailFiles(dir: string): string[] {
  const out: string[] = []
  const walk = (d: string) => {
    let entries
    try {
      entries = readdirSync(d, { withFileTypes: true })
    } catch {
      return
    }
    for (const e of entries) {
      if (e.name === 'node_modules' || e.name === 'target' || e.name.startsWith('.')) continue
      const full = join(d, e.name)
      if (e.isDirectory()) walk(full)
      else if (e.isFile() && e.name.endsWith('.sail')) out.push(full)
    }
  }
  if (statSync(dir).isDirectory()) walk(dir)
  return out.sort()
}

export async function gradeCompile(opts: CompileGradeOpts): Promise<CompileGrade> {
  const { host, workdir, trajectory, allowSkip = false } = opts
  if (!host.sailBin) {
    if (allowSkip) {
      return {
        ok: true,
        errorCount: 0,
        warningCount: 0,
        diagnostics: [],
        repairCount: 0,
        iterationsToGreen: trajectory ? trajectory.turnCount : null,
        skipped: true,
        detail: 'sail binary not available — compile grade skipped',
      }
    }
    return {
      ok: false,
      errorCount: 1,
      warningCount: 0,
      diagnostics: [{ severity: 'error', message: 'sail binary not available' }],
      repairCount: 0,
      iterationsToGreen: null,
    }
  }

  const files = collectSailFiles(workdir)
  if (files.length === 0) {
    return {
      ok: false,
      errorCount: 1,
      warningCount: 0,
      diagnostics: [{ severity: 'error', message: 'no .sail files in workdir' }],
      repairCount: 0,
      iterationsToGreen: null,
      detail: 'author produced no sail sources',
    }
  }

  const allDiags: Diagnostic[] = []
  let repairCount = 0
  let anyFail = false
  let hostError: string | undefined

  for (const f of files) {
    const r = await host.check(f)
    if (r.error && !r.raw) {
      hostError = r.error
      anyFail = true
      allDiags.push({ severity: 'error', message: r.error, path: f })
      continue
    }
    allDiags.push(
      ...r.diagnostics.map((d) => ({
        ...d,
        path: d.path ?? f,
      })),
    )
    repairCount += r.autoFixed?.length ?? 0
    if (!r.ok) anyFail = true
  }

  const errorCount = allDiags.filter((d) => d.severity === 'error').length
  const warningCount = allDiags.filter((d) => d.severity === 'warning').length
  const ok = !anyFail && errorCount === 0

  return {
    ok,
    errorCount,
    warningCount,
    diagnostics: allDiags,
    repairCount,
    iterationsToGreen: ok ? (trajectory?.turnCount ?? 1) : null,
    detail: hostError,
  }
}
