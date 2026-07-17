#!/usr/bin/env node
// Compiles every .sui under src/ via the sui-compiler binary so main.ts
// can side-effect-import the emitted ESM. Emits sibling <stem>/dist/.
//
// Uses the release sui-compiler from the framework's own target dir,
// so this example tracks whatever the local checkout is building.
// Override with SAIL_COMPILER=/path/to/sui-compiler if you want to
// point at a different binary.

import { execFileSync } from 'node:child_process'
import { existsSync, readdirSync, statSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { resolve, dirname, basename } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = resolve(__dirname, '..')
// Package lives at packages/sui/runtime/example/sui-app/; scripts/precompile.mjs
// → root is sui-app; repo root is four levels up.
const repoRoot = resolve(root, '..', '..', '..', '..')
const srcDir = resolve(root, 'src')

const compilerPath = process.env.SAIL_COMPILER ?? (() => {
  const release = resolve(repoRoot, 'target/release/sui-compiler')
  const debug = resolve(repoRoot, 'target/debug/sui-compiler')
  if (existsSync(release)) return release
  if (existsSync(debug)) return debug
  console.error(
    'sui-compiler not built. Run from the repo root:\n' +
    '  cargo build -p sui-compiler --release\n' +
    'or set SAIL_COMPILER to a binary path.',
  )
  process.exit(1)
})()

function* walk(dir) {
  for (const entry of readdirSync(dir)) {
    const p = resolve(dir, entry)
    const st = statSync(p)
    if (st.isDirectory()) {
      // Don't recurse into compiled output.
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
  // Emit alongside source in src/dist/ so the compiler-emitted
  // `import setup from "./<stem>.controller.ts"` resolves against
  // src/ when esbuild walks the import graph (`./` from src/dist/
  // → src/). Matches the Krausest harness layout; the only reason
  // we pass --out here is to override the compiler's default of
  // writing to <repo-root>/.ship/sail-out/, which is awkward for a
  // standalone example that lives in a tree containing a .ship/.
  const outDir = resolve(dirname(target), 'dist')
  mkdirSync(outDir, { recursive: true })
  console.log(`compiling ${target.replace(root + '/', '')}`)
  execFileSync(
    compilerPath,
    ['build', target, '--component-name', stem, '--out', outDir],
    { stdio: 'inherit' },
  )
}

// Same patch as the Krausest harness: the compiler hardcodes
// shadow: true for components but this demo renders into light DOM
// (no scoped CSS isolation; index.html's <style> targets selectors
// against the rendered tree directly). When the compiler grows a
// shadow="false" directive this patch can come out.
for (const target of targets) {
  const stem = basename(target, '.sui')
  // With --out, the compiler emits `<stem>.js` (legacy name), not
  // the `<stem>.compiled.js` it produces in mirror-tree mode.
  const compiledPath = resolve(dirname(target), 'dist', stem + '.js')
  let src = readFileSync(compiledPath, 'utf8')
  let changed = false
  if (src.includes('shadow: true')) {
    src = src.replace(/shadow: true,/g, 'shadow: false,')
    changed = true
    console.log(`  patched shadow:false in ${compiledPath.replace(root + '/', '')}`)
  }
  // Compiler bug workaround: with --out the emitted controller-import
  // stays sibling-relative (`./<stem>.controller.ts`) even though the
  // output is no longer alongside the source. Rewrite to `../` so the
  // import resolves to src/<stem>.controller.ts when esbuild walks
  // the graph. Without --out, the compiler emits `../<stem>.controller.ts`
  // correctly — see the Krausest harness — so this only matters in
  // example mode where we override the default output dir.
  const wrongImport = `from "./${stem}.controller.ts"`
  const rightImport = `from "../${stem}.controller.ts"`
  if (src.includes(wrongImport)) {
    src = src.replace(wrongImport, rightImport)
    changed = true
    console.log(`  fixed controller import in ${compiledPath.replace(root + '/', '')}`)
  }
  if (changed) writeFileSync(compiledPath, src, 'utf8')
}

console.log(`compiled ${targets.length} .sui file(s)`)
