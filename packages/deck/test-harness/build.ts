#!/usr/bin/env bun
// Build the browser harness entries (ses + deck-host + the real runtime/dom
// inlined) into dist/. Test infrastructure, not a shipped artifact. This build
// uses Bun.build for convenience; it is dev/CI tooling and ships in nothing
// (see #163 — bun is dev-only).

import { copyFile, mkdir, readFile, rm } from 'node:fs/promises'
import { existsSync } from 'node:fs'
import { dirname, join } from 'node:path'

const root = dirname(Bun.fileURLToPath(import.meta.url))
const dist = join(root, 'dist')
const repoRoot = join(root, '..', '..', '..')

// Inline a REAL compiled view as the confinement subject (slice 2). Prefer the
// .deck build output; the test asserts it confines + evaluates in-browser.
const viewPath = join(repoRoot, '.deck/views/git/view.js')
if (!existsSync(viewPath)) {
  console.error(`harness: missing ${viewPath} — run the bundler (just dev-build-all) first`)
  process.exit(1)
}
const gitViewSource = await readFile(viewPath, 'utf8')

await rm(dist, { recursive: true, force: true })
await mkdir(dist, { recursive: true })

const r = await Bun.build({
  entrypoints: [
    join(root, 'browser/lockdown-smoke.ts'),
    join(root, 'browser/view-confine.ts'),
    join(root, 'browser/bench-confine.ts'),
  ],
  outdir: dist,
  target: 'browser',
  format: 'esm',
  minify: false,
  sourcemap: 'none',
  // Inject the real view source as a compile-time constant (74KB string).
  define: { __GIT_VIEW_SOURCE__: JSON.stringify(gitViewSource) },
})
if (!r.success) {
  for (const l of r.logs) console.error(String(l))
  process.exit(1)
}

// HTML wrappers are tracked source (browser/*.html); copy next to the bundles.
for (const html of ['index.html', 'view-confine.html', 'bench.html']) {
  await copyFile(join(root, 'browser', html), join(dist, html))
}
process.stdout.write(`✓ harness build → ${dist}\n`)
