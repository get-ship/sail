#!/usr/bin/env node
// Drive `sail-compiler profile` and land the JSON under tests/perf/baselines.
// Continues past per-file compile errors (the Rust driver does not fail-fast).
//
//   bun tests/perf/scripts/profile-compiler.mjs
//   bun tests/perf/scripts/profile-compiler.mjs --skip-dev-rss
//   bun tests/perf/scripts/profile-compiler.mjs --sail-dev-ram
//   bun tests/perf/scripts/profile-compiler.mjs --out tests/perf/baselines/custom.json
//
// `--sail-dev-ram` is PERF-7: steady-state sail-dev + Studio-tab RSS
// (see profile-sail-dev-ram.mjs). It does not run the compile() walk.

import { spawnSync } from 'node:child_process'
import { existsSync, mkdirSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const REPO_ROOT = resolve(__dirname, '../../..')
const DEFAULT_OUT = resolve(REPO_ROOT, 'tests/perf/baselines/2026-08-13-compiler.json')

const skipDevRss = process.argv.includes('--skip-dev-rss')
const sailDevRam = process.argv.includes('--sail-dev-ram')
const outIdx = process.argv.indexOf('--out')
const out = outIdx >= 0 ? resolve(process.argv[outIdx + 1]) : DEFAULT_OUT

if (sailDevRam) {
  const ramOut =
    outIdx >= 0 ? out : resolve(REPO_ROOT, 'tests/perf/baselines/2026-08-13-ram.json')
  const ram = spawnSync(
    process.execPath,
    [resolve(__dirname, 'profile-sail-dev-ram.mjs'), '--out', ramOut],
    { cwd: REPO_ROOT, stdio: 'inherit' },
  )
  process.exit(ram.status ?? 1)
}

mkdirSync(dirname(out), { recursive: true })

const cargo = spawnSync(
  'cargo',
  ['build', '-p', 'sail-compiler', '--bin', 'sail-compiler'],
  { cwd: REPO_ROOT, stdio: 'inherit' },
)
if (cargo.status !== 0) {
  process.exit(cargo.status ?? 1)
}

const bin = resolve(REPO_ROOT, 'target/debug/sail-compiler')
if (!existsSync(bin)) {
  console.error(`missing ${bin}`)
  process.exit(1)
}

const args = ['profile', '--workspace', REPO_ROOT, '--out', out]
if (skipDevRss) args.push('--skip-dev-rss')

const profile = spawnSync(bin, args, { cwd: REPO_ROOT, stdio: 'inherit' })
if (profile.status !== 0) {
  process.exit(profile.status ?? 1)
}

console.log(`wrote ${out}`)
