#!/usr/bin/env node
/**
 * Emit declaration-only `dist/` for monorepo packages that app Pure typecheck
 * follows via `@ship/*` (G2d / types→src residual).
 *
 * Pure (`lib: ESNext`, no DOM) cannot typecheck Framework package SOURCE.
 * `skipLibCheck` only applies to `.d.ts` — so package `types`/`exports.types`
 * must point at emitted declarations, not `src/*.ts`.
 *
 * Usage: `node scripts/emit-package-dts.mjs`
 * Wire: root `prepare` / `emit:package-dts` / `sail types` (monorepo) /
 *        before pure typecheck scripts.
 *
 * Exits non-zero if a listed package does not produce the expected
 * `dist/index.d.ts` (or package-specific allowlisted entry). Packages that use
 * checked-in public types (not dual `types→dist`) are not listed here.
 */
import { execFileSync } from 'node:child_process'
import { existsSync, mkdirSync, writeFileSync, rmSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const TSC = join(ROOT, 'node_modules/.bin/tsc')

/** Shared Framework declaration emit options (DOM allowed — these packages are Framework). */
const FRAMEWORK_EMIT = {
  target: 'ES2022',
  module: 'ESNext',
  moduleResolution: 'Bundler',
  lib: ['ES2022', 'DOM', 'DOM.Iterable'],
  strict: true,
  skipLibCheck: true,
  declaration: true,
  emitDeclarationOnly: true,
  noEmit: false,
  outDir: 'dist',
  rootDir: 'src',
  esModuleInterop: true,
  isolatedModules: true,
  resolveJsonModule: true,
}

/**
 * Packages Pure apps resolve into (studio listFiles / @ship imports).
 * Dual `exports.types → dist/*.d.ts`. Missing dist → TypeScript falls back to
 * Framework `src/` (pure regression) — emit must run before pure/IDE typecheck.
 *
 * Not listed (checked-in types, not dual dist):
 *   packages/components/sail/code-editor → types/index.d.ts
 *   (generated `.controller.ts` is gitignored; public surface is hand-maintained)
 */
const PACKAGES = [
  {
    dir: 'packages/sail/runtime',
    tempConfig: {
      compilerOptions: { ...FRAMEWORK_EMIT, exactOptionalPropertyTypes: true, noUncheckedIndexedAccess: true },
      include: ['src/**/*.ts'],
      exclude: ['dist', 'src/**/*.test.ts'],
    },
  },
  {
    dir: 'packages/sail/studio-client',
    tempConfig: {
      compilerOptions: { ...FRAMEWORK_EMIT },
      include: ['src/**/*.ts'],
      exclude: ['dist', 'test'],
    },
  },
  {
    dir: 'packages/sail/studio-shell',
    tempConfig: {
      compilerOptions: {
        ...FRAMEWORK_EMIT,
        exactOptionalPropertyTypes: true,
        noUncheckedIndexedAccess: true,
        verbatimModuleSyntax: true,
      },
      include: ['src/**/*.ts'],
      exclude: ['src/**/*.test.ts', 'dist'],
    },
  },
  {
    dir: 'packages/sail/tokens',
    // tokens also exports themes.d.ts; require index.d.ts for pure consumers.
    tempConfig: {
      compilerOptions: { ...FRAMEWORK_EMIT },
      include: ['src/**/*.ts'],
      exclude: ['dist'],
    },
  },
  {
    dir: 'packages/components/sail/primitives',
    tempConfig: {
      compilerOptions: { ...FRAMEWORK_EMIT },
      include: ['src/**/*.ts'],
      exclude: ['dist', '**/test/**', '**/*.test.ts'],
    },
  },
  {
    dir: 'packages/components/sail/viewer',
    tempConfig: {
      compilerOptions: { ...FRAMEWORK_EMIT },
      include: ['src/**/*.ts'],
      exclude: ['dist', '**/test/**', '**/*.test.ts'],
    },
  },
  {
    dir: 'packages/components/sail/chat-elements',
    tempConfig: {
      compilerOptions: { ...FRAMEWORK_EMIT },
      include: ['src/**/*.ts'],
      exclude: ['dist', '**/test/**', '**/*.test.ts'],
    },
  },
]

/** @returns {'ok' | 'fail' | 'skip'} */
function emitOne(pkg) {
  const cwd = join(ROOT, pkg.dir)
  if (!existsSync(cwd)) {
    console.error(`[emit-package-dts] FAIL: missing package dir ${pkg.dir}`)
    return 'fail'
  }
  const dist = join(cwd, 'dist')
  mkdirSync(dist, { recursive: true })

  if (pkg.tempConfig) {
    const cfgPath = join(cwd, 'tsconfig.emit-dts.json')
    writeFileSync(cfgPath, JSON.stringify(pkg.tempConfig, null, 2))
    try {
      execFileSync(TSC, ['-p', 'tsconfig.emit-dts.json'], {
        cwd,
        stdio: 'pipe',
        encoding: 'utf8',
      })
    } catch (e) {
      // tsc often still emits usable .d.ts with local errors; log and require
      // dist entry below. Missing entry → hard fail.
      const out = `${e.stdout ?? ''}${e.stderr ?? ''}`
      if (out.trim()) {
        console.warn(
          `[emit-package-dts] ${pkg.dir} (tsc non-zero, checking dist):\n${out
            .split('\n')
            .slice(0, 12)
            .join('\n')}`,
        )
      }
    } finally {
      try {
        rmSync(cfgPath)
      } catch {
        /* ignore */
      }
    }
  }

  const indexDts = join(dist, 'index.d.ts')
  if (!existsSync(indexDts)) {
    console.error(`[emit-package-dts] FAIL: no dist/index.d.ts for ${pkg.dir}`)
    return 'fail'
  }
  console.log(`[emit-package-dts] ok ${pkg.dir}`)
  return 'ok'
}

if (!existsSync(TSC)) {
  console.error('tsc not found at', TSC)
  process.exit(1)
}

let failed = 0
for (const pkg of PACKAGES) {
  if (emitOne(pkg) === 'fail') failed++
}
if (failed > 0) {
  console.error(`[emit-package-dts] failed for ${failed} package(s)`)
  process.exit(1)
}
console.log('[emit-package-dts] done')
