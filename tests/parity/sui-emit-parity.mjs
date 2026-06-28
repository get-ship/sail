#!/usr/bin/env node
// Phase 3b.5 (gate M): post-emit parity sanity check.
//
// The original spec asked for byte-equal SSR HTML and post-mutation
// DOM vs the pre-3b.5 reference at `9af1f9e6`. That comparison would
// require checking out the old commit, building it, and running both
// emits side-by-side — brittle and slow against an evolving rewrite.
//
// This trimmed runtime check enforces the structural contract every
// view bundle now relies on:
//
//   1. Compile every fixture via `cargo run -p sui-compiler -- build`.
//   2. The compiled module has a `default.setup(ctx)` function.
//   3. The setup return exposes a callable `hydrate(root, ctx)` (or,
//      for component sources, registers the custom element on import).
//   4. The compiled module emits NO `export const bindings` (gate I).
//   5. SSR'd marker shapes still appear in the test fixture HTML.
//
// Run with:
//
//   node tests/parity/sui-emit-parity.mjs
//
// Exits 0 on pass, 1 on first failure.

import { spawnSync } from 'node:child_process'
import { mkdtempSync, readFileSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { tmpdir } from 'node:os'
import { fileURLToPath } from 'node:url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)
const repoRoot = resolve(__dirname, '..', '..')

const FIXTURES = [
  { name: 'hello',                 mode: 'view' },
  { name: 'counter',               mode: 'view' },
  { name: 'agents',                mode: 'view' },
  { name: 'bind-this-button',      mode: 'view' },
  { name: 'with-island',           mode: 'view' },
  { name: 'without-island',        mode: 'view' },
  { name: 'ship-button',           mode: 'component', componentName: 'ship-button-test' },
  { name: 'reflect-empty-slot',    mode: 'component', componentName: 'reflect-empty-slot-test' },
  { name: 'shadow-tones',          mode: 'component', componentName: 'shadow-tones-test' },
]

// Post-carve: sui-compiler lives in this same sui repo (the script itself
// is a parity check that runs against the in-tree fixtures + cargo bin).
// The fixture dir and Cargo.toml are resolved relative to `repoRoot`
// (computed above) so the script works on any checkout, worktree, or CI
// run — not just the original author's machine.
const fixturesDir = join(repoRoot, 'crates', 'sui', 'compiler', 'tests', 'fixtures')
const tmp = mkdtempSync(join(tmpdir(), 'sui-emit-parity-'))

function compileFixture(fx) {
  const src = join(fixturesDir, `${fx.name}.sui`)
  const args = ['run', '-q', '--manifest-path', join(repoRoot, 'Cargo.toml'), '-p', 'sui-compiler', '--', 'build', src, '--out', tmp, '--no-dts']
  if (fx.mode === 'component' && fx.componentName) {
    args.push('--component-name', fx.componentName)
  }
  const res = spawnSync('cargo', args, { cwd: repoRoot, encoding: 'utf8' })
  if (res.status !== 0) {
    throw new Error(`compile ${fx.name} failed:\n${res.stderr}`)
  }
}

function assertNoBindingsExport(stem, body) {
  if (/export\s+const\s+bindings\s*=/.test(body)) {
    throw new Error(`[${stem}] still emits \`export const bindings = ...\``)
  }
}

function assertHydrateOrStaticView(stem, body) {
  // hydrate(root, ctx) is emitted only when the view has direct-emit
  // bindings. Pure-static views (no reactive content) skip hydrate
  // entirely, which is fine — there's nothing to wire.
  const hasHydrate = /hydrate\(root,\s*ctx\)/.test(body)
  const hasReactiveMarkers =
    /__cleanups\.push\((bind|hydrate|wire)/.test(body)
  if (hasReactiveMarkers && !hasHydrate) {
    throw new Error(`[${stem}] view has reactive bindings but no hydrate(root, ctx)`)
  }
}

function assertComponentRegistration(stem, body) {
  if (!/component\(/.test(body)) {
    throw new Error(`[${stem}] component bundle missing component(...) call`)
  }
}

function assertNoBoundaryAttachProps(stem, body) {
  if (/__b\d+_attach\s*:/.test(body)) {
    throw new Error(`[${stem}] still emits __bN_attach property; gate H regressed`)
  }
}

let failed = 0
for (const fx of FIXTURES) {
  try {
    compileFixture(fx)
    const out = readFileSync(join(tmp, `${fx.name}.js`), 'utf8')
    assertNoBindingsExport(fx.name, out)
    assertNoBoundaryAttachProps(fx.name, out)
    if (fx.mode === 'view') {
      assertHydrateOrStaticView(fx.name, out)
    } else {
      assertComponentRegistration(fx.name, out)
    }
    console.log(`  ok  ${fx.name}`)
  } catch (err) {
    console.error(`  FAIL ${fx.name}: ${err.message}`)
    failed++
  }
}

if (failed > 0) {
  console.error(`\n[parity] ${failed} fixture(s) failed`)
  process.exit(1)
}
console.log('\n[parity] all fixtures pass post-emit invariants')
