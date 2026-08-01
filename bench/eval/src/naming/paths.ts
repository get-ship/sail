/**
 * Lab FS layout helpers.
 * runs/<toolchain>/<task>/<model>/<word-pair>/
 */

import { existsSync, mkdirSync, readdirSync } from 'node:fs'
import { homedir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))

/** Absolute path to the sail repo root (…/sail). */
export function repoRoot(from = HERE): string {
  // bench/eval/src/naming → up 4
  return resolve(from, '../../../..')
}

/** Absolute path to bench/ standards root. */
export function standardsDir(repo = repoRoot()): string {
  return join(repo, 'bench')
}

/** Lab data root (never inside the sail repo). */
export function labRoot(env: NodeJS.ProcessEnv = process.env): string {
  const raw = env.SAIL_LAB ?? join(homedir(), 'dev/sail-lab')
  return resolve(raw)
}

export interface RunPathParts {
  toolchain: string
  task: string
  model: string
  runId: string
}

/** runs/<toolchain>/<task>/<model>/<word-pair> relative to lab root. */
export function runDirRel(parts: RunPathParts): string {
  return join('runs', sanitizeSeg(parts.toolchain), sanitizeSeg(parts.task), sanitizeSeg(parts.model), sanitizeSeg(parts.runId))
}

export function runDirAbs(lab: string, parts: RunPathParts): string {
  return join(lab, runDirRel(parts))
}

export function reportsDir(lab: string, batchId: string): string {
  return join(lab, 'reports', sanitizeSeg(batchId))
}

export function quarantineDir(lab: string): string {
  return join(lab, 'quarantine')
}

export function storePath(lab: string): string {
  return join(lab, 'lab-bench.sqlite')
}

/** List existing word-pair ids under runs/<tc>/<task>/<model>/. */
export function listTakenRunIds(lab: string, toolchain: string, task: string, model: string): Set<string> {
  const parent = join(lab, 'runs', sanitizeSeg(toolchain), sanitizeSeg(task), sanitizeSeg(model))
  const taken = new Set<string>()
  if (!existsSync(parent)) return taken
  for (const name of readdirSync(parent)) {
    taken.add(name)
  }
  return taken
}

export function ensureDir(path: string): void {
  mkdirSync(path, { recursive: true })
}

function sanitizeSeg(s: string): string {
  return s
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9._-]+/g, '-')
    .replace(/^-+|-+$/g, '') || 'unnamed'
}

/** Resolve sail binary: SAIL_BIN → toolchain → release → debug. */
export function resolveSailBin(
  repo = repoRoot(),
  env: NodeJS.ProcessEnv = process.env,
  toolchainBin?: string | null,
): string | null {
  if (env.SAIL_BIN && existsSync(env.SAIL_BIN)) return env.SAIL_BIN
  if (toolchainBin && existsSync(toolchainBin)) return toolchainBin
  const release = join(repo, 'target/release/sail')
  if (existsSync(release)) return release
  const debug = join(repo, 'target/debug/sail')
  if (existsSync(debug)) return debug
  return null
}

/** Default chromium from playwright cache or CHROME_PATH. */
export function resolveChrome(env: NodeJS.ProcessEnv = process.env): string | null {
  if (env.CHROME_PATH && existsSync(env.CHROME_PATH)) return env.CHROME_PATH
  const home = homedir()
  const candidates = [
    join(home, '.cache/ms-playwright/chromium-1228/chrome-linux64/chrome'),
    join(home, '.cache/ms-playwright/chromium-1208/chrome-linux64/chrome'),
    join(home, '.cache/ms-playwright/chromium-1148/chrome-linux64/chrome'),
  ]
  for (const c of candidates) {
    if (existsSync(c)) return c
  }
  return null
}
