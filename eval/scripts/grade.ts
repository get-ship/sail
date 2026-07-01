#!/usr/bin/env bun
// grade.ts — score each lane's views/create-space/ output.
//
// Run by the `sui-eval` skill after `dispatch-eval.sh` lanes finish. We
// discover lane worktrees by `-haiku`/`-sonnet` suffix (no daemon, no
// orchestrator), then for each lane:
//
//   1. compile every .sui file under the view dir via sui-compiler
//   2. lint via sui-lint
//   3. structural pass: required signals + control-flow + token-only CSS +
//      pure-expression templates (no capabilities/statements in `.sui`) +
//      primitive-adherence (composes <sui-*> + tokens vs hand-rolls layout)
//   4. behavioral pass (when SUI_EVAL_BEHAVIOR_SPEC is set): scenario runner
//      in jsdom via behavior-grade.ts
//   5. emit one JSON row per lane → .artifacts/sui-eval-runs/<run-id>.jsonl
//
// Discovery is best-effort and fail-loud — if a lane's worktree isn't
// reachable the row records the discovery gap rather than silently passing.

import { execSync, spawnSync } from 'node:child_process'
import { readdirSync, statSync, existsSync, readFileSync, mkdirSync, writeFileSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { scanTemplatePurity } from './template-purity'
import { gradePrimitiveAdherence, type PerFileAdherence } from './primitive-adherence'

const RUN_ID = process.env.SHIP_PIPELINE_RUN_ID ?? `local-${process.env.SHIP_ACTOR_ID ?? 'unknown'}`
const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url))

// The compiler binary. sui-compiler moved to the external sui repo
// (`sui/crates/sui/compiler`) post-carve — the
// orchestrating repo (ship) no longer carries it. A lane would have to
// `cargo run` against the external sui checkout (slow, racy against
// parallel compiler work). Use the prebuilt binary from the external
// sui's target dir when present, then fall back to cargo there too.
function resolveCompiler(): { cmd: string; preArgs: string[] } {
  const envBin = process.env.SUI_COMPILER
  if (envBin && existsSync(envBin)) return { cmd: envBin, preArgs: [] }
  // Prebuilt binary. This repo now OWNS both the eval and the compiler, so the
  // in-repo `target/` wins; the legacy ship-sibling `sui/target/` stays as a
  // fallback for a ship-orchestrated run.
  for (const bin of ['target/debug/sui-compiler', 'target/release/sui-compiler', 'sui/target/debug/sui-compiler']) {
    if (existsSync(bin)) return { cmd: bin, preArgs: [] }
  }
  // cargo fallback (rare — the eval prefers the prebuilt binary). In-repo
  // manifest when the sui-compiler crate lives here, else the sibling checkout.
  const manifest = existsSync('crates/sui/compiler/Cargo.toml') ? 'Cargo.toml' : 'sui/Cargo.toml'
  return { cmd: 'cargo', preArgs: ['run', '--manifest-path', manifest, '-p', 'sui-compiler', '--'] }
}
const COMPILER = resolveCompiler()

// Optional behavioral dimension: a *.behavior.jsonc run by behavior-grade.ts
// (scenario-runner in jsdom). When set, behavior joins the composite — it is
// the "does it do what was asked" axis, so it carries the largest weight.
const BEHAVIOR_SPEC = process.env.SUI_EVAL_BEHAVIOR_SPEC

// The view the lanes were asked to build + the structural rubric to score it
// against. Both are env-overridable so the cheap smoke pipeline (haiku →
// views/smoke-counter) can drive the SAME grade → record → view chain as the
// live create-space eval without rewriting the scorer. Defaults preserve the
// create-space rubric.
const VIEW_PATH = process.env.SUI_EVAL_VIEW_PATH ?? 'views/create-space'
const REQUIRED_TEMPLATE_FRAGMENTS = process.env.SUI_EVAL_REQUIRED_FRAGMENTS
  ? process.env.SUI_EVAL_REQUIRED_FRAGMENTS.split(',').map((s) => s.trim()).filter(Boolean)
  : [
      // Step 0 / welcome
      '{#if', '{#match', '{:case',
      // Form / input
      'bind:value',
      // Composition
      'on:click',
    ]
const REQUIRED_MANIFEST_FIELDS = ['name', 'path', 'title', 'auth']
const FORBIDDEN_IN_CSS = /#[0-9a-fA-F]{3,8}|rgb\(|\b\d+px\b/g

interface BehaviorScore {
  total: number
  passed: number
  score: number
  results: { name: string; passed: boolean; failedStep?: number; error?: string }[]
}

interface LaneScore {
  laneId: string
  worktree: string
  compile: { passed: boolean; stderr: string }
  lint: { errors: number; warnings: number; output: string }
  structural: {
    manifestOk: boolean
    requiredFragments: { found: string[]; missing: string[] }
    cssTokenViolations: string[]
    templatePurityViolations: string[]
    // Composes primitives + tokens vs hand-rolls layout/values — see
    // primitive-adherence.ts. `primitiveAdherence` is the overall [0,1];
    // the per-file breakdown carries the detector counts.
    primitiveAdherence: number
    primitiveAdherenceByFile: PerFileAdherence[]
    viewFiles: string[]
  }
  behavior?: BehaviorScore
  composite: number
}

function findLaneWorktrees(): { laneId: string; path: string }[] {
  // Discover lane worktrees from git itself — robust against whatever base dir
  // dispatch.sh used (~/dev/ship-worktrees vs ~/.ship/worktrees differ per
  // machine). git knows every worktree; we match the eval-lane shape by its
  // `-haiku` / `-sonnet` suffix.
  const cwd = process.cwd()
  const lanes: { laneId: string; path: string }[] = []
  let listing = ''
  try {
    listing = execSync('git worktree list --porcelain', { encoding: 'utf8' })
  } catch {
    console.warn('grade: `git worktree list` failed')
  }
  // Lane worktrees are named `eval-<model>-<runId>`. Scope to THIS run when we
  // have a real run id (env-set) so we don't grade stale lanes from past runs;
  // fall back to a model-only match for ad-hoc manual grading.
  const scoped = /^\d{6,}/.test(RUN_ID)
    ? new RegExp(`-(haiku|sonnet|opus)-${RUN_ID}$`)
    : /-(haiku|sonnet|opus)$/
  for (const line of listing.split('\n')) {
    if (!line.startsWith('worktree ')) continue
    const p = line.slice('worktree '.length).trim()
    const m = p.match(scoped)
    if (!m) continue
    if (existsSync(join(p, VIEW_PATH))) lanes.push({ laneId: m[1]!, path: p })
  }
  if (lanes.length === 0) {
    console.warn('grade: discovered 0 lane worktrees via `git worktree list`')
    console.warn('grade: falling back to current cwd as a single anonymous lane')
    if (existsSync(join(cwd, VIEW_PATH))) lanes.push({ laneId: 'cwd', path: cwd })
  }
  return lanes
}

function scoreLane(laneId: string, worktree: string): LaneScore {
  const viewDir = join(worktree, VIEW_PATH)
  const viewFiles = collectViewFiles(viewDir)

  const compile = compileView(worktree, viewFiles)
  const lint = lintView(worktree)
  const structural = structuralCheck(worktree, viewFiles)
  const behavior = BEHAVIOR_SPEC ? behaviorCheck(worktree) : undefined

  // Composite weights (surfaced in the jsonl so the comparison view can
  // re-weight later). With a behavior spec, behavior is the largest axis —
  // "does it do what was asked" — and structure/lint shrink accordingly.
  // Without one, the legacy compile/lint/structural split applies.
  const compileScore = compile.passed ? 1 : 0
  const lintScore = lint.errors === 0
    ? Math.max(0, 1 - lint.warnings * 0.05)
    : 0
  const structuralScore = structuralFraction(structural)
  const composite = behavior
    ? 0.30 * compileScore + 0.10 * lintScore + 0.20 * structuralScore + 0.40 * behavior.score
    : 0.40 * compileScore + 0.20 * lintScore + 0.40 * structuralScore

  return { laneId, worktree, compile, lint, structural, ...(behavior ? { behavior } : {}), composite }
}

function behaviorCheck(worktree: string): BehaviorScore {
  // behavior-grade.ts prints one JSON object on stdout (a measurement, exit 0
  // even on failures). A crash or unparseable output scores 0 with the error
  // captured — never silently passing.
  const r = spawnSync('bun', [join(SCRIPT_DIR, 'behavior-grade.ts'), worktree, VIEW_PATH, BEHAVIOR_SPEC!], {
    encoding: 'utf8',
  })
  try {
    const parsed = JSON.parse(r.stdout ?? '') as BehaviorScore & { view: string }
    return { total: parsed.total, passed: parsed.passed, score: parsed.score, results: parsed.results }
  } catch {
    return {
      total: 0,
      passed: 0,
      score: 0,
      results: [{ name: 'behavior-grade run', passed: false, error: (r.stderr ?? 'no output').slice(0, 400) }],
    }
  }
}

function collectViewFiles(dir: string): string[] {
  const out: string[] = []
  if (!existsSync(dir)) return out
  function walk(d: string): void {
    for (const entry of readdirSync(d)) {
      const p = join(d, entry)
      if (statSync(p).isDirectory()) walk(p)
      else out.push(p)
    }
  }
  walk(dir)
  return out
}

function compileView(worktree: string, files: string[]): LaneScore['compile'] {
  const suiFiles = files.filter((p) => p.endsWith('.sui'))
  if (suiFiles.length === 0) return { passed: false, stderr: 'no .sui files emitted' }
  let stderr = ''
  for (const f of suiFiles) {
    const isComponent = f.includes('/_components/')
    const compName = isComponent ? deriveComponentName(f) : undefined
    const args = [...COMPILER.preArgs, 'build', f]
    if (compName) args.push('--component-name', compName)
    const r = spawnSync(COMPILER.cmd, args, { cwd: worktree, encoding: 'utf8' })
    if (r.status !== 0) {
      stderr += `[${f}] ${r.stderr ?? ''}\n`
    }
  }
  return { passed: stderr === '', stderr }
}

function deriveComponentName(suiPath: string): string | undefined {
  const m = suiPath.match(/_components\/([\w-]+)\/[\w-]+\.sui$/)
  return m?.[1]
}

function lintView(worktree: string): LaneScore['lint'] {
  // sui-lint is a [lib] crate with no [[bin]] target — it runs as a library
  // inside `sui-compiler build`'s check phase. The compile step IS the lint
  // step; parse errors + warnings from the compiler's output.
  const r = spawnSync(
    COMPILER.cmd,
    [...COMPILER.preArgs, 'build', `${VIEW_PATH}/page.sui`],
    { cwd: worktree, encoding: 'utf8' },
  )
  const text = (r.stdout ?? '') + (r.stderr ?? '')
  const errors = (text.match(/\berror:/g) ?? []).length
  const warnings = (text.match(/\bwarning:/g) ?? []).length
  return { errors, warnings, output: text }
}

function structuralCheck(worktree: string, files: string[]): LaneScore['structural'] {
  const manifestPath = join(worktree, VIEW_PATH, 'manifest.jsonc')
  let manifestOk = false
  if (existsSync(manifestPath)) {
    try {
      const raw = readFileSync(manifestPath, 'utf8').replace(/\/\/.*$/gm, '')
      const m = JSON.parse(raw) as Record<string, unknown>
      manifestOk = REQUIRED_MANIFEST_FIELDS.every((k) => k in m)
    } catch { manifestOk = false }
  }

  const templateText = files
    .filter((p) => p.endsWith('.sui'))
    .map((p) => readFileSync(p, 'utf8'))
    .join('\n')
  const found = REQUIRED_TEMPLATE_FRAGMENTS.filter((s) => templateText.includes(s))
  const missing = REQUIRED_TEMPLATE_FRAGMENTS.filter((s) => !templateText.includes(s))

  const cssText = files
    .filter((p) => p.endsWith('.css'))
    .map((p) => readFileSync(p, 'utf8'))
    .join('\n')
    .replace(/\/\*[\s\S]*?\*\//g, '')
  const cssTokenViolations = Array.from(new Set(cssText.match(FORBIDDEN_IN_CSS) ?? []))

  // A capability (fetch/IO/non-determinism/non-owned mutation) or a
  // statement/declaration in a `.sui` template is a FAIL — it must promote to
  // controller.ts. See template-purity.ts + docs/ship-sui authoring-model.
  const templatePurityViolations = scanTemplatePurity(templateText)

  // Composition axis: does each `.sui` reach for <sui-*> primitives + tokens,
  // or hand-roll layout (ad-hoc display:grid/flex) and literal values? Scored
  // per-file so one bad file doesn't hide behind good ones. See
  // primitive-adherence.ts.
  const adherence = gradePrimitiveAdherence(
    files
      .filter((p) => p.endsWith('.sui'))
      .map((p) => ({ file: p.replace(worktree + '/', ''), text: readFileSync(p, 'utf8') })),
  )

  return {
    manifestOk,
    requiredFragments: { found, missing },
    cssTokenViolations,
    templatePurityViolations,
    primitiveAdherence: adherence.overall,
    primitiveAdherenceByFile: adherence.perFile,
    viewFiles: files.map((p) => p.replace(worktree + '/', '')),
  }
}

function structuralFraction(s: LaneScore['structural']): number {
  if (s.viewFiles.length === 0) return 0
  const reqTotal = REQUIRED_TEMPLATE_FRAGMENTS.length
  const reqHits = s.requiredFragments.found.length
  const cssClean = s.cssTokenViolations.length === 0 ? 1 : 0
  const manifest = s.manifestOk ? 1 : 0
  // Pure-expression template gate: any capability/statement in a `.sui` file
  // floors this slice (the line must hold for the structure to count as sound).
  const templatePure = s.templatePurityViolations.length === 0 ? 1 : 0
  // Composition axis is already a continuous [0,1] (primitive-adherence.ts).
  // Weights: template fragments 0.35, css purity 0.15, manifest 0.15,
  // template purity 0.15, primitive adherence 0.20.
  return (
    (reqHits / reqTotal) * 0.35 +
    cssClean * 0.15 +
    manifest * 0.15 +
    templatePure * 0.15 +
    s.primitiveAdherence * 0.2
  )
}

// --- main -----------------------------------------------------------------

const lanes = findLaneWorktrees()
console.log(`grade: scoring ${lanes.length} lane(s) for run ${RUN_ID}`)
const results = lanes.map(({ laneId, path }) => scoreLane(laneId, path))

// Written cwd-relative — the watcher runs grade from the orchestrating repo
// (this worktree), which is where views/eval reads .artifacts from. Run grade
// from the repo root, not from inside a lane worktree.
const outDir = resolve('.artifacts/sui-eval-runs')
mkdirSync(outDir, { recursive: true })
const outFile = join(outDir, `${RUN_ID}.jsonl`)
const lines = results.map((r) => JSON.stringify({ runId: RUN_ID, ...r })).join('\n') + '\n'
writeFileSync(outFile, lines)

console.log(`grade: wrote ${results.length} row(s) → ${outFile}`)
for (const r of results) {
  const tag = r.compile.passed ? 'ok' : 'fail'
  const beh = r.behavior ? ` behavior=${r.behavior.passed}/${r.behavior.total}` : ''
  const impure = r.structural.templatePurityViolations.length
  const pure = impure === 0 ? '' : ` impure=${impure}!`
  const adh = ` adh=${r.structural.primitiveAdherence.toFixed(2)}`
  console.log(`  ${r.laneId.padEnd(8)} composite=${r.composite.toFixed(2)} compile=${tag} lint=${r.lint.errors}e/${r.lint.warnings}w${pure}${adh}${beh}`)
}

// Exit 0 even if a lane scored zero — `grade` is a measurement, not a gate.
// The `record` step that follows is what makes the data durable.
process.exit(0)
