/**
 * `eval record <dir>` — wrap ANY sail authoring session and save
 * (prompt, trajectory, final files, grades) as a run.
 * Promotion path: run → task + golden.
 */

import {
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'node:fs'
import { basename, join, resolve } from 'node:path'
import { realHost, toWorkspacePath } from './host.ts'
import { assertSafeSlug, evalRoot, runsDir, tasksDir } from './paths.ts'
import { runGradeChain } from './graders/chain.ts'
import { sanitizeJson, sanitizeText } from './sanitize.ts'
import type { Lane, Trajectory, TrajectoryTurn } from './types.ts'

export interface RecordOptions {
  /** Directory containing the authoring session (.sail files). */
  dir: string
  /** Task slug to attribute (creates under runs/<date>/<slug>). */
  task?: string
  prompt?: string
  lane?: Lane
  /** Optional trajectory.jsonl to include. */
  trajectoryPath?: string
  /** Promote final files into tasks/<slug>/golden. */
  promote?: boolean
  model?: string
}

export interface RecordResult {
  runDir: string
  /** Workspace-relative when under the monorepo. */
  runDirDisplay: string
  task: string
  gradesOk: boolean
  promotedTo?: string
  promotedToDisplay?: string
}

export async function recordSession(opts: RecordOptions): Promise<RecordResult> {
  const dir = resolve(opts.dir)
  if (!existsSync(dir) || !statSync(dir).isDirectory()) {
    throw new Error(`record: not a directory: ${dir}`)
  }

  const rawSlug =
    opts.task ??
    basename(dir)
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-|-$/g, '')
  const task = assertSafeSlug(rawSlug || 'session')
  const startedAt = new Date().toISOString()
  const date = startedAt.slice(0, 10)
  const runDir = join(runsDir(), date, task)
  mkdirSync(runDir, { recursive: true })

  // Snapshot final files into runDir/files
  const filesOut = join(runDir, 'files')
  mkdirSync(filesOut, { recursive: true })
  copySailTree(dir, filesOut)

  let turns: TrajectoryTurn[] = []
  if (opts.trajectoryPath && existsSync(opts.trajectoryPath)) {
    for (const line of readFileSync(opts.trajectoryPath, 'utf8').split('\n')) {
      const t = line.trim()
      if (!t) continue
      try {
        turns.push(JSON.parse(t) as TrajectoryTurn)
      } catch {
        /* skip */
      }
    }
  } else {
    turns = [
      {
        role: 'user',
        content: opts.prompt ?? `(recorded session from ${basename(dir)})`,
        ts: startedAt,
      },
      {
        role: 'assistant',
        content: '[recorder] session snapshot',
        ts: startedAt,
      },
    ]
  }

  const finishedAt = new Date().toISOString()
  const trajectory: Trajectory = {
    task,
    model: opts.model ?? 'recorded-session',
    dispatch: 'recorded',
    turns,
    turnCount: Math.max(1, turns.filter((t) => t.role === 'assistant').length),
    startedAt,
    finishedAt,
    filesWritten: listRelFiles(filesOut),
    meta: { sourceDir: '<path>' },
  }

  const host = realHost()
  const lane: Lane = opts.lane ?? 'cheap'
  // Grade the in-workspace snapshot so sail ops accept relative paths.
  // Recorder is operator-local: soft-skip when sail is absent.
  const grades = await runGradeChain({
    host,
    workdir: filesOut,
    lane,
    grading: {
      compile: true,
      vocab_share: true,
      gates: true,
      behavior: false,
      beauty: false,
    },
    trajectory,
    allowSkip: true,
  })

  writeFileSync(
    join(runDir, 'trajectory.jsonl'),
    trajectory.turns.map((t) => JSON.stringify(sanitizeJson(t, dir))).join('\n') + '\n',
  )
  writeFileSync(join(runDir, 'grades.json'), JSON.stringify(sanitizeJson(grades, dir), null, 2))
  writeFileSync(
    join(runDir, 'result.json'),
    JSON.stringify(
      sanitizeJson(
        {
          task,
          lane,
          dispatch: 'recorded',
          model: trajectory.model,
          grades,
          startedAt,
          finishedAt,
          prompt: opts.prompt ?? null,
        },
        dir,
      ),
      null,
      2,
    ),
  )
  writeFileSync(
    join(runDir, 'prompt.txt'),
    sanitizeText(opts.prompt ?? '', dir),
  )

  let promotedTo: string | undefined
  let promotedToDisplay: string | undefined
  if (opts.promote) {
    const golden = join(tasksDir(), task, 'golden')
    // Replace semantics: clear stale golden files so replay is not a hybrid.
    if (existsSync(golden)) {
      rmSync(golden, { recursive: true, force: true })
    }
    mkdirSync(golden, { recursive: true })
    copySailTree(dir, golden)
    // Ensure task.jsonc exists if missing
    const taskJson = join(tasksDir(), task, 'task.jsonc')
    if (!existsSync(taskJson)) {
      mkdirSync(join(tasksDir(), task), { recursive: true })
      writeFileSync(
        taskJson,
        `// Promoted from recorder ${date}\n` +
          JSON.stringify(
            {
              prompt: opts.prompt ?? `Promoted session ${task}`,
              scaffold: 'blank',
              lanes: ['cheap', 'frontier'],
              grading: {
                compile: true,
                vocab_share: true,
                gates: true,
                behavior: true,
                beauty: true,
              },
            },
            null,
            2,
          ) +
          '\n',
      )
    }
    promotedTo = golden
    promotedToDisplay = displayPath(golden)
  }

  return {
    runDir,
    runDirDisplay: displayPath(runDir),
    task,
    gradesOk: grades.ok,
    promotedTo,
    promotedToDisplay,
  }
}

function displayPath(abs: string): string {
  try {
    return toWorkspacePath(abs)
  } catch {
    // Outside monorepo: still prefer relative-to-eval when possible.
    try {
      return toWorkspacePath(abs, evalRoot())
    } catch {
      return sanitizeText(abs)
    }
  }
}

function copySailTree(src: string, dest: string): void {
  const walk = (d: string, rel = '') => {
    for (const name of readdirSync(d, { withFileTypes: true })) {
      if (name.name === 'node_modules' || name.name === '.git' || name.name === 'target') continue
      const from = join(d, name.name)
      const toRel = rel ? join(rel, name.name) : name.name
      const to = join(dest, toRel)
      if (name.isDirectory()) {
        mkdirSync(to, { recursive: true })
        walk(from, toRel)
      } else if (name.isFile()) {
        // Prefer sail/config sources; skip huge binaries
        if (/\.(png|jpg|jpeg|webp|wasm|node)$/i.test(name.name)) continue
        mkdirSync(join(dest, rel), { recursive: true })
        cpSync(from, to)
      }
    }
  }
  mkdirSync(dest, { recursive: true })
  walk(src)
}

function listRelFiles(root: string): string[] {
  const out: string[] = []
  const walk = (d: string, rel = '') => {
    for (const name of readdirSync(d, { withFileTypes: true })) {
      const toRel = rel ? join(rel, name.name) : name.name
      if (name.isDirectory()) walk(join(d, name.name), toRel)
      else out.push(toRel)
    }
  }
  walk(root)
  return out
}
