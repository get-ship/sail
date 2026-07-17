#!/usr/bin/env node
// Compiles every .sui under src/ via the sail-compiler binary so main.ts can
// side-effect-import the emitted ESM (which registers the custom element).
// Adapted from packages/sail/runtime/example/sail-app/scripts/precompile.mjs.
//
// OWNERSHIP: the canonical copy of this file (and the rest of the keyed/sui
// bench app) lives in the Ship repo at tests/perf/frameworks/keyed/sail/ and is
// synced into the external js-framework-benchmark checkout by the bench harness
// (scripts/dev/bench-lib.mjs) before each build. Edit it there, not in the
// checkout — the checkout copy is overwritten on the next run.
//
// This app lives OUTSIDE the sui checkout, so in-tree compiler discovery doesn't
// apply. Rather than hardcode an absolute path (a cutover landmine — the sui
// checkout moves), resolve a single SAIL_REPO root by env or by probing
// conventional locations, then derive the compiler + runtime from it.

import { execFileSync } from 'node:child_process'
import {
  existsSync, readdirSync, statSync, lstatSync, mkdirSync,
  readFileSync, writeFileSync, readlinkSync, symlinkSync, rmSync,
} from 'node:fs'
import { homedir } from 'node:os'
import { resolve, dirname, basename } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = resolve(__dirname, '..')
const srcDir = resolve(root, 'src')

// --- resolve the sui checkout ------------------------------------------------
// Precedence: explicit SAIL_REPO env → granular SAIL_RUNTIME/SAIL_COMPILER envs →
// discovery across conventional locations. Moving the sui checkout then needs
// only `SAIL_REPO=/new/path` (or nothing, if it lands in a probed location).
function firstExisting(paths) {
  for (const p of paths) if (p && existsSync(p)) return p
  return null
}
function discoverSuiRepo() {
  const probes = [
    process.env.SAIL_REPO,
    resolve(homedir(), 'dev/sui'),
    resolve(homedir(), 'dev/Ship/sui'), // post-carve: sui nested under Ship
  ].filter(Boolean)
  return probes.find((p) => existsSync(resolve(p, 'packages/sail/runtime'))) ?? null
}

const SAIL_REPO = discoverSuiRepo()

// Resolve @ship/sail-runtime to the live sui checkout WITHOUT a package.json
// `file:` dep — that drags in the runtime's `workspace:*` devDeps, which don't
// resolve outside the sui monorepo. A direct symlink gives esbuild the current
// source AND lets the bench runner read the version from its package.json.
const RUNTIME_SRC = process.env.SAIL_RUNTIME
  ?? (SAIL_REPO ? resolve(SAIL_REPO, 'packages/sail/runtime') : null)
if (!RUNTIME_SRC || !existsSync(RUNTIME_SRC)) {
  console.error(
    'sui runtime source not found. Set SAIL_REPO to your sui checkout root\n' +
    '(expects <SAIL_REPO>/packages/sail/runtime), or SAIL_RUNTIME to the runtime\n' +
    'directory directly.',
  )
  process.exit(1)
}

const shipModules = resolve(root, 'node_modules/@ship')
mkdirSync(shipModules, { recursive: true })
const runtimeLink = resolve(shipModules, 'sail-runtime')

// (Re)link @ship/sail-runtime → the resolved runtime. Recreate when missing OR
// stale (dangling / points elsewhere) — a moved sui checkout would otherwise
// leave a dead symlink esbuild can't follow.
let linkOk = false
try {
  linkOk = lstatSync(runtimeLink).isSymbolicLink()
    && readlinkSync(runtimeLink) === RUNTIME_SRC
    && existsSync(runtimeLink)
} catch { linkOk = false }
if (!linkOk) {
  try { rmSync(runtimeLink, { recursive: true, force: true }) } catch {}
  symlinkSync(RUNTIME_SRC, runtimeLink, 'dir')
  console.log(`linked @ship/sail-runtime -> ${RUNTIME_SRC}`)
}

const compiler = process.env.SAIL_COMPILER ?? firstExisting([
  SAIL_REPO && resolve(SAIL_REPO, 'target/release/sail-compiler'),
  SAIL_REPO && resolve(SAIL_REPO, 'target/debug/sail-compiler'),
])
if (!compiler || !existsSync(compiler)) {
  console.error(
    'sail-compiler not found. Build it (cargo build -p sail-compiler [--release])\n' +
    'under your sui checkout, or set SAIL_COMPILER to a binary path.' +
    (SAIL_REPO ? `\n(searched ${SAIL_REPO}/target/{release,debug}/sail-compiler)` : ''),
  )
  process.exit(1)
}

function* walk(dir) {
  for (const entry of readdirSync(dir)) {
    const p = resolve(dir, entry)
    const st = statSync(p)
    if (st.isDirectory()) {
      if (entry === 'dist') continue
      yield* walk(p)
    } else if (entry.endsWith('.sui')) {
      yield p
    }
  }
}

const targets = [...walk(srcDir)]
if (targets.length === 0) {
  console.error('no .sui files found under', srcDir)
  process.exit(1)
}

for (const target of targets) {
  const stem = basename(target, '.sui')
  const outDir = resolve(dirname(target), 'dist')
  mkdirSync(outDir, { recursive: true })
  console.log(`compiling ${target.replace(root + '/', '')}`)
  execFileSync(compiler, ['build', target, '--component-name', stem, '--out', outDir], {
    stdio: 'inherit',
  })
}

// Render into LIGHT DOM so the benchmark runner's selectors (#run, tbody tr,
// td.col-md-1, …) resolve — the compiler hardcodes shadow:true. Also rewrite
// the sibling-relative controller import to `../` since --out moves the output
// out of src/. Both patches mirror the example app + the original harness.
for (const target of targets) {
  const stem = basename(target, '.sui')
  const compiledPath = resolve(dirname(target), 'dist', stem + '.js')
  let src = readFileSync(compiledPath, 'utf8')
  let changed = false
  if (src.includes('shadow: true')) {
    src = src.replace(/shadow: true,/g, 'shadow: false,')
    changed = true
  }
  const wrongImport = `from "./${stem}.controller.ts"`
  const rightImport = `from "../${stem}.controller.ts"`
  if (src.includes(wrongImport)) {
    src = src.replace(wrongImport, rightImport)
    changed = true
  }
  if (changed) writeFileSync(compiledPath, src, 'utf8')
}

console.log(`compiled ${targets.length} .sui file(s)`)
