#!/usr/bin/env bun
/**
 * PARITY-1 umbrella — geometry (structure gate) + paint (structure gate).
 *
 * Forwards argv flags to both suites (--report-only, --strict, --fixture, …).
 * Exit 1 if either half fails.
 *
 *   bun tests/parity/run.ts
 *   bun tests/parity/run.ts --report-only
 *   bun tests/parity/run.ts --strict
 */

import { spawnSync } from 'node:child_process'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { existsSync } from 'node:fs'

const __dirname = dirname(fileURLToPath(import.meta.url))
const repoRoot = resolve(__dirname, '../..')
const forwarded = process.argv.slice(2)

const geometry = join(__dirname, 'geometry/run.ts')
const paint = join(__dirname, 'paint/run.ts')

function run(label: string, script: string): number {
  console.log(`\n======== PARITY ${label} ========`)
  const r = spawnSync('bun', [script, ...forwarded], {
    cwd: repoRoot,
    stdio: 'inherit',
    env: process.env,
  })
  const code = r.status == null ? 1 : r.status
  console.log(`======== ${label} exit ${code} ========`)
  return code
}

if (!existsSync(geometry) || !existsSync(paint)) {
  console.error('error: missing geometry or paint suite under tests/parity/')
  process.exit(1)
}

const g = run('geometry', geometry)
const p = run('paint', paint)
const code = g !== 0 || p !== 0 ? 1 : 0
console.log(`\nPARITY umbrella: geometry=${g} paint=${p} → exit ${code}`)
process.exit(code)
