// Build the standalone SUI primitives bundle for the tutorial preview iframe.
//
// The preview iframe (public/tut-host.html) is its OWN document with its OWN
// customElements registry — web components must be registered THERE, not in the
// parent. So we bundle every `<sui-*>` primitive into one ESM the iframe loads,
// with ONLY the framework singletons externalized (@ship/sui-runtime + /route,
// /template-hooks, /patch, and @ship/sui-dom) — those resolve through the
// iframe's importmap to deck's served /_deck/runtime.js + /_deck/dom.js. Every
// other dependency is bundled in.
//
// Worktree-aware: compiles each primitive's `.sui` in-process via the wasm
// compiler (the same compiler the deck uses) and lets Bun bundle the emitted
// modules + their `.controller.ts` siblings. No `deck build-assets` (which
// resolves the topmost Cargo workspace, the wrong tree inside a git worktree).
//
// Run: `bun scripts/gen-tut-primitives.mjs` (from apps/sui-site). Wired into the
// package.json `gen` step.
import init, { compile } from '../../../packages/sui/compiler-wasm/pkg/sui_compiler.js'
import { readFileSync, writeFileSync, existsSync, readdirSync, statSync } from 'node:fs'
import { join, resolve, dirname, basename } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const REPO = resolve(__dirname, '../../..')
const PRIM_SRC = join(REPO, 'packages/components/sui/primitives/src')
const OUT = join(__dirname, '../public/tut-sui-primitives.js')

// Primitives excluded from the lesson-preview bundle: a spreadsheet formula
// engine (sui-sheet), the theme picker (pulls @ship/tokens JS — site chrome),
// and the virtualized data components (sui-table/tree/tree-grid) which import
// `@ship/sui-runtime/virtual` — a runtime subpath NOT in deck's served
// /_deck/runtime.js (it pulls @tanstack/virtual-core). None are lesson
// primitives; keeping them would break the whole bundle's module resolution.
const EXCLUDE = new Set([
  'sui-sheet',
  'sui-theme-picker',
  'sui-table',
  'sui-tree',
  'sui-tree-grid',
])

const EXTERNAL = [
  '@ship/sui-runtime',
  '@ship/sui-runtime/route',
  '@ship/sui-runtime/template-hooks',
  '@ship/sui-runtime/patch',
  '@ship/sui-dom',
]

// ── discover component `.sui` files (hyphenated stem, not page/index) ───────
function findSui(dir, out) {
  for (const name of readdirSync(dir)) {
    if (name === 'node_modules' || name === 'dist' || name.startsWith('.')) continue
    const p = join(dir, name)
    const st = statSync(p)
    if (st.isDirectory()) {
      if (EXCLUDE.has(name)) continue
      findSui(p, out)
    } else if (name.endsWith('.sui')) {
      const stem = basename(name, '.sui')
      if (stem.includes('-') && stem !== 'page' && stem !== 'index') out.push(p)
    }
  }
}

await init({ module_or_path: readFileSync(join(REPO, 'packages/sui/compiler-wasm/pkg/sui_compiler_bg.wasm')) })

const suiFiles = []
findSui(PRIM_SRC, suiFiles)
suiFiles.sort()

// Pre-compile each `.sui`; drop any that fail so one bad primitive never sinks
// the whole bundle. Keyed by absolute path → emitted component-mode JS.
const compiled = new Map()
const skipped = []
for (const p of suiFiles) {
  const tag = basename(p, '.sui')
  const src = readFileSync(p, 'utf8')
  // `mode: 'component'` + `shadow: true` emits a SELF-REGISTERING module — a
  // side-effecting `component(tag, def)` that calls `customElements.define(tag,
  // …)` on import (vs the mount-form, which only EXPORTS a definition). The
  // `shadow: true` flag is what the viewer's tag-mount passes for primitives
  // (they are shadow components) and what turns on the define side-effect.
  const opts = { mode: 'component', componentName: tag, shadow: true }
  if (existsSync(p.replace(/\.sui$/, '.controller.ts'))) opts.setupUrl = `./${tag}.controller`
  try {
    const out = compile(src, JSON.stringify(opts))
    if (out.kind !== 'ok') throw new Error(out.message)
    compiled.set(p, out.js)
  } catch (e) {
    skipped.push(`${tag}: ${String(e.message || e).slice(0, 60)}`)
  }
}

// ── Bun bundle: a virtual entry imports each compiled `.sui`; the plugin serves
//    the wasm-compiled JS, and the `import setup from './<tag>.controller'`
//    resolves to the sibling `.controller.ts` (Bun transpiles TS natively). ──
const entry = [...compiled.keys()].map((p) => `import ${JSON.stringify('suiprim:' + p)}`).join('\n')

const suiPlugin = {
  name: 'sui-primitive-compiler',
  setup(build) {
    build.onResolve({ filter: /^suiprim:/ }, (args) => ({
      path: args.path.slice('suiprim:'.length),
      namespace: 'suiprim',
    }))
    build.onLoad({ filter: /.*/, namespace: 'suiprim' }, (args) => ({
      contents: compiled.get(args.path) ?? '',
      loader: 'js',
      resolveDir: dirname(args.path),
    }))
  },
}

const result = await Bun.build({
  entrypoints: ['./__tut-primitives-entry.js'],
  external: EXTERNAL,
  target: 'browser',
  format: 'esm',
  minify: false,
  plugins: [
    {
      name: 'virtual-entry',
      setup(build) {
        build.onResolve({ filter: /^\.\/__tut-primitives-entry\.js$/ }, () => ({
          path: '__entry',
          namespace: 'virtual-entry',
        }))
        build.onLoad({ filter: /.*/, namespace: 'virtual-entry' }, () => ({
          contents: entry,
          loader: 'js',
          resolveDir: PRIM_SRC,
        }))
      },
    },
    suiPlugin,
  ],
})

if (!result.success) {
  console.error('[gen-tut-primitives] bundle failed:')
  for (const log of result.logs) console.error('  ', log.message)
  process.exit(1)
}

const code = await result.outputs[0].text()
writeFileSync(OUT, code)
console.log(
  `wrote ${OUT} — ${compiled.size} primitives bundled (${(code.length / 1024) | 0} KB)` +
    (skipped.length ? `, ${skipped.length} skipped` : ''),
)
if (skipped.length) console.log('  skipped:', skipped.join('; '))
