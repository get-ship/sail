// Compile a `.sail` fixture through the production CLI, then bun-build a
// browser entry that mounts the emitted module. The spec injects the bundle
// into Playwright so assertions run against real computed style + layout —
// not emit text. Work dir stays under this package so bun can resolve @ship/*.

import { execFileSync } from 'node:child_process'
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { basename, dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))
const REPO_ROOT = resolve(HERE, '../../../..')
const COMPILER = resolve(REPO_ROOT, 'target/debug/sail-compiler')
const COMPILED_ROOT = resolve(HERE, '.compiled')

function ensureCompiler(): void {
  if (existsSync(COMPILER)) return
  execFileSync('cargo', ['build', '--bin', 'sail-compiler', '--quiet'], {
    cwd: REPO_ROOT,
    stdio: 'inherit',
  })
}

function extractController(source: string): string | null {
  const m = /<controller\b[^>]*>([\s\S]*?)<\/controller>/.exec(source)
  return m ? m[1]!.trim() : null
}

function runCompile(sailPath: string, outDir: string): void {
  try {
    execFileSync(
      COMPILER,
      ['build', '--mode', 'auto', sailPath, '--out', outDir, '--no-dts'],
      { cwd: REPO_ROOT, stdio: 'pipe' },
    )
  } catch (err) {
    const e = err as { stderr?: Buffer; stdout?: Buffer; message?: string }
    throw new Error(
      e.stderr?.toString() || e.stdout?.toString() || e.message || String(err),
    )
  }
}

/** Token floor the compiler's CSS vars resolve against. Production ships
 *  these via `@ship/tokens`; the fixture only needs the handful it reads. */
export const TOKEN_FLOOR = `
:root {
  --space-4: 16px;
  --space-20: 80px;
  --space-48: 192px;
  --primary: rgb(200, 40, 20);
  --radius-md: 6px;
}
html, body { margin: 0; }
* { box-sizing: content-box; }
`

export function pageHtml(title: string): string {
  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>${title}</title>
    <style>${TOKEN_FLOOR}</style>
  </head>
  <body>
    <div id="root"></div>
  </body>
</html>`
}

/** Compile `sailPath` and return a browser bundle that mounts into `#root`. */
export function compileAndBundle(sailPath: string): string {
  ensureCompiler()
  const stem = basename(sailPath, '.sail')
  const work = join(COMPILED_ROOT, stem)
  rmSync(work, { recursive: true, force: true })
  mkdirSync(work, { recursive: true })

  runCompile(sailPath, work)

  const source = readFileSync(sailPath, 'utf8')
  const controller = extractController(source)
  if (controller) {
    writeFileSync(join(work, `${stem}.sail.controller.ts`), controller)
  }

  const entry = join(work, 'entry.ts')
  writeFileSync(
    entry,
    `import compiled from './${stem}.js'
import { route } from '@ship/sail-runtime/route'

const host = document.getElementById('root')
if (!host) throw new Error('[sail-e2e] #root missing')
const scope = compiled.setup({
  route: route({ url: new URL('http://localhost/') }),
  navigate: () => {},
})
scope.__mount(host)
`,
  )

  const out = join(work, 'bundle.js')
  try {
    execFileSync('bun', ['build', entry, '--target=browser', '--outfile', out], {
      cwd: resolve(HERE, '..'),
      stdio: 'pipe',
    })
  } catch (err) {
    const e = err as { stderr?: Buffer; stdout?: Buffer; message?: string }
    throw new Error(
      e.stderr?.toString() || e.stdout?.toString() || e.message || String(err),
    )
  }
  return readFileSync(out, 'utf8')
}
