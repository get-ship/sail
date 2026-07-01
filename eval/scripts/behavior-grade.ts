#!/usr/bin/env bun
// behavior-grade.ts — score a built view against its behavioral spec.
//
// Runs the scenarios in a `*.behavior.jsonc` against a lane's compiled view in
// jsdom (via @ship/test-setup's runScenarios) and reports % passing. This is
// the behavioral dimension of the eval — "does it actually do what was asked,"
// not just "does it compile."
//
// Substrate: vitest provides the jsdom env + wasm-compiler aliases the harness
// needs, so we generate a one-off test under packages/test-setup/test/, run it,
// read the results it writes, and clean up. (The in-app author path is the same
// scenario format mounted in the Tauri webview — a separate runner.)
//
// Usage:
//   bun behavior-grade.ts <lane-worktree> <view-path> <behavior-spec.jsonc>
//   bun behavior-grade.ts ~/dev/ship-worktrees/eval-sonnet-XXXX \
//       views/authority eval/tasks/authority/authority.behavior.jsonc

import { execSync } from 'node:child_process'
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { join, resolve } from 'node:path'

const REPO = execSync('git rev-parse --show-toplevel', { encoding: 'utf8' }).trim()
const [laneArg, viewPath, specArg] = process.argv.slice(2)
if (!laneArg || !viewPath || !specArg) {
  console.error('usage: behavior-grade.ts <lane-worktree> <view-path> <behavior-spec.jsonc>')
  process.exit(2)
}
const lane = resolve(laneArg)
const specPath = resolve(specArg)
const pageSui = join(lane, viewPath, 'page.sui')
const controllerTs = join(lane, viewPath, 'controller.ts')

interface ScenarioResult { name: string; passed: boolean; failedStep?: number; error?: string }

function emit(results: ScenarioResult[]): void {
  const total = results.length
  const passed = results.filter((r) => r.passed).length
  const score = total === 0 ? 0 : passed / total
  console.log(JSON.stringify({ view: viewPath, total, passed, score, results }, null, 2))
}

if (!existsSync(pageSui)) {
  emit([{ name: 'view exists', passed: false, error: `no page.sui at ${pageSui}` }])
  process.exit(0)
}

const testDir = join(REPO, 'packages/test-setup/test')
const genFile = join(testDir, '_behavior-grade.gen.test.ts')
const laneName = laneArg.replace(/\/+$/, '').split('/').pop() ?? 'lane'
const outFile = join(REPO, '.artifacts/sui-eval-runs', `behavior-${laneName}.json`)
mkdirSync(join(REPO, '.artifacts/sui-eval-runs'), { recursive: true })

// A controller is optional — fall back to undefined if the lane didn't write one.
const hasController = existsSync(controllerTs)
const controllerImport = hasController
  ? `import controllerMod from ${JSON.stringify(controllerTs)}`
  : `const controllerMod = undefined`

const gen = `
import { test } from 'vitest'
import { readFileSync, writeFileSync } from 'node:fs'
import { runScenarios } from '../src/scenario-runner'
${controllerImport}

const stripJsonc = (s) => s.replace(/\\/\\/.*$/gm, '').replace(/\\/\\*[\\s\\S]*?\\*\\//g, '')

test('behavior grade (measurement, never fails)', async () => {
  const src = readFileSync(${JSON.stringify(pageSui)}, 'utf8')
  const spec = JSON.parse(stripJsonc(readFileSync(${JSON.stringify(specPath)}, 'utf8')))
  const ctrl = (controllerMod && (controllerMod.setup ? controllerMod : controllerMod.default)) ?? controllerMod
  let results
  try {
    results = await runScenarios(src, ctrl, spec)
  } catch (e) {
    results = spec.scenarios.map((s) => ({ name: s.name, passed: false, error: String(e) }))
  }
  writeFileSync(${JSON.stringify(outFile)}, JSON.stringify(results))
})
`

writeFileSync(genFile, gen)
// Repo-local vitest — npx could fetch a different major from the network.
const vitestBin = join(REPO, 'node_modules/.bin/vitest')
const runner = existsSync(vitestBin) ? JSON.stringify(vitestBin) : 'npx vitest'
try {
  execSync(`${runner} run ${JSON.stringify(genFile)}`, {
    cwd: join(REPO, 'packages/test-setup'),
    stdio: 'ignore',
  })
} catch {
  // vitest exit code is irrelevant — the test writes results regardless and is
  // a measurement, not a gate. A hard crash leaves no outFile → handled below.
} finally {
  rmSync(genFile, { force: true })
}

if (!existsSync(outFile)) {
  const spec = JSON.parse(stripComments(readFileSync(specPath, 'utf8')))
  emit((spec.scenarios ?? []).map((s: { name: string }) => ({ name: s.name, passed: false, error: 'grade run produced no output (view failed to load?)' })))
  process.exit(0)
}
const results = JSON.parse(readFileSync(outFile, 'utf8')) as ScenarioResult[]
rmSync(outFile, { force: true })
emit(results)

function stripComments(s: string): string {
  return s.replace(/\/\/.*$/gm, '').replace(/\/\*[\s\S]*?\*\//g, '')
}
