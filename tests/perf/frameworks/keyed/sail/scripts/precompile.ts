#!/usr/bin/env bun
// Compiles every .sail under src/ via the sail-compiler binary so main.ts can
// side-effect-import the emitted ESM (which registers the custom element).
//
// OWNERSHIP: the canonical copy of this file (and the rest of the keyed/sail
// bench app) lives in this repo at tests/perf/frameworks/keyed/sail/ and is
// synced into the external js-framework-benchmark checkout by
// tests/perf/scripts/sync-keyed-sail.ts. Edit it here, not in the checkout.
// Product apps use `sail create`, not this harness.
//
// This app lives OUTSIDE the sail checkout, so in-tree compiler discovery doesn't
// apply. Rather than hardcode an absolute path (a cutover landmine — the sail
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

// --- resolve the sail checkout ----------------------------------------------
// Precedence: explicit SAIL_REPO env → granular SAIL_RUNTIME/SAIL_COMPILER envs →
// discovery across conventional locations. Moving the checkout then needs only
// `SAIL_REPO=/new/path` (or nothing, if it lands in a probed location).
function firstExisting(paths) {
  for (const p of paths) if (p && existsSync(p)) return p
  return null
}
function discoverSailRepo() {
  const probes = [
    process.env.SAIL_REPO,
    resolve(homedir(), 'dev/sail'),
    resolve(homedir(), 'dev/Ship/sail'),
    // This worktree / monorepo root when precompile is run in-tree:
    resolve(__dirname, '../../../../..'),
  ].filter(Boolean)
  return probes.find((p) => existsSync(resolve(p, 'packages/sail/runtime'))) ?? null
}

const SAIL_REPO = discoverSailRepo()

// Resolve @ship/sail-runtime to the live sail checkout WITHOUT a package.json
// `file:` dep — that drags in the runtime's `workspace:*` devDeps, which don't
// resolve outside the monorepo. A direct symlink gives esbuild the current
// source AND lets the bench runner read the version from its package.json.
const RUNTIME_SRC = process.env.SAIL_RUNTIME
  ?? (SAIL_REPO ? resolve(SAIL_REPO, 'packages/sail/runtime') : null)
if (!RUNTIME_SRC || !existsSync(RUNTIME_SRC)) {
  console.error(
    'sail runtime source not found. Set SAIL_REPO to your sail checkout root\n' +
    '(expects <SAIL_REPO>/packages/sail/runtime), or SAIL_RUNTIME to the runtime\n' +
    'directory directly.',
  )
  process.exit(1)
}

const shipModules = resolve(root, 'node_modules/@ship')
mkdirSync(shipModules, { recursive: true })
const runtimeLink = resolve(shipModules, 'sail-runtime')

// (Re)link @ship/sail-runtime → the resolved runtime. Recreate when missing OR
// stale (dangling / points elsewhere) — a moved sail checkout would otherwise
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
    'under your sail checkout, or set SAIL_COMPILER to a binary path.' +
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
    } else if (entry.endsWith('.sail')) {
      yield p
    }
  }
}

const targets = [...walk(srcDir)]
if (targets.length === 0) {
  console.error('no .sail files found under', srcDir)
  process.exit(1)
}

// Type gate needs tsgo. The bench checkout is not the monorepo — put the
// Sail repo's node_modules/.bin on PATH so the gate still runs.
const sailBin = SAIL_REPO ? resolve(SAIL_REPO, 'node_modules/.bin') : null
const compileEnv = {
  ...process.env,
  ...(sailBin && existsSync(sailBin)
    ? { PATH: `${sailBin}${process.platform === 'win32' ? ';' : ':'}${process.env.PATH ?? ''}` }
    : {}),
}

for (const target of targets) {
  const stem = basename(target, '.sail')
  const outDir = resolve(dirname(target), 'dist')
  mkdirSync(outDir, { recursive: true })
  console.log(`compiling ${target.replace(root + '/', '')}`)
  execFileSync(compiler, ['build', target, '--component-name', stem, '--out', outDir], {
    stdio: 'inherit',
    env: compileEnv,
  })
}

// `--out` writes JS that imports a VIRTUAL `./<file>.sail.controller.ts`
// (deck's bundler extracts the inline <controller>). esbuild cannot. Materialize
// the block next to the JS so the specifier resolves. Driver chrome (classes,
// tr.danger, delete span) is NOT applied here — that lives in the benchmark
// checkout's sail-driver.
for (const target of targets) {
  const stem = basename(target, '.sail')
  const fileName = basename(target)
  const outDir = resolve(dirname(target), 'dist')
  const compiledPath = resolve(outDir, stem + '.js')
  const sailSrc = readFileSync(target, 'utf8')
  const ctrl = sailSrc.match(/<controller>\s*([\s\S]*?)<\/controller>/)
  if (ctrl) {
    writeFileSync(resolve(outDir, `${fileName}.controller.ts`), ctrl[1]!.trimStart(), 'utf8')
  }
  let src = readFileSync(compiledPath, 'utf8')
  if (src.includes('shadow: true')) {
    writeFileSync(compiledPath, src.replace(/shadow: true,/g, 'shadow: false,'), 'utf8')
  }
}

console.log(`compiled ${targets.length} .sail file(s)`)
