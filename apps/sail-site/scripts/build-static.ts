#!/usr/bin/env bun
// Wrapper around product emit: `sail build`.
//
//   sail build --bundle-framework --base assets --out dist
//   + recursive public/ copy (bundler only copies top-level files)
//   + inventory of what landed (HTML / assets / wasm / icons / fonts)
//
// Resolves the binary from SAIL_BIN, then `sail` on PATH. Repo
// `target/debug/sail` is a convenience fallback if that file exists —
// never the documented required path.
import { spawnSync } from 'node:child_process'
import { existsSync, mkdirSync, readdirSync, statSync, copyFileSync, writeFileSync, readFileSync } from 'node:fs'
import { dirname, isAbsolute, join, relative, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))
const APP = resolve(HERE, '..')
const REPO = resolve(APP, '../..')
const OUT = resolve(process.env.SAIL_SITE_STATIC_OUT ?? join(APP, 'dist'))
const WASM_PKG = join(REPO, 'packages/sail/compiler-wasm/pkg/sail_compiler_bg.wasm')
const DEBUG_SAIL = join(REPO, 'target/debug/sail')

function run(cmd, args, opts = {}) {
  const r = spawnSync(cmd, args, { stdio: 'inherit', cwd: REPO, ...opts })
  if (r.status !== 0) {
    throw new Error(`${cmd} ${args.join(' ')} exited ${r.status}`)
  }
}

function walk(dir, acc = []) {
  if (!existsSync(dir)) return acc
  for (const name of readdirSync(dir)) {
    const p = join(dir, name)
    const st = statSync(p)
    if (st.isDirectory()) walk(p, acc)
    else acc.push(p)
  }
  return acc
}

function copyTree(src, dest) {
  if (!existsSync(src)) return 0
  let n = 0
  for (const file of walk(src)) {
    const rel = relative(src, file)
    const dst = join(dest, rel)
    mkdirSync(dirname(dst), { recursive: true })
    copyFileSync(file, dst)
    n += 1
  }
  return n
}

function classify(rel) {
  if (rel === 'index.html') return 'html'
  if (rel.endsWith('.wasm')) return 'wasm'
  if (rel.startsWith('_deck/icons/')) return 'icon'
  if (rel.endsWith('.woff2') || rel.startsWith('fonts/') || rel.startsWith('_deck/fonts/')) {
    return 'font'
  }
  if (rel.startsWith('assets/')) return 'asset'
  if (rel.startsWith('_deck/')) return 'framework'
  if (rel === '_headers' || rel === '_redirects') return 'cf-config'
  return 'public'
}

function sailArgs() {
  return [
    'build',
    '--project',
    APP,
    '--out',
    OUT,
    '--base',
    'assets',
    '--bundle-framework',
  ]
}

/** Inventory must stay machine-portable: `sail` or a path relative to the repo. */
function portableVia(bin) {
  if (!bin || bin === 'sail') return 'sail'
  const abs = isAbsolute(bin) ? bin : resolve(REPO, bin)
  const rel = relative(REPO, abs)
  if (rel && !rel.startsWith('..') && !isAbsolute(rel)) return rel.replaceAll('\\', '/')
  return 'sail'
}

function sailOnPath() {
  const pathEnv = process.env.PATH ?? ''
  const sep = process.platform === 'win32' ? ';' : ':'
  for (const dir of pathEnv.split(sep)) {
    if (!dir) continue
    if (existsSync(join(dir, 'sail'))) return true
  }
  return false
}

function resolveSail() {
  const envBin = process.env.SAIL_BIN?.trim()
  if (envBin) return { cmd: envBin, prefix: [], via: portableVia(envBin) }

  const forceCargo = process.env.SAIL_SITE_FORCE_CARGO === '1'
  if (!forceCargo && sailOnPath()) {
    return { cmd: 'sail', prefix: [], via: 'sail' }
  }
  // Monorepo convenience only — never the documented required path.
  if (!forceCargo && existsSync(DEBUG_SAIL)) {
    return { cmd: DEBUG_SAIL, prefix: [], via: 'target/debug/sail' }
  }
  return {
    cmd: 'cargo',
    prefix: ['run', '-q', '-p', 'deck-cli', '--bin', 'sail', '--'],
    via: 'cargo-run',
  }
}

function runSailBuild() {
  const { cmd, prefix, via } = resolveSail()
  run(cmd, [...prefix, ...sailArgs()])
  return via
}

function ensureWasmPublic() {
  if (!existsSync(WASM_PKG)) {
    if (process.env.SAIL_SITE_SKIP_WASM === '1') {
      console.warn('build-static: wasm pkg missing and SAIL_SITE_SKIP_WASM=1')
      return
    }
    run('bun', ['run', 'wasm'], { cwd: APP })
  }
  const dest = join(APP, 'public', 'sail_compiler_bg.wasm')
  if (existsSync(WASM_PKG)) {
    copyFileSync(WASM_PKG, dest)
  }
}

/** Icon collector walks routes/ + pack sources, not src/component-docs.
 *  Copy lucide SVGs named in the app so catalog glyphs resolve offline. */
function stageReferencedIcons(outDir) {
  const lucide = join(REPO, 'node_modules/lucide-static/icons')
  const dest = join(outDir, '_deck/icons')
  if (!existsSync(lucide)) {
    console.warn('build-static: lucide-static/icons missing — skip icon backfill')
    return 0
  }
  mkdirSync(dest, { recursive: true })
  const names = new Set()
  for (const root of [join(APP, 'src'), join(APP, 'routes')]) {
    for (const file of walk(root)) {
      if (!file.endsWith('.sail') && !file.endsWith('.ts')) continue
      const src = readFileSync(file, 'utf8')
      for (const m of src.matchAll(/\bname="([a-z0-9-]+)"/g)) names.add(m[1])
    }
  }
  let n = 0
  for (const name of names) {
    const src = join(lucide, `${name}.svg`)
    const dst = join(dest, `${name}.svg`)
    if (existsSync(src) && !existsSync(dst)) {
      copyFileSync(src, dst)
      n += 1
    }
  }
  return n
}

function inventory() {
  const files = walk(OUT).map((p) => relative(OUT, p).replaceAll('\\', '/'))
  files.sort()
  const buckets = {
    html: [],
    wasm: [],
    icon: [],
    font: [],
    asset: [],
    framework: [],
    'cf-config': [],
    public: [],
  }
  for (const rel of files) buckets[classify(rel)].push(rel)
  const report = {
    out: relative(REPO, OUT).replaceAll('\\', '/'),
    fileCount: files.length,
    buckets: Object.fromEntries(
      Object.entries(buckets).map(([k, v]) => [k, { count: v.length, files: v }]),
    ),
    notes: {
      htmlPerRoute: false,
      spaShell: files.includes('index.html'),
      prerender: false,
      ssr: false,
    },
  }
  return report
}

mkdirSync(OUT, { recursive: true })
if (process.env.SAIL_SITE_SKIP_GEN !== '1') {
  // Prop registry / tutorial / grammar are already in-tree when iterating.
  // Full `bun run gen` rebuilds wasm + grammar; skip unless asked.
}
ensureWasmPublic()
const via = runSailBuild()
const copied = copyTree(join(APP, 'public'), OUT)
const iconsBackfilled = stageReferencedIcons(OUT)
const report = inventory()
report.buildVia = via
report.publicCopied = copied
report.iconsBackfilled = iconsBackfilled
const reportPath = join(APP, '.static-inventory.json')
writeFileSync(reportPath, JSON.stringify(report, null, 2) + '\n')
console.log(`build-static: ${report.fileCount} files → ${OUT}`)
console.log(
  `  html=${report.buckets.html.count} assets=${report.buckets.asset.count} ` +
    `_deck=${report.buckets.framework.count} wasm=${report.buckets.wasm.count} ` +
    `icons=${report.buckets.icon.count} fonts=${report.buckets.font.count} ` +
    `public=${report.buckets.public.count} cf=${report.buckets['cf-config'].count}`,
)
console.log(`  inventory: ${reportPath}`)
console.log(`  spa shell only — no HTML per route (CSR + host SPA fallback)`)
