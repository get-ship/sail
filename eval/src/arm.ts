/**
 * Context profiles (arms) — controlled experiment surfaces for EVAL-1.
 * Each arm is a scaffold + context manifest declaring exactly what the agent gets.
 * Non-sail arms MUST NOT leak sail artifacts (asserted by tests + materialize).
 */

import {
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  statSync,
  writeFileSync,
  cpSync,
} from 'node:fs'
import { dirname, join, relative } from 'node:path'
import { parseJsonc } from './jsonc.ts'
import { armsDir, assertSafeSlug, repoRoot } from './paths.ts'
import {
  injectSkillFile,
  scaffoldWorkdir,
  type ScaffoldKind,
} from './scaffold.ts'
import type { TaskGrading } from './types.ts'

export type ArmId = string

export interface ArmContext {
  /** When false, workdir must contain zero sail artifacts. */
  sailArtifacts: boolean
  skillFile: boolean
  mcp: boolean
  sailOnPath: boolean
}

export interface ArmDef {
  id: ArmId
  scaffold: ScaffoldKind
  context: ArmContext
  grading: TaskGrading
}

export interface LoadedArm {
  id: ArmId
  dir: string
  def: ArmDef
}

const DEFAULT_CONTEXT: ArmContext = {
  sailArtifacts: false,
  skillFile: false,
  mcp: false,
  sailOnPath: false,
}

/** Sail artifact patterns that must be absent on non-sail arms. */
export const SAIL_ARTIFACT_PATTERNS = {
  /** File extensions / names that are sail product surface. */
  extensions: ['.sail'] as const,
  /** Basename markers (skill, mcp, sail config). */
  basenames: [
    'SAIL_SKILL.md',
    'sail.config.jsonc',
    'sail.config.json',
    '.mcp.json',
    'author-surface.json',
  ] as const,
  /** Path segments that indicate sail authoring skill injection. */
  pathSegments: ['.agents/skills/sail', 'skills/sail-authoring'] as const,
  /** Content markers in free-form files (MCP config, package scripts). */
  contentMarkers: [
    /"sail"\s*:\s*\{/,
    /mcpServers[\s\S]*sail/i,
    /@ship\/sail/,
    /sail-mcp/,
  ] as const,
}

export function listArms(root = armsDir()): string[] {
  if (!existsSync(root)) return []
  return readdirSync(root)
    .filter((name) => {
      try {
        return statSync(join(root, name)).isDirectory()
      } catch {
        return false
      }
    })
    .filter((name) => existsSync(join(root, name, 'arm.jsonc')))
    .sort()
}

export function loadArm(id: string, root = armsDir()): LoadedArm {
  const safe = assertSafeSlug(id)
  const dir = join(root, safe)
  const path = join(dir, 'arm.jsonc')
  if (!existsSync(path)) {
    throw new Error(`arm not found: ${safe} (expected ${path})`)
  }
  const raw = parseJsonc(readFileSync(path, 'utf8')) as Partial<ArmDef> & {
    id?: string
    scaffold?: string
    context?: Partial<ArmContext>
    grading?: TaskGrading
  }
  if (raw.id && raw.id !== safe) {
    throw new Error(`arm id mismatch: dir=${safe} arm.jsonc id=${raw.id}`)
  }
  const scaffold = (raw.scaffold ?? 'blank') as ScaffoldKind
  if (!isScaffoldKind(scaffold)) {
    throw new Error(`arm ${safe}: invalid scaffold ${raw.scaffold}`)
  }
  const ctx = { ...DEFAULT_CONTEXT, ...(raw.context ?? {}) }
  // Consistency: sailArtifacts false forces skill/mcp/sailOnPath off.
  if (!ctx.sailArtifacts) {
    ctx.skillFile = false
    ctx.mcp = false
    ctx.sailOnPath = false
  }
  const def: ArmDef = {
    id: safe,
    scaffold,
    context: ctx,
    grading: raw.grading ?? defaultGradingFor(ctx.sailArtifacts),
  }
  return { id: safe, dir, def }
}

function isScaffoldKind(s: string): s is ScaffoldKind {
  return (
    s === 'blank' ||
    s === 'kitchen-sink' ||
    s === 'react-tw' ||
    s === 'vanilla'
  )
}

function defaultGradingFor(sailArtifacts: boolean): TaskGrading {
  if (sailArtifacts) {
    return {
      compile: true,
      vocab_share: true,
      gates: true,
      behavior: true,
      beauty: true,
    }
  }
  return {
    compile: false,
    vocab_share: false,
    gates: false,
    behavior: true,
    beauty: true,
  }
}

export function loadArms(ids?: string[], root = armsDir()): LoadedArm[] {
  const all = ids?.length ? ids.map((id) => assertSafeSlug(id)) : listArms(root)
  return all.map((id) => loadArm(id, root))
}

/**
 * Materialize arm context into workdir from the arm template + scaffold.
 * Returns relative paths written. Asserts the leak invariant for non-sail arms.
 */
export function materializeArm(workdir: string, arm: LoadedArm): string[] {
  mkdirSync(workdir, { recursive: true })
  const files: string[] = []

  // Optional arm-local template overlay (eval/arms/<id>/template/**).
  const templateDir = join(arm.dir, 'template')
  if (existsSync(templateDir)) {
    files.push(...copyTemplate(templateDir, workdir))
  }

  // Scaffold fills the rest (page, package.json, etc.).
  files.push(...scaffoldWorkdir(workdir, arm.def.scaffold))

  if (arm.def.context.skillFile) {
    const skill = injectSkillFile(workdir)
    if (skill) files.push(skill)
    // Authoring skill path used by real agent hosts.
    const skillDir = join(workdir, '.agents', 'skills', 'sail-authoring')
    mkdirSync(skillDir, { recursive: true })
    const skillPath = join(skillDir, 'SKILL.md')
    if (!existsSync(skillPath) && existsSync(join(workdir, 'SAIL_SKILL.md'))) {
      cpSync(join(workdir, 'SAIL_SKILL.md'), skillPath)
      files.push('.agents/skills/sail-authoring/SKILL.md')
    }
  }

  if (arm.def.context.mcp) {
    const mcp = buildSailMcpConfig()
    writeFileSync(join(workdir, '.mcp.json'), mcp)
    files.push('.mcp.json')
  }

  if (arm.def.context.sailOnPath) {
    // Document + leave a non-secret hint. Paced runner also injects SAIL_BIN
    // and prepends the bin dir to PATH for the omp child (see pace.ts).
    writeFileSync(
      join(workdir, '.sail-path-hint'),
      [
        'Sail binary on PATH for this arm:',
        '- Paced/omp runs set SAIL_BIN and prepend the sail bin dir to PATH when findSailBin() resolves.',
        '- Override with SAIL_BIN env if needed (target/debug|release/sail).',
        '',
      ].join('\n'),
    )
    files.push('.sail-path-hint')
  }

  // Enforce profile: non-sail arms must be clean.
  if (!arm.def.context.sailArtifacts) {
    const leaks = findSailArtifactLeaks(workdir)
    if (leaks.length) {
      throw new Error(
        `arm ${arm.id} leaked sail artifacts after materialize: ${leaks.join(', ')}`,
      )
    }
  }

  return files
}

function copyTemplate(srcDir: string, destDir: string): string[] {
  const written: string[] = []
  const walk = (dir: string) => {
    for (const name of readdirSync(dir, { withFileTypes: true })) {
      if (name.name === 'node_modules' || name.name === '.git') continue
      const from = join(dir, name.name)
      const rel = relative(srcDir, from)
      const to = join(destDir, rel)
      if (name.isDirectory()) {
        mkdirSync(to, { recursive: true })
        walk(from)
      } else if (name.isFile()) {
        mkdirSync(dirname(to), { recursive: true })
        cpSync(from, to)
        written.push(rel)
      }
    }
  }
  walk(srcDir)
  return written
}

function buildSailMcpConfig(): string {
  // Relative launch — no absolute operator paths.
  const sailMcp = join(repoRoot(), 'target/debug/sail-mcp')
  const cfg = {
    mcpServers: {
      sail: {
        command: existsSync(sailMcp) ? sailMcp : 'sail-mcp',
        args: [] as string[],
      },
    },
  }
  return JSON.stringify(cfg, null, 2) + '\n'
}

/**
 * Scan workdir for sail product artifacts. Returns relative paths / descriptions.
 * Used by leak-assertion tests and materialize guard.
 */
export function findSailArtifactLeaks(workdir: string): string[] {
  const hits: string[] = []
  if (!existsSync(workdir)) return hits

  const walk = (dir: string) => {
    let entries
    try {
      entries = readdirSync(dir, { withFileTypes: true })
    } catch {
      return
    }
    for (const ent of entries) {
      if (ent.name === 'node_modules' || ent.name === '.git') continue
      const abs = join(dir, ent.name)
      const rel = relative(workdir, abs)
      if (ent.isDirectory()) {
        for (const seg of SAIL_ARTIFACT_PATTERNS.pathSegments) {
          if (rel.includes(seg) || rel.replace(/\\/g, '/').includes(seg)) {
            hits.push(rel)
          }
        }
        walk(abs)
        continue
      }
      if (!ent.isFile()) continue

      for (const ext of SAIL_ARTIFACT_PATTERNS.extensions) {
        if (ent.name.endsWith(ext)) hits.push(rel)
      }
      for (const base of SAIL_ARTIFACT_PATTERNS.basenames) {
        if (ent.name === base) hits.push(rel)
      }

      // Content scan for MCP / package hints (text files only).
      if (/\.(jsonc?|md|js|jsx|ts|tsx|html|css|txt)$/i.test(ent.name) || ent.name.startsWith('.')) {
        let text: string
        try {
          text = readFileSync(abs, 'utf8')
        } catch {
          continue
        }
        // Cap scan size
        if (text.length > 200_000) text = text.slice(0, 200_000)
        for (const re of SAIL_ARTIFACT_PATTERNS.contentMarkers) {
          if (re.test(text)) {
            hits.push(`${rel} (content:${re.source.slice(0, 40)})`)
            break
          }
        }
      }
    }
  }
  walk(workdir)
  return [...new Set(hits)]
}

/** Merge task grading with arm grading: arm disables sail legs when set false. */
export function gradingForArm(
  taskGrading: TaskGrading,
  arm: ArmDef,
): TaskGrading {
  const a = arm.grading
  return {
    compile: a.compile === false ? false : taskGrading.compile,
    vocab_share: a.vocab_share === false ? false : taskGrading.vocab_share,
    gates: a.gates === false ? false : taskGrading.gates,
    behavior: a.behavior === false ? false : taskGrading.behavior,
    beauty: a.beauty === false ? false : taskGrading.beauty,
  }
}
