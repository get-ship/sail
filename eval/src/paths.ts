import { existsSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))

/** eval/ package root. */
export function evalRoot(): string {
  return resolve(HERE, '..')
}

/** Sail monorepo root (parent of eval/). */
export function repoRoot(): string {
  return resolve(evalRoot(), '..')
}

export function tasksDir(): string {
  return join(evalRoot(), 'tasks')
}

/** Context-profile arms (EVAL-1): eval/arms/<name>/arm.jsonc */
export function armsDir(): string {
  return join(evalRoot(), 'arms')
}

export function runsDir(): string {
  return join(evalRoot(), 'runs')
}

export function fixturesDir(): string {
  return join(evalRoot(), 'fixtures')
}

export function resultsPath(): string {
  return join(evalRoot(), 'RESULTS.md')
}

export function beautyRubricPath(): string {
  return join(evalRoot(), 'beauty-rubric.md')
}

/**
 * Validate a task slug: lowercase alphanumerics + hyphens only.
 * Prevents path traversal when slug is interpolated into filesystem paths.
 */
export function assertSafeSlug(slug: string): string {
  if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(slug)) {
    throw new Error(
      `invalid task slug "${slug}": must match /^[a-z0-9]+(?:-[a-z0-9]+)*$/`,
    )
  }
  return slug
}

/** Resolve sail binary: SAIL_BIN env, then target/{release,debug}/sail. */
export function findSailBin(root = repoRoot()): string | null {
  if (process.env.SAIL_BIN && existsSync(process.env.SAIL_BIN)) {
    return process.env.SAIL_BIN
  }
  for (const rel of ['target/release/sail', 'target/debug/sail']) {
    const p = join(root, rel)
    if (existsSync(p)) return p
  }
  return null
}
