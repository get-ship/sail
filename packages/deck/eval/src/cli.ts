// `sui-eval` CLI entry. Loads task JSON from disk, dispatches to a
// model via the public runEval API, prints structured results.
//
// Usage:
//   sui-eval run --task <id> --model <model> [--out <dir>]
//   sui-eval run --all --tier <easy|medium|hard> --model <model> [--out <dir>]
//   sui-eval --help

import { readFile, readdir, mkdir, writeFile } from 'node:fs/promises'
import { existsSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { runEval, type EvalResult, type EvalTask } from './index.ts'

const HELP = `sui-eval — design-quality regression suite for .sui views

Usage:
  sui-eval run --task <id> --model <model> [--out <dir>]
  sui-eval run --all --tier <easy|medium|hard> --model <model> [--out <dir>]
  sui-eval list [--tier <easy|medium|hard>]
  sui-eval --help

Environment:
  ANTHROPIC_API_KEY    required for live dispatch.

Examples:
  sui-eval run --task counter-toggle --model claude-haiku-4-5
  sui-eval run --all --tier easy --model claude-sonnet-4-6 --out .artifacts/eval
`

interface Argv {
  command: string
  flags: Record<string, string | boolean>
}

export function parseArgv(argv: readonly string[]): Argv {
  const [command = '', ...rest] = argv
  const flags: Record<string, string | boolean> = {}
  for (let i = 0; i < rest.length; i++) {
    const tok = rest[i]
    if (tok === undefined || !tok.startsWith('--')) continue
    const key = tok.slice(2)
    const next = rest[i + 1]
    if (next !== undefined && !next.startsWith('--')) {
      flags[key] = next
      i += 1
    } else {
      flags[key] = true
    }
  }
  return { command, flags }
}

export async function loadTask(tasksDir: string, id: string): Promise<EvalTask> {
  for (const tier of ['easy', 'medium', 'hard'] as const) {
    const path = join(tasksDir, tier, `${id}.json`)
    if (existsSync(path)) {
      return parseTaskFile(await readFile(path, 'utf8'), path)
    }
  }
  throw new Error(`task not found: ${id} (looked in ${tasksDir}/{easy,medium,hard})`)
}

export async function listTasks(
  tasksDir: string,
  tier?: 'easy' | 'medium' | 'hard',
): Promise<EvalTask[]> {
  const tiers = tier ? [tier] : (['easy', 'medium', 'hard'] as const)
  const out: EvalTask[] = []
  for (const t of tiers) {
    const dir = join(tasksDir, t)
    if (!existsSync(dir)) continue
    const entries = await readdir(dir)
    for (const name of entries) {
      if (!name.endsWith('.json')) continue
      const path = join(dir, name)
      out.push(parseTaskFile(await readFile(path, 'utf8'), path))
    }
  }
  return out
}

function parseTaskFile(raw: string, path: string): EvalTask {
  let json: unknown
  try {
    json = JSON.parse(raw)
  } catch (err) {
    throw new Error(`task ${path}: invalid JSON — ${describe(err)}`)
  }
  if (!isObject(json)) throw new Error(`task ${path}: expected object`)
  const id = json['id']
  const difficulty = json['difficulty']
  const description = json['description']
  const fixtures = json['fixtures']
  const behavioralTests = json['behavioralTests']
  if (typeof id !== 'string') throw new Error(`task ${path}: id missing`)
  if (difficulty !== 'easy' && difficulty !== 'medium' && difficulty !== 'hard') {
    throw new Error(`task ${path}: difficulty must be easy|medium|hard`)
  }
  if (typeof description !== 'string') {
    throw new Error(`task ${path}: description must be string`)
  }
  if (!isObject(fixtures) || typeof fixtures['template'] !== 'string') {
    throw new Error(`task ${path}: fixtures.template must be string`)
  }
  if (typeof behavioralTests !== 'string') {
    throw new Error(`task ${path}: behavioralTests must be string`)
  }
  const task: EvalTask = {
    id,
    difficulty,
    description,
    fixtures: { template: fixtures['template'] },
    behavioralTests,
  }
  if (typeof fixtures['setup'] === 'string') {
    task.fixtures.setup = fixtures['setup']
  }
  if (Array.isArray(json['expectedBindings'])) {
    task.expectedBindings = json['expectedBindings'].map(String)
  }
  return task
}

async function writeReport(outDir: string, result: EvalResult): Promise<void> {
  await mkdir(outDir, { recursive: true })
  const target = join(outDir, `${result.taskId}.json`)
  await writeFile(target, JSON.stringify(result, null, 2) + '\n', 'utf8')
}

function defaultTasksDir(): string {
  const here = dirname(fileURLToPath(import.meta.url))
  return resolve(here, '..', 'tasks')
}

function isObject(v: unknown): v is Record<string, unknown> {
  return typeof v === 'object' && v !== null && !Array.isArray(v)
}

function describe(err: unknown): string {
  return err instanceof Error ? err.message : String(err)
}

function asString(v: string | boolean | undefined, name: string): string {
  if (typeof v !== 'string') throw new Error(`--${name} requires a value`)
  return v
}

/** CLI entry. Returns an exit code. */
export async function main(argv: readonly string[]): Promise<number> {
  if (argv.length === 0 || argv.includes('--help') || argv.includes('-h')) {
    process.stdout.write(HELP)
    return 0
  }
  const parsed = parseArgv(argv)
  const tasksDir = typeof parsed.flags['tasks'] === 'string'
    ? resolve(parsed.flags['tasks'])
    : defaultTasksDir()

  if (parsed.command === 'list') {
    const tier = parsed.flags['tier']
    const filter =
      tier === 'easy' || tier === 'medium' || tier === 'hard' ? tier : undefined
    const tasks = await listTasks(tasksDir, filter)
    for (const t of tasks) {
      process.stdout.write(`${t.difficulty.padEnd(6)} ${t.id}\n`)
    }
    return 0
  }

  if (parsed.command === 'run') {
    const model = asString(parsed.flags['model'], 'model')
    const outDir =
      typeof parsed.flags['out'] === 'string' ? resolve(parsed.flags['out']) : undefined
    const tasks: EvalTask[] = parsed.flags['all']
      ? await listTasks(
          tasksDir,
          (() => {
            const t = parsed.flags['tier']
            return t === 'easy' || t === 'medium' || t === 'hard' ? t : undefined
          })(),
        )
      : [await loadTask(tasksDir, asString(parsed.flags['task'], 'task'))]

    let failures = 0
    for (const task of tasks) {
      const result = await runEval(task, { model })
      process.stdout.write(JSON.stringify(result) + '\n')
      if (outDir) await writeReport(outDir, result)
      if (!result.passed) failures += 1
    }
    return failures === 0 ? 0 : 1
  }

  process.stderr.write(`unknown command: ${parsed.command}\n${HELP}`)
  return 2
}
