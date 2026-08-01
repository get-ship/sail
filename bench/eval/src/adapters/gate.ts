/**
 * gate phase — per-arm completion. Structured results, never grepped text.
 */

import { existsSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { spawnSync } from 'node:child_process'
import {
  CONTRACT_VERSION,
  type PhaseResult,
  type RunManifest,
  emptyPhaseResult,
} from '../contract/types.ts'
import { ARMS } from '../config/types.ts'
import { resolveSailBin } from '../naming/paths.ts'

function collectFiles(dir: string, ext: string): string[] {
  const out: string[] = []
  const walk = (d: string) => {
    let entries
    try {
      entries = readdirSync(d, { withFileTypes: true })
    } catch {
      return
    }
    for (const e of entries) {
      if (e.name === 'node_modules' || e.name === '.git' || e.name.startsWith('.')) continue
      const full = join(d, e.name)
      if (e.isDirectory()) walk(full)
      else if (e.isFile() && e.name.endsWith(ext)) out.push(full)
    }
  }
  if (existsSync(dir) && statSync(dir).isDirectory()) walk(dir)
  return out.sort()
}

function gateSailCheck(sandbox: string, sailBin: string | null): {
  ok: boolean | null
  detail: Record<string, unknown>
  missing?: string
} {
  if (!sailBin) {
    return {
      ok: null,
      detail: { type: 'sail_check', error: 'sail binary not available' },
      missing: 'sail_bin_unavailable',
    }
  }
  const files = collectFiles(sandbox, '.sail')
  if (files.length === 0) {
    return {
      ok: false,
      detail: { type: 'sail_check', error: 'no .sail files', files: [] },
    }
  }
  const diagnostics: unknown[] = []
  let anyFail = false
  for (const f of files) {
    const r = spawnSync(sailBin, ['check', f], {
      cwd: sandbox,
      encoding: 'utf8',
      env: { ...process.env },
      timeout: 120_000,
    })
    const entry = {
      file: f,
      exit: r.status,
      stdout_tail: (r.stdout ?? '').slice(-800),
      stderr_tail: (r.stderr ?? '').slice(-800),
    }
    diagnostics.push(entry)
    if (r.status !== 0) anyFail = true
  }
  return {
    ok: !anyFail,
    detail: { type: 'sail_check', ok: !anyFail, files, diagnostics },
  }
}

function gateViteBuild(sandbox: string): {
  ok: boolean | null
  detail: Record<string, unknown>
  missing?: string
} {
  const runners: string[][] = [
    ['bun', 'run', 'build'],
    ['npm', 'run', 'build', '--silent'],
  ]
  const tried: unknown[] = []
  let lastFail: { ok: false; detail: Record<string, unknown> } | null = null
  for (const cmd of runners) {
    const which = spawnSync('which', [cmd[0]!], { encoding: 'utf8' })
    if (which.status !== 0) {
      tried.push({ skip: cmd[0], reason: 'not on PATH' })
      continue
    }
    const r = spawnSync(cmd[0]!, cmd.slice(1), {
      cwd: sandbox,
      encoding: 'utf8',
      timeout: 300_000,
    })
    const dist = join(sandbox, 'dist/index.html')
    const ok = r.status === 0 && existsSync(dist)
    const detail = {
      type: 'vite_build',
      ok,
      cmd,
      exit: r.status,
      dist_present: existsSync(dist),
      stdout_tail: (r.stdout ?? '').slice(-500),
      stderr_tail: (r.stderr ?? '').slice(-500),
      tried,
    }
    if (ok) {
      return { ok: true, detail }
    }
    tried.push({ cmd, exit: r.status, dist_present: existsSync(dist) })
    lastFail = { ok: false, detail: { ...detail, tried: [...tried] } }
  }
  if (lastFail) return lastFail
  return {
    ok: false,
    detail: {
      type: 'vite_build',
      ok: false,
      error: 'neither bun nor npm available / build failed',
      tried,
    },
    missing: 'build_tooling',
  }
}

function gateHtmlStructure(sandbox: string): {
  ok: boolean | null
  detail: Record<string, unknown>
} {
  const index = join(sandbox, 'index.html')
  if (!existsSync(index)) {
    return { ok: false, detail: { type: 'html_structure', error: 'index.html missing' } }
  }
  const text = readFileSync(index, 'utf8').toLowerCase()
  const hasHtml = text.includes('<html')
  const hasBody = text.includes('<body')
  const ok = hasHtml && hasBody
  return {
    ok,
    detail: { type: 'html_structure', path: 'index.html', has_html: hasHtml, has_body: hasBody, ok },
  }
}

export async function gatePhase(manifest: RunManifest): Promise<PhaseResult> {
  const t0 = Date.now()
  const result = emptyPhaseResult('gate')
  result.contract_version = CONTRACT_VERSION

  const armCfg = ARMS[manifest.arm]
  const gateKind = armCfg?.gate ?? 'none'
  const sailBin =
    (manifest.env.SAIL_BIN && existsSync(manifest.env.SAIL_BIN) ? manifest.env.SAIL_BIN : null) ??
    resolveSailBin()

  let outcome: { ok: boolean | null; detail: Record<string, unknown>; missing?: string }

  switch (gateKind) {
    case 'sail_check':
      outcome = gateSailCheck(manifest.sandbox_dir, sailBin)
      break
    case 'vite_build':
      outcome = gateViteBuild(manifest.sandbox_dir)
      break
    case 'html_structure':
      outcome = gateHtmlStructure(manifest.sandbox_dir)
      break
    default:
      outcome = {
        ok: null,
        detail: { type: 'none', note: 'arm has no mechanical gate; not comparable' },
        missing: 'no_gate',
      }
  }

  const gatePath = join(manifest.run_dir, manifest.paths.gate)
  const gateBody = {
    arm: manifest.arm,
    gate: gateKind,
    ok: outcome.ok,
    detail: outcome.detail,
    ts: new Date().toISOString(),
  }
  writeFileSync(gatePath, JSON.stringify(gateBody, null, 2) + '\n')

  result.ok = outcome.ok === true
  result.verdict = outcome.ok === true ? 'pass' : outcome.ok === false ? 'fail' : 'skip'
  result.detail = gateBody
  result.metrics = { gate_ok: outcome.ok }
  if (outcome.ok === null) {
    result.missing.gate_ok = outcome.missing ?? 'gate_not_applicable'
    result.flags.push(outcome.missing ?? 'gate_null')
  }
  result.artifacts = [
    {
      name: 'gate.json',
      rel_path: manifest.paths.gate,
      blocking: true,
      status: 'present',
      bytes: JSON.stringify(gateBody).length,
    },
  ]
  result.wall_ms = Date.now() - t0
  return result
}
