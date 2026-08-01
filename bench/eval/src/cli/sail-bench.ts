#!/usr/bin/env bun
/**
 * sail-bench — thin CLI speaking RUN CONTRACT v1 to the interim local core.
 *
 *   sail-bench run --task parallax-landing --arm sail --toolchain current [--seed]
 *   sail-bench report --batch <id>
 *   sail-bench grade <run> key=value … [note]
 *   sail-bench serve [--port N]
 */

import { labRoot } from '../naming/paths.ts'
import { LocalRunner } from '../runner/local.ts'
import { writeBatchReport } from '../report/batch.ts'
import { writeGrades } from '../store/grades.ts'
import { RunStore } from '../store/db.ts'
import { storePath } from '../naming/paths.ts'
import { serveLab } from '../serve/static.ts'
import { HARNESS_VER } from '../config/types.ts'

function usage(): never {
  console.log(`sail-bench ${HARNESS_VER} — typed eval harness (HARNESS-2)

Usage:
  sail-bench run --task <id> --arm <sail|react|vanilla> --toolchain <current|stamp>
                 [--model <slug>] [--profile p0] [--seed] [--lab <path>]
                 [--in-process]   # adapters in-process (default for speed)
                 [--subprocess]  # spawn eval-adapter (contract smoke)
  sail-bench report --batch <id> [--lab <path>]
  sail-bench grade <run_id|run_dir> key=value … [note=…] [--lab <path>]
  sail-bench serve [--port N] [--lab <path>]

Env:
  SAIL_LAB       lab root (default ~/dev/sail-lab)
  SAIL_REPO      sail repo root
  SAIL_BIN       sail binary for gates
  CHROME_PATH    chromium for render
  SAIL_BENCH_MODEL  default model slug when --model omitted (live runs)

Notes:
  --seed skips the live agent and keeps the payload scaffold (for harness smoke).
  Live agent requires grok on PATH and is not the default for CI.
  Reports reference run dirs; they never copy artifact bytes.
`)
  process.exit(2)
}

function parseArgs(argv: string[]) {
  const args = argv.slice(2)
  if (args.length === 0 || args[0] === '-h' || args[0] === '--help') usage()
  const cmd = args[0]!
  const flags: Record<string, string | boolean> = {}
  const positionals: string[] = []
  for (let i = 1; i < args.length; i++) {
    const a = args[i]!
    if (a.startsWith('--')) {
      const key = a.slice(2)
      const bools = new Set(['seed', 'in-process', 'subprocess'])
      if (bools.has(key)) {
        flags[key] = true
        continue
      }
      const val = args[i + 1]
      if (!val || val.startsWith('--')) {
        flags[key] = true
        continue
      }
      i++
      flags[key] = val
    } else {
      positionals.push(a)
    }
  }
  return { cmd, flags, positionals }
}

async function main() {
  const { cmd, flags, positionals } = parseArgs(process.argv)
  const lab = typeof flags.lab === 'string' ? flags.lab : labRoot()

  if (cmd === 'run') {
    const task = String(flags.task ?? '')
    const arm = String(flags.arm ?? 'sail')
    const toolchain = String(flags.toolchain ?? 'current')
    if (!task) {
      console.error('error: --task required')
      usage()
    }
    const modelSlug =
      (typeof flags.model === 'string' ? flags.model : null) ??
      process.env.SAIL_BENCH_MODEL ??
      (flags.seed ? 'seed' : null)
    if (!modelSlug) {
      console.error('error: --model or SAIL_BENCH_MODEL required (or pass --seed)')
      process.exit(2)
    }

    const subprocess = Boolean(flags.subprocess)
    // default in-process for speed; --subprocess for full contract
    const runner = new LocalRunner({
      lab,
      subprocess,
    })

    console.error(`== sail-bench run task=${task} arm=${arm} toolchain=${toolchain} model=${modelSlug} seed=${Boolean(flags.seed)} ==`)
    const outcome = await runner.runOne({
      taskId: task,
      arm,
      modelSlug,
      model: typeof flags.model === 'string' ? flags.model : modelSlug,
      toolchain,
      profileId: typeof flags.profile === 'string' ? flags.profile : 'p0',
      seed: Boolean(flags.seed),
      batchName: typeof flags.batch === 'string' ? flags.batch : undefined,
    })

    console.log(
      JSON.stringify(
        {
          batch_id: outcome.batchId,
          run_id: outcome.runId,
          run_dir: outcome.runDir,
          status: outcome.status,
          wall_ms: outcome.wall_ms,
          gate_ok: outcome.gate_ok,
          render_ok: outcome.render_ok,
          flags: outcome.flags,
          manifest: outcome.manifestPath,
        },
        null,
        2,
      ),
    )
    process.exit(outcome.status === 'complete' ? 0 : 1)
  }

  if (cmd === 'report') {
    const batch = String(flags.batch ?? positionals[0] ?? '')
    if (!batch) {
      console.error('error: --batch required')
      usage()
    }
    const r = writeBatchReport(lab, batch)
    console.log(
      JSON.stringify(
        {
          batch_id: r.batchId,
          report_dir: r.reportDir,
          index: r.indexPath,
          runs: r.runCount,
          copied_bytes: r.copiedBytes,
        },
        null,
        2,
      ),
    )
    process.exit(0)
  }

  if (cmd === 'grade') {
    const runRef = positionals[0]
    if (!runRef) {
      console.error('error: grade requires <run_id|run_dir>')
      usage()
    }
    const pairs: Array<{ key: string; value: string }> = []
    let note: string | undefined
    for (const p of positionals.slice(1)) {
      if (p.startsWith('note=')) {
        note = p.slice(5)
        continue
      }
      const eq = p.indexOf('=')
      if (eq < 1) {
        note = note ? `${note} ${p}` : p
        continue
      }
      pairs.push({ key: p.slice(0, eq), value: p.slice(eq + 1) })
    }
    if (pairs.length === 0) {
      console.error('error: expected key=value pairs')
      process.exit(2)
    }
    const store = RunStore.open(storePath(lab))
    const written = writeGrades(store, { runRef, pairs, note })
    store.close()
    console.log(JSON.stringify(written, null, 2))
    process.exit(0)
  }

  if (cmd === 'serve') {
    const port = flags.port ? Number(flags.port) : 0
    const s = await serveLab({ lab, port: Number.isFinite(port) ? port : 0 })
    console.error(`sail-bench serve http://127.0.0.1:${s.port}/ (lab=${lab})`)
    // keep alive
    await new Promise(() => {})
  }

  console.error(`unknown command: ${cmd}`)
  usage()
}

main().catch((e) => {
  console.error(e)
  process.exit(1)
})
