#!/usr/bin/env node
/**
 * scripts/typecheck-components.mjs — curated root helper (HYGIENE-0).
 *
 * Runs: `bun run typecheck` / `npm run typecheck` (package.json).
 * Contract: regenerate adjacent `.sail.d.ts` companions, then `tsc --noEmit`
 * per component pack. Invokes scripts/gen-component-types.mjs first.
 *
 * Keep scripts/ to load-bearing root helpers only — no one-shot generators.
 */
// Typecheck every sail component pack. Regenerates the adjacent `.sail.d.ts` type
// companions FIRST (they're gitignored, so a clean checkout has none) so each
// controller's `import type { Scope } from './x.sail'` resolves, then runs
// `tsc --noEmit` per pack. Wired as the root `typecheck` script.
import { execFileSync } from 'node:child_process'
import { existsSync, readdirSync, statSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..')

console.log('▸ generating .sail.d.ts companions')
execFileSync(process.execPath, [resolve(ROOT, 'scripts/gen-component-types.mjs')], { stdio: 'inherit' })

const tsc = resolve(ROOT, 'node_modules/.bin/tsc')
const PACKS = readdirSync(resolve(ROOT, 'packages/components/sail'))
  .map((d) => resolve(ROOT, 'packages/components/sail', d))
  .filter((d) => statSync(d).isDirectory() && existsSync(resolve(d, 'tsconfig.json')))

let failed = 0
for (const pack of PACKS) {
  console.log(`▸ tsc ${pack.split('/').pop()}`)
  try { execFileSync(tsc, ['--noEmit', '-p', 'tsconfig.json'], { cwd: pack, stdio: 'inherit' }) }
  catch { failed++ }
}
if (failed) { console.error(`✗ ${failed}/${PACKS.length} pack(s) failed typecheck`); process.exit(1) }
console.log(`✓ ${PACKS.length} pack(s) typecheck clean`)
