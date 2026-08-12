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
 * Exits non-zero if any declared dual `types` path under `dist/` is missing
 * after emit (root `types` + every `exports.*.types` / nested condition that
 * points at `./dist/...`). Packages that use checked-in public types (not dual
 * `types→dist`) are not listed here.
 */
import { execFileSync } from 'node:child_process'
import { existsSync, mkdirSync, readFileSync, writeFileSync, rmSync } from 'node:fs'
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
 * Hard-fail requires **all** package.json `types` / `exports.*.types` paths under
 * `dist/` (not only `dist/index.d.ts`), so multi-entry dual packages
 * (tokens themes/unions, runtime internal/virtual, viewer wasm-bootstrap, …)
 * cannot silently leave a subpath resolving to Framework src.
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

/**
 * Normalize a package-relative types path to a path under the package root.
 * Accepts `./dist/foo.d.ts`, `dist/foo.d.ts`. Returns null if not under dist/.
 * @param {unknown} typesPath
 * @returns {string | null}
 */
function distTypesRel(typesPath) {
  if (typeof typesPath !== 'string') return null
  const rel = typesPath.replace(/^\.\//, '')
  if (!rel.startsWith('dist/') && rel !== 'dist') return null
  return rel
}

/**
 * Collect every dual-export types path under dist/ from package.json.
 * Walks top-level `types` and `exports` (string | { types } | nested conditions).
 * @param {Record<string, unknown>} pkgJson
 * @returns {string[]} package-relative paths like `dist/themes.d.ts`
 */
function collectDistTypesEntries(pkgJson) {
  /** @type {Set<string>} */
  const out = new Set()

  const top = distTypesRel(pkgJson.types)
  if (top) out.add(top)

  /**
   * @param {unknown} node
   */
  function walkExports(node) {
    if (node == null) return
    if (typeof node === 'string') {
      // Bare string export targets (e.g. "./themes.css") are not types paths.
      return
    }
    if (Array.isArray(node)) {
      for (const item of node) walkExports(item)
      return
    }
    if (typeof node !== 'object') return
    /** @type {Record<string, unknown>} */
    const obj = node
    if ('types' in obj) {
      const t = distTypesRel(obj.types)
      if (t) out.add(t)
    }
    for (const [key, val] of Object.entries(obj)) {
      if (key === 'types') continue
      // Subpath keys (".", "./themes") or conditions (default, import, …)
      walkExports(val)
    }
  }

  walkExports(pkgJson.exports)
  return [...out].sort()
}

/**
 * After emit, require every package.json dual types path under dist/ to exist.
 * @param {string} pkgDir package dir relative to ROOT
 * @param {string} cwd absolute package dir
 * @returns {string[]} missing package-relative paths
 */
function missingDistTypesEntries(pkgDir, cwd) {
  const pkgJsonPath = join(cwd, 'package.json')
  if (!existsSync(pkgJsonPath)) {
    return ['package.json']
  }
  /** @type {Record<string, unknown>} */
  let pkgJson
  try {
    pkgJson = JSON.parse(readFileSync(pkgJsonPath, 'utf8'))
  } catch {
    return ['package.json (unreadable)']
  }

  const required = collectDistTypesEntries(pkgJson)
  // Dual packages always need at least index; if package.json has no dist types
  // entries at all, still require dist/index.d.ts so misconfigured packages fail.
  if (required.length === 0) {
    required.push('dist/index.d.ts')
  }

  return required.filter((rel) => !existsSync(join(cwd, rel)))
}

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
      // all dual dist types entries below. Missing entry → hard fail.
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

  const missing = missingDistTypesEntries(pkg.dir, cwd)
  if (missing.length > 0) {
    console.error(
      `[emit-package-dts] FAIL: missing dual types for ${pkg.dir}:\n${missing
        .map((m) => `  - ${m}`)
        .join('\n')}`,
    )
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
