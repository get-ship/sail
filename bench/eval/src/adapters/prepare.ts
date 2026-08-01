/**
 * prepare phase — materialize sandbox from arm payload + task overlay.
 * Does not run the agent. Never vendors node_modules (symlink shared if present).
 */

import {
  cpSync,
  existsSync,
  mkdirSync,
  readdirSync,
  statSync,
  symlinkSync,
  writeFileSync,
} from 'node:fs'
import { dirname, join, relative, resolve } from 'node:path'
// resolve used for copyTree self-copy guard + lab root walk
import {
  CONTRACT_VERSION,
  type PhaseResult,
  type RunManifest,
  emptyPhaseResult,
} from '../contract/types.ts'
import { fileSha256 } from '../config/hash.ts'

function copyTree(src: string, dst: string, written: string[], base: string): void {
  const srcR = resolve(src)
  const dstR = resolve(dst)
  if (srcR === dstR) return
  mkdirSync(dst, { recursive: true })
  for (const name of readdirSync(src)) {
    if (name === 'node_modules' || name === '.git') continue
    const from = join(src, name)
    const to = join(dst, name)
    // Never copy a tree into itself (e.g. src nested under growing dst).
    if (resolve(to).startsWith(srcR + '/') || resolve(from) === dstR) continue
    const st = statSync(from)
    if (st.isDirectory()) {
      copyTree(from, to, written, base)
    } else {
      mkdirSync(dirname(to), { recursive: true })
      cpSync(from, to)
      written.push(relative(base, to))
    }
  }
}

function linkSharedDeps(sandbox: string, standardsDir: string, labHint: string, written: string[]): string[] {
  const linked: string[] = []
  // react-base: prefer lab shared node_modules
  const candidates = [
    join(labHint, 'shared/react-base/node_modules'),
    join(standardsDir, 'lab-kit/payload/react-base/node_modules'),
  ]
  for (const nm of candidates) {
    if (!existsSync(nm)) continue
    const target = join(sandbox, 'node_modules')
    if (existsSync(target)) break
    try {
      symlinkSync(nm, target, 'dir')
      linked.push(nm)
      written.push('node_modules')
    } catch {
      /* ignore — gate/render may fail with explicit reason later */
    }
    break
  }
  return linked
}

export async function preparePhase(manifest: RunManifest): Promise<PhaseResult> {
  const t0 = Date.now()
  const result = emptyPhaseResult('prepare')
  result.contract_version = CONTRACT_VERSION

  try {
    const sandbox = manifest.sandbox_dir
    mkdirSync(sandbox, { recursive: true })
    mkdirSync(manifest.run_dir, { recursive: true })

    const written: string[] = []
    if (!existsSync(manifest.payload_dir)) {
      result.ok = false
      result.verdict = 'fail'
      result.error = {
        code: 'PAYLOAD_MISSING',
        message: `payload_dir not found: ${manifest.payload_dir}`,
      }
      result.wall_ms = Date.now() - t0
      return result
    }

    copyTree(manifest.payload_dir, sandbox, written, sandbox)

    // Task overlay (e.g. mini-app-routing)
    const overlay = join(manifest.standards_dir, 'tasks', manifest.task_id, 'overlay')
    if (existsSync(overlay)) {
      copyTree(overlay, sandbox, written, sandbox)
    }

    // AGENTS.md from profile
    const agentsRel = (manifest.profile.agents_md as string | undefined) ?? 'agents/agents-v2.md'
    const agentsSrc = join(manifest.standards_dir, agentsRel)
    if (existsSync(agentsSrc)) {
      const dest = join(sandbox, 'AGENTS.md')
      cpSync(agentsSrc, dest)
      written.push('AGENTS.md')
    }

    // Prompt file for agent
    const prompt = (manifest.task.prompt as string) ?? ''
    writeFileSync(join(sandbox, '.bench-prompt.txt'), prompt + '\n')
    written.push('.bench-prompt.txt')

    // Seed mode: leave scaffold as-is; optionally mark
    const seed = Boolean(manifest.options?.seed)
    if (seed) {
      writeFileSync(
        join(sandbox, '.bench-seed'),
        JSON.stringify({ seed: true, task: manifest.task_id, arm: manifest.arm }) + '\n',
      )
      written.push('.bench-seed')
    }

    // Lab root: explicit env on manifest, SAIL_LAB, or walk up from
    // runs/<tc>/<task>/<model>/<id>/ → four parents.
    const labRootGuess =
      manifest.env.SAIL_LAB ||
      process.env.SAIL_LAB ||
      resolve(join(manifest.run_dir, '..', '..', '..', '..'))
    const shared = linkSharedDeps(sandbox, manifest.standards_dir, labRootGuess, written)

    // Meta for collect
    const meta = {
      run_id: manifest.run_id,
      batch_id: manifest.batch_id,
      task_id: manifest.task_id,
      arm: manifest.arm,
      model_slug: manifest.model_slug,
      toolchain: manifest.toolchain,
      profile_hash: manifest.profile_hash,
      seed,
      prepared_at: new Date().toISOString(),
    }
    writeFileSync(join(manifest.run_dir, 'meta.json'), JSON.stringify(meta, null, 2) + '\n')

    result.ok = true
    result.verdict = 'pass'
    result.detail = {
      sandbox_dir: sandbox,
      files_written: written,
      shared_deps: shared,
      seed,
    }
    result.artifacts = [
      {
        name: 'meta.json',
        rel_path: 'meta.json',
        blocking: false,
        status: 'present',
        sha256: fileSha256(JSON.stringify(meta)),
        bytes: JSON.stringify(meta).length,
      },
      {
        name: 'sandbox',
        rel_path: manifest.paths.source,
        blocking: true,
        status: 'present',
        reason: null,
      },
    ]
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    result.ok = false
    result.verdict = 'error'
    result.error = { code: 'PREPARE', message: msg }
  }

  result.wall_ms = Date.now() - t0
  return result
}
