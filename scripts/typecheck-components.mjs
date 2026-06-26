#!/usr/bin/env node
// Typecheck every sui component pack. Regenerates the adjacent `.sui.d.ts` type
// companions FIRST (they're gitignored, so a clean checkout has none) so each
// controller's `import type { Scope } from './x.sui'` resolves, then runs
// `tsc --noEmit` per pack. Wired as the root `typecheck` script.
import { execFileSync } from 'node:child_process'
import { existsSync, readdirSync, statSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..')

console.log('▸ generating .sui.d.ts companions')
execFileSync(process.execPath, [resolve(ROOT, 'scripts/gen-component-types.mjs')], { stdio: 'inherit' })

const tsc = resolve(ROOT, 'node_modules/.bin/tsc')
const PACKS = readdirSync(resolve(ROOT, 'packages/components/sui'))
  .map((d) => resolve(ROOT, 'packages/components/sui', d))
  .filter((d) => statSync(d).isDirectory() && existsSync(resolve(d, 'tsconfig.json')))

let failed = 0
for (const pack of PACKS) {
  console.log(`▸ tsc ${pack.split('/').pop()}`)
  try { execFileSync(tsc, ['--noEmit', '-p', 'tsconfig.json'], { cwd: pack, stdio: 'inherit' }) }
  catch { failed++ }
}
if (failed) { console.error(`✗ ${failed}/${PACKS.length} pack(s) failed typecheck`); process.exit(1) }
console.log(`✓ ${PACKS.length} pack(s) typecheck clean`)
