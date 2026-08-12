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
 * Wire: root `emit:package-dts` / before studio pure typecheck.
 */
import { execFileSync } from 'node:child_process'
import { existsSync, mkdirSync, writeFileSync, rmSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const TSC = join(ROOT, 'node_modules/.bin/tsc')

/** Packages Pure apps resolve into (studio listFiles / @ship imports). */
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
    dir: 'packages/components/sail/code-editor',
    // Missing generated .controller.ts — stub public types for Pure consumers.
    stubIndex: `/** Side-effect registration + ops surface (G2d Pure types). */
export type SailDiagnostic = {
  from: number
  to: number
  message: string
  severity?: 'error' | 'warning' | 'info' | 'hint'
}
export type SailCompletionSource = unknown
export type SailCodeEditorApi = {
  setDiagnostics?: (diags: SailDiagnostic[]) => void
}
export declare function createOpsAuthoringClient(...args: never[]): unknown
export declare function normalizeOpsBase(...args: never[]): string
export declare function offsetToLineCol(...args: never[]): { line: number; column: number }
export declare function parseOpsDiagnostics(...args: never[]): SailDiagnostic[]
export type OpsAuthoringClient = unknown
export type OpsCompletionItem = unknown
export type OpsDiagnostic = unknown
export type OpsHoverResult = unknown
`,
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

function emitOne(pkg) {
  const cwd = join(ROOT, pkg.dir)
  if (!existsSync(cwd)) {
    console.warn(`[emit-package-dts] skip missing ${pkg.dir}`)
    return
  }
  const dist = join(cwd, 'dist')
  mkdirSync(dist, { recursive: true })

  if (pkg.stubIndex) {
    writeFileSync(join(dist, 'index.d.ts'), pkg.stubIndex)
    console.log(`[emit-package-dts] stub ${pkg.dir}`)
    return
  }

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
      // tsc still emits with local errors; surface stdout/stderr but continue.
      const out = `${e.stdout ?? ''}${e.stderr ?? ''}`
      if (out.trim()) console.warn(`[emit-package-dts] ${pkg.dir}:\n${out.split('\n').slice(0, 12).join('\n')}`)
    } finally {
      try {
        rmSync(cfgPath)
      } catch {
        /* ignore */
      }
    }
  }

  if (!existsSync(join(dist, 'index.d.ts')) && !existsSync(join(dist, 'themes.d.ts'))) {
    if (existsSync(join(cwd, 'src/index.ts'))) {
      console.warn(`[emit-package-dts] WARN: no dist/index.d.ts for ${pkg.dir}`)
    }
  } else {
    console.log(`[emit-package-dts] ok ${pkg.dir}`)
  }
}

if (!existsSync(TSC)) {
  console.error('tsc not found at', TSC)
  process.exit(1)
}

for (const pkg of PACKAGES) emitOne(pkg)
console.log('[emit-package-dts] done')
