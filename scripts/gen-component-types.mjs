#!/usr/bin/env node
// Generate the `<stem>.sui.d.ts` type companions for every component pack so
// `tsc` can resolve each controller's `import type { Scope } from './<stem>.sui'`.
//
// WHY THIS EXISTS: a `.sui` template's `Scope` type is generated, not authored.
// The compiler emits `<stem>.sui.d.ts` ADJACENT to its `.sui` (a codegen
// companion, like a CSS-modules `.d.ts`), and the build artifacts (JS/CEM/…) to
// the pack's own `dist/` via `--out`. Nothing here touches `.ship/sail-out` —
// that path is the Ship dynamic-views output, owned by Ship, and components must
// stay insulated from it. Run before `typecheck` (wired as each pack's
// `pretypecheck`).
//
// Compiler: SAIL_COMPILER env, else the repo's debug build.

import { execFileSync } from 'node:child_process'
import { existsSync, readdirSync, statSync } from 'node:fs'
import { dirname, resolve, basename } from 'node:path'
import { fileURLToPath } from 'node:url'

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const COMPILER = process.env.SAIL_COMPILER
  ?? resolve(REPO_ROOT, 'target/debug/sui-compiler')
if (!existsSync(COMPILER)) {
  console.error(`sui-compiler not found at ${COMPILER}. Build it (cargo build -p sui-compiler) or set SAIL_COMPILER.`)
  process.exit(1)
}

// Pack roots: an explicit arg (a single pack dir), else every pack under
// packages/components/sui/*.
const arg = process.argv[2]
const PACKS = arg
  ? [resolve(arg)]
  : readdirSync(resolve(REPO_ROOT, 'packages/components/sui'))
      .map((d) => resolve(REPO_ROOT, 'packages/components/sui', d))
      .filter((d) => statSync(d).isDirectory() && existsSync(resolve(d, 'src')))

function* walkSui(dir) {
  for (const entry of readdirSync(dir)) {
    const p = resolve(dir, entry)
    const st = statSync(p)
    if (st.isDirectory()) {
      if (entry === 'dist' || entry === 'node_modules') continue
      yield* walkSui(p)
    } else if (entry.endsWith('.sui')) {
      yield p
    }
  }
}

let total = 0
for (const pack of PACKS) {
  const src = resolve(pack, 'src')
  if (!existsSync(src)) continue
  const dist = resolve(pack, 'dist')
  for (const sui of walkSui(src)) {
    const stem = basename(sui, '.sui')
    // `--out <pack>/dist`: build artifacts land in the pack's own dist; the
    // `.sui.d.ts` lands adjacent to the source regardless. (The compiler emits
    // no SSR skeletons — that path was removed in the JS-only SSR cutover — so
    // there's nothing to suppress here.)
    execFileSync(COMPILER, ['build', sui, '--component-name', stem, '--out', dist], {
      stdio: ['inherit', 'ignore', 'inherit'],
    })
    total++
  }
}
console.log(`generated types for ${total} component(s) across ${PACKS.length} pack(s)`)
