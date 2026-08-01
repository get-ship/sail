/**
 * Load Task / Profile / Toolchain from bench/ standards.
 */

import { spawnSync } from 'node:child_process'
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs'
import { join } from 'node:path'
import { contentHash } from './hash.ts'
import { ARMS, type ArmConfig, type ProfileDef, type TaskDef, type ToolchainInfo } from './types.ts'
import { labRoot, repoRoot, resolveSailBin, standardsDir } from '../naming/paths.ts'

/** Strip // line comments and trailing commas — enough for our jsonc files. */
export function parseJsonc<T = unknown>(raw: string): T {
  const lines = raw.split(/\r?\n/).map((ln) => {
    const trimmed = ln.trim()
    if (trimmed.startsWith('//')) return ''
    // strip trailing // comments not inside strings (best-effort)
    let inStr = false
    let out = ''
    for (let i = 0; i < ln.length; i++) {
      const c = ln[i]!
      if (c === '"' && ln[i - 1] !== '\\') inStr = !inStr
      if (!inStr && c === '/' && ln[i + 1] === '/') break
      out += c
    }
    return out
  })
  let text = lines.join('\n')
  text = text.replace(/,\s*([}\]])/g, '$1')
  return JSON.parse(text) as T
}

export function loadTask(taskId: string, standards = standardsDir()): TaskDef {
  const path = join(standards, 'tasks', taskId, 'task.jsonc')
  if (!existsSync(path)) {
    throw new Error(`unknown task '${taskId}': missing ${path}`)
  }
  const def = parseJsonc<TaskDef>(readFileSync(path, 'utf8'))
  if (!def.prompt) throw new Error(`task '${taskId}' missing prompt`)
  return { ...def, id: def.id ?? taskId }
}

export function loadProfile(profileId: string, standards = standardsDir()): { def: ProfileDef; hash: string } {
  const path = join(standards, 'profiles', `${profileId}.jsonc`)
  if (!existsSync(path)) {
    throw new Error(`unknown profile '${profileId}': missing ${path}`)
  }
  const def = parseJsonc<ProfileDef>(readFileSync(path, 'utf8'))
  def.id = def.id ?? profileId
  const hash = contentHash(def)
  return { def, hash }
}

export function resolveArm(armId: string): ArmConfig {
  const a = ARMS[armId]
  if (!a) {
    throw new Error(`unknown arm '${armId}' (known: ${Object.keys(ARMS).join(', ')})`)
  }
  return a
}

/** Payload directory for an arm (lab-kit payload, then lab payload). */
export function resolvePayloadDir(arm: ArmConfig, standards = standardsDir(), lab = labRoot()): string {
  const candidates = [
    join(standards, 'lab-kit/payload', arm.payload),
    join(lab, 'payload', arm.payload),
  ]
  for (const c of candidates) {
    if (existsSync(c)) return c
  }
  throw new Error(`payload for arm '${arm.id}' not found (looked in ${candidates.join(', ')})`)
}

/**
 * Resolve toolchain alias.
 * - `current` → latest under $SAIL_LAB/toolchains, else sail from repo build
 * - otherwise treat as stamp under toolchains/
 */
export function resolveToolchain(
  id: string,
  repo = repoRoot(),
  lab = labRoot(),
): ToolchainInfo {
  const now = new Date().toISOString()
  if (id === 'current') {
    const stamp = latestToolchainStamp(lab) ?? gitToolchainStamp(repo)
    const binDir = stamp ? join(lab, 'toolchains', stamp, 'bin', 'sail') : null
    const sailBin = resolveSailBin(repo, process.env, existsSync(binDir!) ? binDir : null)
    return {
      id: 'current',
      stamp: stamp ?? 'current-nostamp',
      sailBin,
      createdAt: now,
    }
  }

  const tcSail = join(lab, 'toolchains', id, 'bin', 'sail')
  if (existsSync(tcSail)) {
    return { id, stamp: id, sailBin: tcSail, createdAt: now }
  }
  // Allow stamp-as-id even without toolchain dir (use repo sail).
  return {
    id,
    stamp: id,
    sailBin: resolveSailBin(repo),
    createdAt: now,
  }
}

function latestToolchainStamp(lab: string): string | null {
  const root = join(lab, 'toolchains')
  if (!existsSync(root)) return null
  const dirs = readdirSync(root)
    .map((name) => ({ name, mtime: statSync(join(root, name)).mtimeMs }))
    .filter((d) => statSync(join(root, d.name)).isDirectory())
    .sort((a, b) => b.mtime - a.mtime)
  return dirs[0]?.name ?? null
}

function gitToolchainStamp(repo: string): string {
  try {
    const sha = spawnSync('git', ['rev-parse', '--short=12', 'HEAD'], {
      cwd: repo,
      encoding: 'utf8',
    })
    const short = sha.status === 0 ? sha.stdout.trim() : 'nogit'
    let dirty = ''
    const d1 = spawnSync('git', ['diff', '--quiet'], { cwd: repo })
    const d2 = spawnSync('git', ['diff', '--cached', '--quiet'], { cwd: repo })
    if (d1.status !== 0 || d2.status !== 0) dirty = '-dirty'
    return `${short}${dirty}-r0.1.0`
  } catch {
    return 'nogit-r0.1.0'
  }
}
