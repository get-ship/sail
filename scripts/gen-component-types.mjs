#!/usr/bin/env node
/**
 * scripts/gen-component-types.mjs — curated root helper (HYGIENE-0).
 *
 * Runs: invoked by scripts/typecheck-components.mjs (`bun run typecheck`),
 * and pack-level pretypecheck hooks when present.
 * Contract: emit adjacent `<stem>.sail.d.ts` companions for component packs
 * so controllers can `import type { Scope } from './x.sail'`.
 *
 * Keep scripts/ to load-bearing root helpers only — no one-shot generators.
 */
// Generate the `<stem>.sail.d.ts` type companions for every component pack so
// `tsc` can resolve each controller's `import type { Scope } from './<stem>.sail'`.
//
// WHY THIS EXISTS: a `.sail` template's `Scope` type is generated, not authored.
// The compiler emits `<stem>.sail.d.ts` ADJACENT to its `.sail` (a codegen
// companion, like a CSS-modules `.d.ts`), and the build artifacts (JS/CEM/…) to
// the pack's own `dist/` via `--out`. Components stay insulated from host
// dynamic-views output paths. Run before `typecheck` (wired as each pack's
// `pretypecheck`).
//
// Compiler: SAIL_COMPILER env, else the repo's debug build.

import { execFileSync } from 'node:child_process'
import { existsSync, readdirSync, statSync } from 'node:fs'
import { dirname, resolve, basename } from 'node:path'
import { fileURLToPath } from 'node:url'

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const COMPILER = process.env.SAIL_COMPILER
  ?? resolve(REPO_ROOT, 'target/debug/sail-compiler')
if (!existsSync(COMPILER)) {
  console.error(`sail-compiler not found at ${COMPILER}. Build it (cargo build -p sail-compiler) or set SAIL_COMPILER.`)
  process.exit(1)
}

// Pack roots: an explicit arg (a single pack dir), else every pack under
// packages/components/sail/*.
const arg = process.argv[2]
const PACKS = arg
  ? [resolve(arg)]
  : readdirSync(resolve(REPO_ROOT, 'packages/components/sail'))
      .map((d) => resolve(REPO_ROOT, 'packages/components/sail', d))
      .filter((d) => statSync(d).isDirectory() && existsSync(resolve(d, 'src')))

function* walkSui(dir) {
  for (const entry of readdirSync(dir)) {
    const p = resolve(dir, entry)
    const st = statSync(p)
    if (st.isDirectory()) {
      if (entry === 'dist' || entry === 'node_modules') continue
      yield* walkSui(p)
    } else if (entry.endsWith('.sail')) {
      yield p
    }
  }
}

let total = 0
for (const pack of PACKS) {
  const src = resolve(pack, 'src')
  if (!existsSync(src)) continue
  const dist = resolve(pack, 'dist')
  for (const sail of walkSui(src)) {
    const stem = basename(sail, '.sail')
    // `--out <pack>/dist`: build artifacts land in the pack's own dist; the
    // `.sail.d.ts` lands adjacent to the source regardless. (The compiler emits
    // no SSR skeletons — that path was removed in the JS-only SSR cutover — so
    // there's nothing to suppress here.)
    execFileSync(COMPILER, ['build', sail, '--component-name', stem, '--out', dist], {
      stdio: ['inherit', 'ignore', 'inherit'],
    })
    total++
  }
}
console.log(`generated types for ${total} component(s) across ${PACKS.length} pack(s)`)
