#!/usr/bin/env node
// Sync the owned keyed/sail bench app into an external js-framework-benchmark
// checkout. Single source of truth: tests/perf/frameworks/keyed/sail in this
// repo. $BENCH_HOME/frameworks/keyed/sail is a sync OUTPUT — never hand-edit it.
//
// Also removes the stale $BENCH_HOME/frameworks/keyed/sail tree (pre-rename
// leftover). No webdriver run; this is copy + cleanup only.
//
// Usage:
//   bun tests/perf/scripts/sync-keyed-sail.mjs
//   BENCH_HOME=/path/to/js-framework-benchmark bun tests/perf/scripts/sync-keyed-sail.mjs
//   bun tests/perf/scripts/sync-keyed-sail.mjs --dry-run

import {
  cpSync,
  existsSync,
  lstatSync,
  mkdirSync,
  readdirSync,
  rmSync,
  statSync,
} from 'node:fs'
import { homedir } from 'node:os'
import { dirname, join, relative, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const REPO_ROOT = resolve(__dirname, '../../..')
const OWNED = resolve(REPO_ROOT, 'tests/perf/frameworks/keyed/sail')

const dryRun = process.argv.includes('--dry-run') || process.env.DRY_RUN === '1'

const BENCH_HOME = resolve(
  process.env.BENCH_HOME
    ?? join(homedir(), 'dev/js-framework-benchmark'),
)

const DEST = resolve(BENCH_HOME, 'frameworks/keyed/sail')
const STALE_SAIL = resolve(BENCH_HOME, 'frameworks/keyed/sail')

// Authored set only — never push build state into the checkout as source of truth.
const SKIP_DIR_NAMES = new Set(['node_modules', 'dist', '.git'])
const SKIP_FILE_NAMES = new Set(['bun.lock', 'bun.lockb'])

function die(msg) {
  console.error(`sync-keyed-sail: ${msg}`)
  process.exit(1)
}

function log(msg) {
  console.log(msg)
}

if (!existsSync(OWNED) || !statSync(OWNED).isDirectory()) {
  die(`owned source missing: ${OWNED}`)
}

if (!existsSync(BENCH_HOME)) {
  die(
    `BENCH_HOME does not exist: ${BENCH_HOME}\n` +
      `Clone js-framework-benchmark there, or set BENCH_HOME=/path/to/checkout.`,
  )
}

const keyedRoot = resolve(BENCH_HOME, 'frameworks/keyed')
if (!existsSync(keyedRoot)) {
  die(`not a js-framework-benchmark checkout (no frameworks/keyed): ${BENCH_HOME}`)
}

/** @returns {string[]} absolute paths of files to copy, relative paths for display */
function collectOwnedFiles(dir, acc = []) {
  for (const entry of readdirSync(dir)) {
    if (SKIP_DIR_NAMES.has(entry) || SKIP_FILE_NAMES.has(entry)) continue
    const p = resolve(dir, entry)
    let st
    try {
      st = lstatSync(p)
    } catch {
      continue
    }
    if (st.isSymbolicLink()) continue
    if (st.isDirectory()) {
      collectOwnedFiles(p, acc)
    } else if (st.isFile()) {
      acc.push(p)
    }
  }
  return acc
}

// --- remove stale keyed/sail -------------------------------------------------
if (existsSync(STALE_SAIL)) {
  if (dryRun) {
    log(`[dry-run] would remove stale ${STALE_SAIL}`)
  } else {
    log(`removing stale keyed/sail → ${STALE_SAIL}`)
    rmSync(STALE_SAIL, { recursive: true, force: true })
  }
} else {
  log(`no stale keyed/sail at ${STALE_SAIL}`)
}

// --- mirror owned → keyed/sail ---------------------------------------------
const files = collectOwnedFiles(OWNED)
if (files.length === 0) die(`no files under ${OWNED}`)

if (dryRun) {
  log(`[dry-run] would sync ${files.length} file(s)`)
  log(`  from ${OWNED}`)
  log(`  to   ${DEST}`)
  for (const f of files) {
    log(`  + ${relative(OWNED, f)}`)
  }
  log('[dry-run] done (no writes)')
  process.exit(0)
}

// Wipe prior sync output so deleted owned files do not linger. Keep
// node_modules/ and dist/ if present (local build state in the checkout).
mkdirSync(resolve(BENCH_HOME, 'frameworks/keyed'), { recursive: true })
if (existsSync(DEST)) {
  for (const entry of readdirSync(DEST)) {
    if (entry === 'node_modules' || entry === 'dist') continue
    rmSync(resolve(DEST, entry), { recursive: true, force: true })
  }
} else {
  mkdirSync(DEST, { recursive: true })
}

for (const src of files) {
  const rel = relative(OWNED, src)
  const out = resolve(DEST, rel)
  mkdirSync(dirname(out), { recursive: true })
  cpSync(src, out)
  log(`  synced ${rel}`)
}

log(`synced ${files.length} file(s) → ${DEST}`)
log('next (once, in the checkout): cd "$BENCH_HOME/frameworks/keyed/sail" && bun install')
log('optional smoke (in this repo): SAIL_REPO=<this-repo> bun tests/perf/frameworks/keyed/sail/scripts/precompile.mjs')
