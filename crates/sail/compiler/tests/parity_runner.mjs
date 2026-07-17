// Full DOM-level parity runner. For each fixture:
//   1. Read source.
//   2. Hydrate via @ship/sail-dom into a jsdom container.
//   3. Import the compiled JS, run its `__mount` (or component
//      registration) into a fresh jsdom container.
//   4. Compare outerHTML.
//
// Prerequisites:
//   - Workspace installed (bun install) so `@ship/sail-dom` and
//     `@ship/sail-runtime` resolve via TypeScript-aware loader (tsx).
//   - jsdom installed.
//
// Invoke with:
//   RUN_PARITY=1 cargo test -p sail-compiler --test parity \
//     full_parity_with_interpreter
//
// This script lives in-tree as the integration target. The Rust
// parity test is the entry point that wires it up.

import { mkdtempSync, readFileSync, writeFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { tmpdir } from 'node:os'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

const FIXTURES = [
  { name: 'hello',       mode: 'view',      setup: null },
  { name: 'counter',     mode: 'view',      setup: './counter.ts' },
  { name: 'agents',      mode: 'view',      setup: './agents.ts' },
  { name: 'ship-button', mode: 'component', setup: null },
]

async function main() {
  const { JSDOM } = await import('jsdom').catch(() => null) ?? { JSDOM: null }
  if (!JSDOM) {
    console.error('jsdom not installed; cannot run parity check')
    process.exit(2)
  }
  const { hydrate } = await import('@ship/sail-dom').catch(() => ({}))
  if (!hydrate) {
    console.error('@ship/sail-dom not resolvable; run bun install')
    process.exit(2)
  }

  let failed = 0
  for (const f of FIXTURES) {
    const sourcePath = join(__dirname, 'fixtures', `${f.name}.sail`)
    const source = readFileSync(sourcePath, 'utf8')

    // Interpreter side
    const dom1 = new JSDOM('<!doctype html><html><body><div id=x></div></body></html>')
    const container1 = dom1.window.document.getElementById('x')
    let setupFn
    if (f.setup) {
      const setupMod = await import(join(__dirname, 'fixtures', f.setup.replace('./', '')))
      setupFn = setupMod.default
    }
    hydrate(source, setupFn, { container: container1 })

    // Compiler side: compile via the rust binary.
    const tmp = mkdtempSync(join(tmpdir(), 'sail-parity-'))
    const { execSync } = await import('node:child_process')
    execSync(
      `cargo run -q -p sail-compiler -- build ${sourcePath} --out ${tmp} --no-setup`,
      { stdio: 'inherit' },
    )
    const compiledPath = join(tmp, `${f.name}.js`)
    const compiledMod = await import(compiledPath)
    const dom2 = new JSDOM('<!doctype html><html><body><div id=x></div></body></html>')
    const container2 = dom2.window.document.getElementById('x')
    if (f.mode === 'view') {
      const def = compiledMod.default
      const view = def.setup({ ship: {}, route: {}, navigate: () => {} })
      view.__mount(container2)
    } else {
      // component mode — registration happens by import; compare nothing.
      // Could instantiate `<ship-button-test>` and snapshot, but the
      // interpreter side doesn't render component instances either.
      continue
    }

    const a = container1.innerHTML.replace(/\s+/g, ' ').trim()
    const b = container2.innerHTML.replace(/\s+/g, ' ').trim()
    if (a !== b) {
      console.error(`PARITY DIFF for ${f.name}`)
      console.error('interpreter:', a)
      console.error('compiler   :', b)
      failed++
    } else {
      console.log(`ok ${f.name}`)
    }
  }
  if (failed > 0) {
    console.error(`${failed} fixture(s) diverged`)
    process.exit(1)
  }
}

main().catch((e) => {
  console.error(e)
  process.exit(1)
})
