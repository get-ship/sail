/**
 * omp dispatch: single real-run adapter.
 * Shells `omp -p --cwd <workdir> --no-session --mode json "<prompt>"`.
 * Model, provider, key, and budget resolve from runtime flags/env through omp —
 * this package never defaults a model name.
 */

import { spawn } from 'node:child_process'
import { readdirSync, statSync } from 'node:fs'
import { join, relative } from 'node:path'
import type { Trajectory, TrajectoryTurn } from '../types.ts'
import type { DispatchAdapter, DispatchRequest } from './types.ts'

export interface OmpDispatchOpts {
  /** Binary name or path. Default: omp (or OMP_BIN env). */
  bin?: string
  /** Extra CLI args inserted before the prompt. */
  extraArgs?: string[]
}

export class OmpDispatch implements DispatchAdapter {
  readonly mode = 'omp' as const
  private bin: string
  private extraArgs: string[]

  constructor(opts: OmpDispatchOpts = {}) {
    this.bin = opts.bin ?? process.env.OMP_BIN ?? 'omp'
    this.extraArgs = opts.extraArgs ?? []
  }

  async run(req: DispatchRequest): Promise<Trajectory> {
    const startedAt = new Date().toISOString()
    const before = snapshotFiles(req.workdir)
    const timeoutMs = req.timeoutMs ?? 10 * 60 * 1000

    const prompt = buildPrompt(req.prompt)
    const args = buildOmpArgs({
      workdir: req.workdir,
      prompt,
      model: req.model,
      apiKey: req.apiKey,
      extraArgs: this.extraArgs,
    })

    const { stdout, stderr, code, signal } = await runProcess(this.bin, args, {
      cwd: req.workdir,
      env: { ...process.env, ...req.env },
      timeoutMs,
    })

    const after = snapshotFiles(req.workdir)
    const filesWritten = [...after].filter((f) => !before.has(f))
    const resolvedModel = resolveModelLabel(req.model, stdout)

    const turns: TrajectoryTurn[] = [
      { role: 'user', content: req.prompt, ts: startedAt },
      {
        role: 'assistant',
        content: stdout || (stderr ? `[stderr]\n${stderr}` : '(empty response)'),
        ts: new Date().toISOString(),
      },
    ]

    const finishedAt = new Date().toISOString()
    return {
      task: req.task,
      model: resolvedModel,
      dispatch: 'omp',
      turns,
      turnCount: 1,
      startedAt,
      finishedAt,
      filesWritten,
      meta: {
        exitCode: code,
        signal,
        stderr: stderr.slice(0, 4000),
        bin: this.bin,
        // Record only whether a model was requested — never invent one.
        modelRequested: req.model || null,
      },
    }
  }
}

/** Build omp argv: -p --cwd … --no-session --mode json [--model] [--api-key] <prompt> */
export function buildOmpArgs(opts: {
  workdir: string
  prompt: string
  model?: string
  apiKey?: string
  extraArgs?: string[]
}): string[] {
  const args: string[] = [
    '-p',
    '--cwd',
    opts.workdir,
    '--no-session',
    '--mode',
    'json',
  ]
  // Pass model only when operator supplied one — no code defaults.
  const model = opts.model?.trim()
  if (model) {
    args.push('--model', model)
  }
  const apiKey = opts.apiKey?.trim()
  if (apiKey) {
    args.push('--api-key', apiKey)
  }
  if (opts.extraArgs?.length) {
    args.push(...opts.extraArgs)
  }
  args.push(opts.prompt)
  return args
}

function buildPrompt(taskPrompt: string): string {
  return [
    'You are authoring a Sail UI application in this working directory.',
    'Read SAIL_SKILL.md for the authoring surface.',
    'Write your solution primarily in routes/page.sail.',
    'Do not invent CSS-selector-based tests; use visible text and labels.',
    '',
    '## Task',
    taskPrompt,
  ].join('\n')
}

/**
 * Prefer the operator-supplied model string; optionally scrape resolved id
 * from omp JSON stdout. Never invent a provider model name.
 */
export function resolveModelLabel(requested: string | undefined, stdout: string): string {
  const req = requested?.trim()
  if (req) return req
  const fromJson = scrapeModelFromJson(stdout)
  if (fromJson) return fromJson
  return ''
}

function scrapeModelFromJson(stdout: string): string | null {
  const text = stdout.trim()
  if (!text) return null
  // omp --mode json may emit one object or NDJSON; try a few shapes.
  const candidates = text.startsWith('{') ? [text] : text.split('\n').filter((l) => l.startsWith('{'))
  for (const line of candidates) {
    try {
      const obj = JSON.parse(line) as Record<string, unknown>
      for (const key of ['model', 'modelId', 'resolvedModel']) {
        const v = obj[key]
        if (typeof v === 'string' && v.trim()) return v.trim()
      }
      // Nested common shapes: { info: { model } }, { session: { model } }
      for (const nest of ['info', 'session', 'meta', 'config']) {
        const inner = obj[nest]
        if (inner && typeof inner === 'object') {
          const m = (inner as Record<string, unknown>).model
          if (typeof m === 'string' && m.trim()) return m.trim()
        }
      }
    } catch {
      /* try next line */
    }
  }
  return null
}

function snapshotFiles(root: string): Set<string> {
  const out = new Set<string>()
  const walk = (dir: string) => {
    let entries
    try {
      entries = readdirSync(dir, { withFileTypes: true })
    } catch {
      return
    }
    for (const e of entries) {
      if (e.name === 'node_modules' || e.name === '.git') continue
      const full = join(dir, e.name)
      if (e.isDirectory()) walk(full)
      else if (e.isFile()) {
        try {
          if (statSync(full).isFile()) out.add(relative(root, full))
        } catch {
          /* race */
        }
      }
    }
  }
  walk(root)
  return out
}

function runProcess(
  bin: string,
  args: string[],
  opts: { cwd: string; env: NodeJS.ProcessEnv; timeoutMs: number },
): Promise<{ stdout: string; stderr: string; code: number | null; signal: string | null }> {
  return new Promise((resolve, reject) => {
    const child = spawn(bin, args, {
      cwd: opts.cwd,
      env: opts.env,
      stdio: ['ignore', 'pipe', 'pipe'],
    })
    let stdout = ''
    let stderr = ''
    const timer = setTimeout(() => {
      child.kill('SIGTERM')
      setTimeout(() => child.kill('SIGKILL'), 2000)
    }, opts.timeoutMs)
    child.stdout?.on('data', (d: Buffer) => {
      stdout += d.toString()
    })
    child.stderr?.on('data', (d: Buffer) => {
      stderr += d.toString()
    })
    child.on('error', (err) => {
      clearTimeout(timer)
      reject(err)
    })
    child.on('close', (code, signal) => {
      clearTimeout(timer)
      resolve({ stdout, stderr, code, signal })
    })
  })
}
