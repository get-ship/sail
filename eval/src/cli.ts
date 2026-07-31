#!/usr/bin/env bun
/**
 * sail-eval CLI
 *
 *   bun src/cli.ts run --lane cheap --dispatch recorded
 *   bun src/cli.ts run --lane frontier --dispatch omp --model <fuzzy> --task login-form
 *   bun src/cli.ts run --lane frontier --dispatch omp --scaffold react-tw --task login-form
 *   bun src/cli.ts pace --model <fuzzy> --interval 10m --arms sail,react-tw,vanilla
 *   bun src/cli.ts record <dir> [--task slug] [--promote]
 *   bun src/cli.ts list
 *   bun src/cli.ts list-arms
 */

import { listArms } from './arm.ts'
import { parseInterval, runPaced } from './pace.ts'
import { runCorpus } from './runner.ts'
import { recordSession } from './recorder.ts'
import { listTasks } from './task.ts'
import type { DispatchMode, Lane, ScaffoldKind } from './types.ts'

function usage(): never {
  console.log(`sail-eval — sail-owned eval harness

Usage:
  sail-eval run [--lane cheap|frontier] [--dispatch recorded|omp]
                [--task <slug>]... [--model <fuzzy>] [--api-key <key>]
                [--scaffold blank|kitchen-sink|react-tw|vanilla]
                [--keep-workdir] [--base-url <url>] [--no-report]
                [--strict | --no-skip]   # default: strict (missing sail fails)
                [--allow-skip]          # soft-skip missing host tooling
                [--cheap-behavior]      # opt-in source-text behavior on cheap
                [--allow-skip-beauty]   # frontier: soft-skip missing beauty score
  sail-eval pace --model <fuzzy> --interval 10m
                [--arms sail,react-tw,vanilla] [--task <slug>]...
                [--lane frontier] [--api-key <key>] [--keep-workdir]
                [--no-sleep]            # tests/smoke: no inter-run delay
                [--allow-skip] [--no-skip-beauty] [--no-report]
  sail-eval record <dir> [--task <slug>] [--prompt <text>] [--promote]
  sail-eval list
  sail-eval list-arms

Environment:
  SAIL_BIN     path to sail binary (default: target/debug|release/sail)
  OMP_BIN      path to omp CLI (default: omp)

Notes:
  Cheap lane = compile + vocab + gates only (per EVAL-0).
  --cheap-behavior runs .sail source-text heuristics — not deck/playwright.
  Model is NEVER defaulted by this package. Pass --model for omp (fuzzy match)
  or for pace (required). Keys via env/profile — never commit keys.
  Control arms: react-tw / vanilla profiles under eval/arms/ — no sail leaks.
  pace is resume-safe: completed (task, arm, model) triples under
  eval/runs/<date>/<task>/<arm>/<modelSlug>/ skip (re-run same --model).
  Control-arm behavior without --base-url is static source-text fallback only.
  Beauty screenshots are operator-owned (screenshots/ README per run).

Overnight baseline (EVAL-1):
  cd eval && bun src/cli.ts pace --model grok --interval 10m --arms sail,react-tw,vanilla
  # 10 tasks × 3 arms ≈ 30 runs ≈ 5h at 10m interval. Resume after interrupt with same command.
`)
  process.exit(2)
}

const BOOL_FLAGS = new Set([
  'no-report',
  'keep-workdir',
  'promote',
  'strict',
  'no-skip',
  'allow-skip',
  'cheap-behavior',
  'allow-skip-beauty',
  'no-skip-beauty',
  'no-sleep',
])

function parseArgs(argv: string[]) {
  const args = argv.slice(2)
  if (args.length === 0 || args[0] === '-h' || args[0] === '--help') usage()
  const cmd = args[0]!
  const flags: Record<string, string | boolean | string[]> = {}
  const positionals: string[] = []
  for (let i = 1; i < args.length; i++) {
    const a = args[i]!
    if (a.startsWith('--')) {
      const key = a.slice(2)
      if (BOOL_FLAGS.has(key)) {
        flags[key] = true
        continue
      }
      const val = args[i + 1]
      if (!val || val.startsWith('--')) {
        flags[key] = true
        continue
      }
      i++
      if (key === 'task') {
        const arr = (flags.task as string[] | undefined) ?? []
        arr.push(val)
        flags.task = arr
      } else {
        flags[key] = val
      }
      continue
    }
    positionals.push(a)
  }
  return { cmd, flags, positionals }
}

async function main() {
  const { cmd, flags, positionals } = parseArgs(process.argv)

  if (cmd === 'list') {
    const tasks = listTasks()
    if (!tasks.length) {
      console.log('(no tasks)')
      return
    }
    for (const t of tasks) console.log(t)
    return
  }

  if (cmd === 'list-arms') {
    const arms = listArms()
    if (!arms.length) {
      console.log('(no arms)')
      return
    }
    for (const a of arms) console.log(a)
    return
  }

  if (cmd === 'pace') {
    const model = flags.model as string | undefined
    if (!model || typeof model !== 'string') {
      console.error('pace requires --model <fuzzy> (no hardcoded default)')
      process.exit(1)
    }
    const intervalRaw = (flags.interval as string | undefined) ?? '10m'
    let intervalMs: number
    try {
      intervalMs = parseInterval(String(intervalRaw))
    } catch (e) {
      console.error(e instanceof Error ? e.message : e)
      process.exit(1)
    }
    const armsFlag = flags.arms as string | undefined
    const arms = armsFlag
      ? String(armsFlag)
          .split(',')
          .map((s) => s.trim())
          .filter(Boolean)
      : undefined
    const tasks = flags.task as string[] | undefined
    const lane = (flags.lane as Lane | undefined) ?? 'frontier'
    if (lane !== 'cheap' && lane !== 'frontier') {
      console.error(`invalid --lane ${lane}`)
      process.exit(1)
    }
    const allowSkip =
      Boolean(flags['allow-skip']) && !flags.strict && !flags['no-skip']
    const summary = await runPaced({
      model,
      intervalMs,
      arms,
      tasks,
      lane,
      apiKey: flags['api-key'] as string | undefined,
      keepWorkdir: Boolean(flags['keep-workdir']),
      writeReport: !flags['no-report'],
      behaviorBaseUrl: flags['base-url'] as string | undefined,
      allowSkip,
      // Beauty is operator-scored post-run; default soft-skip so overnight does not fail open incomplete.
      // Pass --no-skip-beauty only if you want incomplete beauty to fail the aggregate.
      allowSkipBeauty: !flags['no-skip-beauty'],
      noSleep: Boolean(flags['no-sleep']),
    })
    console.log(
      `pace done: ${summary.passCount} pass / ${summary.failCount} fail / ${summary.skipCount} skip` +
        ` (${summary.runs.length} runs; resumed ${summary.paced?.resumed ?? 0}, executed ${summary.paced?.executed ?? 0})`,
    )
    if (summary.failCount > 0) process.exitCode = 1
    return
  }

  if (cmd === 'run') {
    const lane = (flags.lane as Lane | undefined) ?? 'cheap'
    const dispatch = (flags.dispatch as DispatchMode | undefined) ?? 'recorded'
    if (lane !== 'cheap' && lane !== 'frontier') {
      console.error(`invalid --lane ${lane}`)
      process.exit(1)
    }
    if (dispatch !== 'recorded' && dispatch !== 'omp') {
      console.error(`invalid --dispatch ${dispatch} (expected recorded|omp)`)
      process.exit(1)
    }
    const scaffold = flags.scaffold as ScaffoldKind | undefined
    if (
      scaffold != null &&
      scaffold !== 'blank' &&
      scaffold !== 'kitchen-sink' &&
      scaffold !== 'react-tw' &&
      scaffold !== 'vanilla'
    ) {
      console.error(`invalid --scaffold ${scaffold}`)
      process.exit(1)
    }
    // Default strict: missing sail fails. --allow-skip soft-skips host legs.
    const allowSkip = Boolean(flags['allow-skip']) && !flags.strict && !flags['no-skip']
    const tasks = flags.task as string[] | undefined
    const summary = await runCorpus({
      lane,
      dispatch,
      // Pass-through only — never invent a model name here.
      model: flags.model as string | undefined,
      apiKey: flags['api-key'] as string | undefined,
      scaffold,
      tasks,
      keepWorkdir: Boolean(flags['keep-workdir']),
      writeReport: !flags['no-report'],
      behaviorBaseUrl: flags['base-url'] as string | undefined,
      allowSkip,
      enableCheapBehavior: Boolean(flags['cheap-behavior']),
      allowSkipBeauty: Boolean(flags['allow-skip-beauty']),
    })
    console.log(
      `done: ${summary.passCount} pass / ${summary.failCount} fail / ${summary.skipCount} skip (${summary.runs.length} tasks)`,
    )
    // Non-zero exit on any fail (includes incomplete frontier beauty without score).
    // Soft-skips alone (skipCount) leave exit 0 — intentional for --allow-skip tooling.
    if (summary.failCount > 0) process.exitCode = 1
    return
  }

  if (cmd === 'record') {
    const dir = positionals[0]
    if (!dir) {
      console.error('record requires <dir>')
      process.exit(1)
    }
    const result = await recordSession({
      dir,
      task: flags.task
        ? Array.isArray(flags.task)
          ? flags.task[0]
          : (flags.task as string)
        : undefined,
      prompt: flags.prompt as string | undefined,
      promote: Boolean(flags.promote),
      lane: (flags.lane as Lane | undefined) ?? 'cheap',
      model: flags.model as string | undefined,
    })
    console.log(`recorded → ${result.runDirDisplay} (ok=${result.gradesOk})`)
    if (result.promotedToDisplay) {
      console.log(`promoted golden → ${result.promotedToDisplay}`)
    }
    return
  }

  usage()
}

main().catch((e) => {
  console.error(e instanceof Error ? e.message : e)
  process.exit(1)
})
