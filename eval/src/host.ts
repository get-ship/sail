/**
 * Host tooling adapters — invoke the sail CLI when present.
 * Unit tests inject fakes via HostTooling; production uses realHost().
 */

import { spawnSync } from 'node:child_process'
import { existsSync } from 'node:fs'
import { isAbsolute, relative, resolve } from 'node:path'
import { findSailBin, repoRoot } from './paths.ts'
import type { Diagnostic } from './types.ts'

export interface CheckResult {
  ok: boolean
  diagnostics: Diagnostic[]
  autoFixed: Array<Record<string, unknown>>
  raw?: unknown
  error?: string
}

export interface TreeNode {
  id?: string
  tag: string
  layer?: string
  children?: string[]
  attrs?: Record<string, string>
}

export interface TreeResult {
  ok: boolean
  nodes: TreeNode[]
  error?: string
  raw?: unknown
}

export interface LayoutResult {
  ok: boolean
  findings: Array<Record<string, unknown>>
  error?: string
}

export interface HostTooling {
  /** sail binary path, or null if unavailable. */
  sailBin: string | null
  check(path: string): Promise<CheckResult>
  inspectTree(path: string): Promise<TreeResult>
  layout(path: string): Promise<LayoutResult>
}

/**
 * Extract JSON from sail stdout that may include log lines before the payload.
 * Tries the full string first, then each `{`/`[` from the end so brace-y log
 * lines (e.g. `info: loading {cache}`) do not poison the parse.
 * Exported for unit tests.
 */
export function parseJsonStdout(stdout: string): unknown {
  const trimmed = stdout.trim()
  if (!trimmed) return null
  try {
    return JSON.parse(trimmed)
  } catch {
    /* fall through — may have leading logs */
  }
  for (let i = trimmed.length - 1; i >= 0; i--) {
    const c = trimmed[i]
    if (c !== '{' && c !== '[') continue
    try {
      return JSON.parse(trimmed.slice(i))
    } catch {
      /* try an earlier brace */
    }
  }
  throw new Error(`no JSON in sail output: ${trimmed.slice(0, 200)}`)
}

/**
 * Sail ops require workspace-relative paths (absolute paths refused).
 * Convert absolute paths under the repo root to relative; leave already-relative as-is.
 */
export function toWorkspacePath(path: string, workspace = repoRoot()): string {
  const abs = isAbsolute(path) ? path : resolve(workspace, path)
  const rel = relative(workspace, abs)
  if (rel.startsWith('..') || isAbsolute(rel)) {
    throw new Error(
      `path escapes workspace (sail ops require workspace-relative paths): ${path}`,
    )
  }
  // sail ops use forward slashes
  return rel.split('\\').join('/')
}

export function realHost(opts?: { sailBin?: string | null; cwd?: string }): HostTooling {
  const cwd = opts?.cwd ?? repoRoot()
  const sailBin =
    opts?.sailBin === undefined ? findSailBin(cwd) : opts.sailBin

  function runOp(serviceMethod: string, body: Record<string, unknown>): {
    ok: boolean
    data: unknown
    error?: string
  } {
    if (!sailBin || !existsSync(sailBin)) {
      return { ok: false, data: null, error: 'sail binary not found (set SAIL_BIN)' }
    }
    let payload = body
    if (typeof body.path === 'string') {
      try {
        payload = { ...body, path: toWorkspacePath(body.path, cwd) }
      } catch (e) {
        return {
          ok: false,
          data: null,
          error: e instanceof Error ? e.message : String(e),
        }
      }
    }
    const r = spawnSync(
      sailBin,
      ['op', serviceMethod, '--json', JSON.stringify(payload)],
      { cwd, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 },
    )
    if (r.error) {
      return { ok: false, data: null, error: r.error.message }
    }
    const stdout = r.stdout ?? ''
    const stderr = r.stderr ?? ''
    try {
      const data = parseJsonStdout(stdout)
      return { ok: r.status === 0, data }
    } catch (e) {
      return {
        ok: false,
        data: null,
        error:
          (e instanceof Error ? e.message : String(e)) +
          (stderr ? `; stderr: ${stderr.slice(0, 400)}` : ''),
      }
    }
  }

  return {
    sailBin,
    async check(path: string): Promise<CheckResult> {
      const r = runOp('SailAuthoring/Check', { path, fix: 'off', actor: 'sail-eval' })
      if (!r.data || typeof r.data !== 'object') {
        return {
          ok: false,
          diagnostics: [],
          autoFixed: [],
          error: r.error ?? 'empty check response',
        }
      }
      const data = r.data as Record<string, unknown>
      const diagnostics = (Array.isArray(data.diagnostics) ? data.diagnostics : []) as Diagnostic[]
      const autoFixed = (Array.isArray(data.autoFixed) ? data.autoFixed : []) as Array<
        Record<string, unknown>
      >
      const ok =
        typeof data.ok === 'boolean'
          ? data.ok
          : !diagnostics.some((d) => d.severity === 'error')
      return { ok, diagnostics, autoFixed, raw: data, error: r.error }
    },
    async inspectTree(path: string): Promise<TreeResult> {
      const r = runOp('SailInspect/Tree', { path })
      if (!r.data || typeof r.data !== 'object') {
        return { ok: false, nodes: [], error: r.error ?? 'empty tree response' }
      }
      const data = r.data as Record<string, unknown>
      const nodes = (Array.isArray(data.nodes) ? data.nodes : []) as TreeNode[]
      return { ok: r.ok && !r.error, nodes, raw: data, error: r.error }
    },
    async layout(path: string): Promise<LayoutResult> {
      if (!sailBin || !existsSync(sailBin)) {
        return { ok: true, findings: [], error: 'sail binary not found — layout skipped' }
      }
      let rel: string
      try {
        rel = toWorkspacePath(path, cwd)
      } catch (e) {
        return {
          ok: true,
          findings: [],
          error: e instanceof Error ? e.message : String(e),
        }
      }
      const r = spawnSync(
        sailBin,
        ['layout', rel, '--json'],
        { cwd, encoding: 'utf8', maxBuffer: 8 * 1024 * 1024 },
      )
      if (r.error) {
        return { ok: true, findings: [], error: r.error.message }
      }
      try {
        const data = parseJsonStdout(r.stdout ?? '')
        if (Array.isArray(data)) {
          return { ok: true, findings: data as Array<Record<string, unknown>> }
        }
        if (data && typeof data === 'object') {
          const obj = data as Record<string, unknown>
          const findings = (Array.isArray(obj.findings)
            ? obj.findings
            : Array.isArray(obj.diagnostics)
              ? obj.diagnostics
              : []) as Array<Record<string, unknown>>
          return { ok: true, findings }
        }
        return { ok: true, findings: [] }
      } catch (e) {
        // layout is best-effort; don't fail the gate chain on parse noise
        return {
          ok: true,
          findings: [],
          error: e instanceof Error ? e.message : String(e),
        }
      }
    },
  }
}
