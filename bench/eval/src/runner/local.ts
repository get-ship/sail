/**
 * LocalRunner — sandbox dir, optional grok launch, adapter phases via RUN CONTRACT.
 */

import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { spawnSync } from 'node:child_process'
import { invokePhase, type InvokeOptions, type InvokeOutcome } from '../contract/invoke.ts'
import {
  CONTRACT_VERSION,
  emptyPhaseResult,
  type PhaseResult,
  type RunManifest,
} from '../contract/types.ts'
import {
  loadProfile,
  loadTask,
  resolveArm,
  resolvePayloadDir,
  resolveToolchain,
} from '../config/load.ts'
import { HARNESS_VER, type RunConfig } from '../config/types.ts'
import {
  ensureDir,
  labRoot,
  listTakenRunIds,
  reportsDir,
  repoRoot,
  runDirAbs,
  standardsDir,
  storePath,
} from '../naming/paths.ts'
import { allocateWordPair } from '../naming/word-pair.ts'
import { RunStore, type BatchRow, type RunRow } from '../store/db.ts'
import type {
  CollectBundle,
  ExecuteOutcome,
  RunHandle,
  RunOutcome,
  Runner,
} from './types.ts'
import { handlers } from '../adapters/index.ts'

export interface LocalRunnerOptions {
  lab?: string
  standards?: string
  repo?: string
  /**
   * Spawn adapters as subprocess (RUN CONTRACT fidelity).
   * Default **false** (in-process) — matches CLI default for speed.
   * Pass `true` or CLI `--subprocess` for real eval-adapter spawns.
   */
  subprocess?: boolean
  invoke?: InvokeOptions
  /** Test hook: replace live agent execute without PATH tricks. */
  executeOverride?: (run: RunHandle) => Promise<ExecuteOutcome>
}

function batchIdNow(purpose: string): string {
  const d = new Date()
  const pad = (n: number, w = 2) => String(n).padStart(w, '0')
  const ts = `${d.getUTCFullYear()}${pad(d.getUTCMonth() + 1)}${pad(d.getUTCDate())}-${pad(d.getUTCHours())}${pad(d.getUTCMinutes())}${pad(d.getUTCSeconds())}`
  const slug = purpose.replace(/[^a-z0-9]+/gi, '-').toLowerCase() || 'run'
  return `${ts}-${slug}`
}

export class LocalRunner implements Runner {
  readonly lab: string
  readonly standards: string
  readonly repo: string
  private invokeOpts: InvokeOptions
  private executeOverride?: (run: RunHandle) => Promise<ExecuteOutcome>

  constructor(opts: LocalRunnerOptions = {}) {
    this.lab = opts.lab ?? labRoot()
    this.standards = opts.standards ?? standardsDir(opts.repo ?? repoRoot())
    this.repo = opts.repo ?? repoRoot()
    // SSOT default: in-process (subprocess=false). Same as sail-bench CLI.
    const subprocess = opts.subprocess ?? false
    this.invokeOpts = opts.invoke ?? {
      inProcess: !subprocess,
      handlers: !subprocess ? handlers : undefined,
    }
    this.executeOverride = opts.executeOverride
  }

  openStore(): RunStore {
    return RunStore.open(storePath(this.lab))
  }

  /**
   * Full single-run lifecycle for phase-1 acceptance:
   * allocate → prepare → execute → gate → render → collect → store → manifest.
   */
  async runOne(config: RunConfig): Promise<RunOutcome> {
    const t0 = Date.now()
    const task = loadTask(config.taskId, this.standards)
    const { def: profile, hash: profileHash } = loadProfile(config.profileId, this.standards)
    const arm = resolveArm(config.arm)
    const payloadDir = resolvePayloadDir(arm, this.standards, this.lab)
    const tc = resolveToolchain(config.toolchain, this.repo, this.lab)

    const modelSlug = config.modelSlug
    const { runId, runDir } = allocateExclusiveRunDir(this.lab, tc.stamp, config.taskId, modelSlug)
    const sandboxDir = join(runDir, 'sandbox')
    ensureDir(sandboxDir)

    const batchId = batchIdNow(config.batchName ?? `${config.taskId}-${config.arm}`)
    const reportDir = reportsDir(this.lab, batchId)
    ensureDir(reportDir)

    const store = this.openStore()
    const ts = new Date().toISOString()

    const batchRow: BatchRow = {
      batch_id: batchId,
      ts,
      name: config.batchName ?? `n1-${config.taskId}-${config.arm}`,
      question: null,
      profile_hash: profileHash,
      harness_ver: HARNESS_VER,
      status: 'running',
      n: config.n ?? 1,
      tasks_json: JSON.stringify([config.taskId]),
      arms_json: JSON.stringify([config.arm]),
      constants_json: JSON.stringify({
        model_slug: modelSlug,
        toolchain: tc.stamp,
        seed: config.seed,
      }),
      report_dir: reportDir,
      manifest_json: null,
    }
    store.upsertBatch(batchRow)

    const manifest: RunManifest = {
      contract_version: CONTRACT_VERSION,
      run_id: runId,
      batch_id: batchId,
      task_id: config.taskId,
      arm: config.arm,
      model_slug: modelSlug,
      model: config.model,
      toolchain: tc.stamp,
      profile_hash: profileHash,
      harness_ver: HARNESS_VER,
      run_dir: runDir,
      sandbox_dir: sandboxDir,
      standards_dir: this.standards,
      payload_dir: payloadDir,
      task: task as unknown as Record<string, unknown>,
      profile: profile as unknown as Record<string, unknown>,
      paths: {
        events: 'events.jsonl',
        manifest: 'manifest.json',
        gate: 'gate.json',
        render: 'render.png',
        source: 'sandbox',
      },
      env: {
        SAIL_LAB: this.lab,
        ...(tc.sailBin ? { SAIL_BIN: tc.sailBin } : {}),
      },
      limits: { phase_timeout_ms: 600_000 },
      phase: 'prepare',
      options: { seed: config.seed },
    }

    const handle: RunHandle = {
      manifest,
      config,
      batchId,
      runId,
      runDir,
    }

    const flags: string[] = []
    if (config.model == null) {
      flags.push('missing_model_resolved')
    }

    // prepare — abort on contract error / crash / timeout (CONTRACT failure algebra)
    const prepOutcome = await invokePhase('prepare', handle.manifest, this.invokeOpts)
    const prep = prepOutcome.result
    const prepAbort = abortReason(prepOutcome)
    if (prepAbort || !prep.ok) {
      if (prepAbort) flags.push(prepAbort)
      return this.finalizeFailed(
        store,
        handle,
        batchRow,
        flags,
        t0,
        prepAbort ?? 'prepare_failed',
        prep,
      )
    }

    // execute (agent or seed)
    const exec = await this.execute(handle)
    flags.push(...exec.flags)
    if (!exec.ok) {
      flags.push('execute_failed')
    }

    // post phases — still collect artifacts after execute failure (policy: keep evidence),
    // but abort remaining adapter phases on contractError / crash / timeout.
    const bundle = await this.collect(handle)
    if (bundle.aborted) {
      flags.push(...bundle.aborted.flags)
    }
    // Prefer explicit metrics.gate_ok from adapter (may be null = not applicable)
    let gateOkNum: number | null = null
    if (typeof bundle.gate.metrics.gate_ok === 'boolean') {
      gateOkNum = bundle.gate.metrics.gate_ok ? 1 : 0
    } else if (bundle.gate.metrics.gate_ok === null || bundle.gate.verdict === 'skip') {
      gateOkNum = null
    } else {
      gateOkNum = bundle.gate.ok ? 1 : 0
    }

    // compile_ok only for vite_build arms; else NULL + named reason
    let compileOk: number | null = null
    const missing: Record<string, string> = {
      ...bundle.gate.missing,
      ...bundle.render.missing,
      ...bundle.collect.missing,
    }
    if (arm.gate === 'vite_build') {
      compileOk = gateOkNum
    } else {
      missing.compile_ok = 'not_applicable_for_arm'
    }

    const renderOk = bundle.render.ok ? 1 : 0
    flags.push(...bundle.gate.flags, ...bundle.render.flags, ...bundle.collect.flags)

    const allArtifacts = [
      ...prep.artifacts,
      ...bundle.gate.artifacts,
      ...bundle.render.artifacts,
      ...bundle.collect.artifacts,
    ]

    // Fail when execute failed, blocking render missing, or adapter abort.
    // Seed mode sets exec.ok=true; live miss/nonzero sets ok=false.
    let status: 'complete' | 'failed' = 'complete'
    if (!exec.ok) {
      status = 'failed'
    }
    if (renderOk !== 1) {
      status = 'failed'
      flags.push('blocking_render_missing')
    }
    if (bundle.aborted) {
      status = 'failed'
    }

    const wall = Date.now() - t0
    const metrics = {
      ...(bundle.collect.metrics ?? {}),
      gate_wall_ms: bundle.gate.wall_ms,
      render_wall_ms: bundle.render.wall_ms,
      collect_wall_ms: bundle.collect.wall_ms,
      execute_wall_ms: exec.wall_ms,
    }

    const failureClasses: string[] = []
    if (!exec.ok) failureClasses.push('execute')
    if (renderOk !== 1) failureClasses.push('render')
    if (bundle.aborted) failureClasses.push(bundle.aborted.reason)
    if (status === 'failed' && failureClasses.length === 0) failureClasses.push('pipeline')

    const runRow: RunRow = {
      run_id: runId,
      batch_id: batchId,
      ts,
      task_id: config.taskId,
      arm: config.arm,
      model: config.model,
      model_slug: modelSlug,
      toolchain: tc.stamp,
      profile_hash: profileHash,
      harness_ver: HARNESS_VER,
      run_dir: runDir,
      status,
      wall_ms: wall,
      gate_ok: gateOkNum,
      compile_ok: compileOk,
      render_ok: renderOk,
      tokens_in: numOrNull(metrics.tokens_in),
      tokens_out: numOrNull(metrics.tokens_out),
      turns: numOrNull(metrics.turns),
      session_id: (metrics.session_id as string | null) ?? exec.session_id,
      render_method: (bundle.render.detail.method as string) ?? null,
      flags_json: JSON.stringify([...new Set(flags)]),
      metrics_json: JSON.stringify(metrics),
      gate_detail_json: JSON.stringify(bundle.gate.detail),
      artifacts_json: JSON.stringify(allArtifacts),
      failure_classes_json: JSON.stringify(failureClasses),
      missing_reasons_json: JSON.stringify(missing),
    }
    store.upsertRun(runRow)
    store.upsertArtifacts(runId, allArtifacts)

    const runManifest = {
      contract_version: CONTRACT_VERSION,
      run_id: runId,
      batch_id: batchId,
      task_id: config.taskId,
      arm: config.arm,
      model_slug: modelSlug,
      model: config.model,
      toolchain: tc.stamp,
      profile_hash: profileHash,
      harness_ver: HARNESS_VER,
      run_dir: runDir,
      status,
      wall_ms: wall,
      gate_ok: gateOkNum,
      compile_ok: compileOk,
      render_ok: renderOk,
      flags: [...new Set(flags)],
      missing,
      artifacts: allArtifacts,
      phases: {
        prepare: { ok: prep.ok, wall_ms: prep.wall_ms },
        execute: { ok: exec.ok, wall_ms: exec.wall_ms, seeded: exec.seeded },
        gate: { ok: bundle.gate.ok, verdict: bundle.gate.verdict, wall_ms: bundle.gate.wall_ms },
        render: {
          ok: bundle.render.ok,
          verdict: bundle.render.verdict,
          wall_ms: bundle.render.wall_ms,
        },
        collect: {
          ok: bundle.collect.ok,
          verdict: bundle.collect.verdict,
          wall_ms: bundle.collect.wall_ms,
        },
      },
      paths: manifest.paths,
    }
    const manifestPath = join(runDir, 'manifest.json')
    writeFileSync(manifestPath, JSON.stringify(runManifest, null, 2) + '\n')

    // n=1: single failed run → batch failed; partial multi-run later uses complete_with_failures
    const batchStatus =
      status === 'complete' ? 'complete' : (batchRow.n ?? 1) <= 1 ? 'failed' : 'complete_with_failures'
    store.upsertBatch({
      ...batchRow,
      status: batchStatus,
      manifest_json: JSON.stringify({ batch_id: batchId, runs: [runId] }),
    })

    // batch pointer file (not a copy of artifacts)
    writeFileSync(
      join(reportDir, 'batch.json'),
      JSON.stringify(
        {
          batch_id: batchId,
          runs: [runId],
          run_dirs: [runDir],
          profile_hash: profileHash,
          harness_ver: HARNESS_VER,
        },
        null,
        2,
      ) + '\n',
    )
    writeFileSync(join(reportDir, 'run_dirs.txt'), runDir + '\n')

    store.close()

    return {
      batchId,
      runId,
      runDir,
      status,
      wall_ms: wall,
      flags: [...new Set(flags)],
      gate_ok: gateOkNum,
      render_ok: renderOk,
      manifestPath,
    }
  }

  async prepare(run: RunHandle): Promise<PhaseResult> {
    const outcome = await invokePhase('prepare', run.manifest, this.invokeOpts)
    return outcome.result
  }

  async execute(run: RunHandle): Promise<ExecuteOutcome> {
    if (this.executeOverride) {
      return this.executeOverride(run)
    }
    const t0 = Date.now()
    if (run.config.seed || run.manifest.options?.seed) {
      // Seed: no agent. Write a minimal events file so collect has something.
      const eventsPath = join(run.runDir, run.manifest.paths.events)
      const line = JSON.stringify({
        ts: new Date().toISOString(),
        event: {
          hookEventName: 'Stop',
          sessionId: `seed-${run.runId}`,
          note: 'seed mode — no live agent',
        },
      })
      writeFileSync(eventsPath, line + '\n')
      return {
        ok: true,
        wall_ms: Date.now() - t0,
        session_id: `seed-${run.runId}`,
        flags: ['seed_mode'],
        seeded: true,
        message: 'seed: scaffold left as agent output',
      }
    }

    // Live grok headless
    const which = spawnSync('which', ['grok'], { encoding: 'utf8' })
    if (which.status !== 0) {
      return {
        ok: false,
        wall_ms: Date.now() - t0,
        session_id: null,
        flags: ['grok_not_on_path', 'execute_skipped'],
        seeded: false,
        message: 'grok not on PATH; pass --seed for scaffold-only runs',
      }
    }

    const promptPath = join(run.manifest.sandbox_dir, '.bench-prompt.txt')
    const prompt = existsSync(promptPath)
      ? readFileSync(promptPath, 'utf8')
      : String(run.manifest.task.prompt ?? '')

    // Install a simple events hook if .grok exists; otherwise append via wrapper
    const eventsPath = join(run.runDir, run.manifest.paths.events)
    writeFileSync(eventsPath, '')

    const r = spawnSync(
      'grok',
      [
        '-p',
        prompt.trim(),
        '--trust',
        '--disable-web-search',
        '--no-memory',
        '--permission-mode',
        'bypassPermissions',
        '--cwd',
        run.manifest.sandbox_dir,
      ],
      {
        cwd: run.manifest.sandbox_dir,
        encoding: 'utf8',
        timeout: run.manifest.limits.phase_timeout_ms,
        env: { ...process.env, ...run.manifest.env },
      },
    )

    writeFileSync(join(run.runDir, 'stdout.txt'), r.stdout ?? '')
    writeFileSync(join(run.runDir, 'stderr.txt'), r.stderr ?? '')
    writeFileSync(
      eventsPath,
      JSON.stringify({
        ts: new Date().toISOString(),
        event: {
          hookEventName: 'Stop',
          sessionId: null,
          exit: r.status,
          note: 'live grok without project hooks — session id may be null',
        },
      }) + '\n',
      { flag: 'a' },
    )

    return {
      ok: r.status === 0,
      wall_ms: Date.now() - t0,
      session_id: null,
      flags: r.status === 0 ? [] : ['dispatch_nonzero_exit'],
      seeded: false,
      message: r.status === 0 ? 'grok ok' : `grok exit ${r.status}`,
    }
  }

  async collect(run: RunHandle): Promise<CollectBundle> {
    // Phase policy (CONTRACT): prepare abort is handled by runOne.
    // gate fail (ok:false) → continue; contract/crash/timeout → abort remaining.
    // render fail → continue to collect for metrics; contract/crash/timeout → abort.
    // collect fail → record flags only (no inventing metrics).
    const gateOut = await invokePhase(
      'gate',
      { ...run.manifest, phase: 'gate' },
      this.invokeOpts,
    )
    const gateAbort = abortReason(gateOut)
    if (gateAbort) {
      return {
        gate: gateOut.result,
        render: skippedPhase('render', gateAbort),
        collect: skippedPhase('collect', gateAbort),
        aborted: { phase: 'gate', reason: gateAbort, flags: [gateAbort, 'adapter_abort'] },
      }
    }

    const renderOut = await invokePhase(
      'render',
      { ...run.manifest, phase: 'render' },
      this.invokeOpts,
    )
    const renderAbort = abortReason(renderOut)
    if (renderAbort) {
      return {
        gate: gateOut.result,
        render: renderOut.result,
        collect: skippedPhase('collect', renderAbort),
        aborted: { phase: 'render', reason: renderAbort, flags: [renderAbort, 'adapter_abort'] },
      }
    }

    const collectOut = await invokePhase(
      'collect',
      { ...run.manifest, phase: 'collect' },
      this.invokeOpts,
    )
    const collectAbort = abortReason(collectOut)
    if (collectAbort) {
      return {
        gate: gateOut.result,
        render: renderOut.result,
        collect: collectOut.result,
        aborted: {
          phase: 'collect',
          reason: collectAbort,
          flags: [collectAbort, 'adapter_abort'],
        },
      }
    }

    return {
      gate: gateOut.result,
      render: renderOut.result,
      collect: collectOut.result,
    }
  }

  private finalizeFailed(
    store: RunStore,
    handle: RunHandle,
    batchRow: BatchRow,
    flags: string[],
    t0: number,
    reason: string,
    prep: PhaseResult,
  ): RunOutcome {
    flags.push(reason)
    const wall = Date.now() - t0
    const runRow: RunRow = {
      run_id: handle.runId,
      batch_id: handle.batchId,
      ts: batchRow.ts,
      task_id: handle.config.taskId,
      arm: handle.config.arm,
      model: handle.config.model,
      model_slug: handle.config.modelSlug,
      toolchain: handle.manifest.toolchain,
      profile_hash: handle.manifest.profile_hash,
      harness_ver: HARNESS_VER,
      run_dir: handle.runDir,
      status: 'failed',
      wall_ms: wall,
      gate_ok: null,
      compile_ok: null,
      render_ok: null,
      tokens_in: null,
      tokens_out: null,
      turns: null,
      session_id: null,
      render_method: null,
      flags_json: JSON.stringify(flags),
      metrics_json: null,
      gate_detail_json: null,
      artifacts_json: JSON.stringify(prep.artifacts),
      failure_classes_json: JSON.stringify([reason]),
      missing_reasons_json: JSON.stringify({ pipeline: reason }),
    }
    store.upsertRun(runRow)
    store.upsertBatch({ ...batchRow, status: 'failed' })
    const manifestPath = join(handle.runDir, 'manifest.json')
    writeFileSync(
      manifestPath,
      JSON.stringify(
        {
          run_id: handle.runId,
          status: 'failed',
          reason,
          flags,
        },
        null,
        2,
      ) + '\n',
    )
    store.close()
    return {
      batchId: handle.batchId,
      runId: handle.runId,
      runDir: handle.runDir,
      status: 'failed',
      wall_ms: wall,
      flags,
      gate_ok: null,
      render_ok: null,
      manifestPath,
    }
  }
}

function numOrNull(v: unknown): number | null {
  if (typeof v === 'number' && Number.isFinite(v)) return v
  return null
}

/**
 * CONTRACT abort mapping: exit 2 / malformed → contract_error;
 * exit 1 / spawn fail → adapter_crash; hang → adapter_timeout.
 * Phase ok:false alone is NOT abort (gate/render may fail and continue).
 */
function abortReason(outcome: InvokeOutcome): string | null {
  if (outcome.timedOut) return 'adapter_timeout'
  if (outcome.contractError) return 'contract_error'
  if (outcome.exitCode === 1) return 'adapter_crash'
  return null
}

function skippedPhase(phase: 'gate' | 'render' | 'collect', reason: string): PhaseResult {
  const r = emptyPhaseResult(phase)
  r.verdict = 'error'
  r.ok = false
  r.flags = ['phase_skipped', reason]
  r.error = {
    code: 'PHASE_SKIPPED',
    message: `skipped after prior adapter abort (${reason})`,
  }
  r.missing[phase] = reason
  return r
}

/**
 * Allocate a word-pair run id and create its directory exclusively.
 * Retries on EEXIST so concurrent runners cannot clobber the same path.
 */
function allocateExclusiveRunDir(
  lab: string,
  toolchain: string,
  task: string,
  model: string,
): { runId: string; runDir: string } {
  for (let attempt = 0; attempt < 64; attempt++) {
    const taken = listTakenRunIds(lab, toolchain, task, model)
    const runId = allocateWordPair({
      taken,
      seed: `${Date.now()}-${attempt}-${Math.random()}`,
    })
    const runDir = runDirAbs(lab, { toolchain, task, model, runId })
    try {
      mkdirSync(dirname(runDir), { recursive: true })
      mkdirSync(runDir) // exclusive: throws EEXIST if already present
      return { runId, runDir }
    } catch (e) {
      const err = e as NodeJS.ErrnoException
      if (err.code === 'EEXIST') continue
      throw e
    }
  }
  throw new Error('failed to allocate exclusive run dir after 64 attempts')
}
