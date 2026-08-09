// Shared vitest config for SAIL component packages.
//
// One factory, `sailComponentVitestConfig()`, replaces the ~90-line
// `compiled-js-test-shim` plugin that used to be copy-pasted into every
// component package's `vitest.config.ts` — plus the depth-counted relative
// imports (`../../../vitest.shared`, `../../../scripts/...`) that broke the
// moment a package moved between directories.
//
// EVERYTHING is resolved by package name or by walking up to the repo root, so
// a config authored with this factory is location-independent: move the package
// anywhere under `packages/` and its tests keep working with zero edits.

import { defineConfig } from 'vitest/config'
import type { Plugin, ViteUserConfig as VitestUserConfig } from 'vitest/config'
import { createRequire } from 'node:module'
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs'
import { resolve, dirname, basename, relative } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'
// @ts-expect-error — JS module, no .d.ts.
import { spawnBoundedOrThrow } from './spawn-bounded.mjs'
import { nativeCompileJson } from '@ship/sail-dom/test-utils'

const require = createRequire(import.meta.url)

/** Known root package names across Ship monorepos. Extend here when a new
 *  repo is carved out so all carve-offs share one test-setup build. */
const KNOWN_ROOT_NAMES = new Set(['ship-monorepo', 'ship-kit'])

/** Walk up from this module until we hit the workspace root package.json.
 *  A root is identified by a known package name (see KNOWN_ROOT_NAMES) or,
 *  more generally, by the presence of a `"workspaces"` field — in EITHER the
 *  npm/yarn array form (`["packages/*"]`) or the bun/ship-apps object form
 *  (`{ "packages": [...] }`). Repo-agnostic: any consuming repo with a
 *  workspaces root works, not just the original Ship monorepo. Never throws —
 *  the root is only used for Vite's fs allow-list and optional native-compile
 *  lookups, both of which tolerate an approximate root, so a standalone
 *  consumer falls back to the highest package.json dir (or cwd). */
function findRepoRoot(): string {
  let dir = dirname(fileURLToPath(import.meta.url))
  let highestWithPkg: string | null = null
  for (let i = 0; i < 20; i++) {
    const pkg = resolve(dir, 'package.json')
    if (existsSync(pkg)) {
      highestWithPkg = dir
      try {
        const manifest = JSON.parse(readFileSync(pkg, 'utf8')) as {
          name?: string
          workspaces?: unknown
        }
        if (KNOWN_ROOT_NAMES.has(manifest.name ?? '') || manifest.workspaces != null)
          return dir
      } catch {
        /* keep walking */
      }
    }
    const parent = dirname(dir)
    if (parent === dir) break
    dir = parent
  }
  return highestWithPkg ?? process.cwd()
}

const repoRoot = findRepoRoot()
// Product emit root (SPEC-OUTPUT-1). MUST match
// `sail_compiler::out_layout::DEFAULT_OUT_DIR` / `deck_config::DECK_OUT_DIRNAME`
// (`.deck`) — no second default (`.sail-out` is gone).
const mirrorRoot = resolve(repoRoot, '.deck')
const compilerBin = resolve(repoRoot, 'target/debug/sail-compiler')
// The native/CLI sail-compiler is only reachable in a repo that carries the
// Rust workspace (a root `Cargo.toml`) or a prebuilt binary. A JS-only
// consumer (e.g. ship-apps post-carve) has neither, so `.sail` sources compile
// through the inline WASM fallback instead of shelling out to `cargo`.
const nativeCompileAvailable =
  existsSync(compilerBin) || existsSync(resolve(repoRoot, 'Cargo.toml'))

// Honesty guard — runs `expect.hasAssertions()` before every test so a body
// that asserts nothing FAILS instead of passing green. Prepended to every
// package's setupFiles (its own setup.ts still runs after). Resolved by
// package subpath so it survives the package moving, same as @ship/sail-* below.
const assertHonestySetup = require.resolve('@ship/test-setup/assert-honesty')

const MAX_FORKS = Number(process.env.VITEST_MAX_FORKS ?? 2)

/** Workers + timeouts. vitest defaults to one fork per core; each jsdom fork
 *  holds ~500 MB, so an unbounded run exhausts memory on WSL2. Cap it once. */
export const SHIP_VITEST_DEFAULTS: NonNullable<VitestUserConfig['test']> = {
  pool: 'forks',
  maxForks: MAX_FORKS,
  minForks: 1,
  testTimeout: 10_000,
  hookTimeout: 10_000,
  // Zero-collection guard (runtime half): fail the run if it collected no
  // tests at all, instead of vitest's default green-on-empty. The static
  // `scripts/check-zero-collection.mjs` catches the per-file empty case; this
  // catches a whole suite that silently collected nothing.
  passWithNoTests: false,
}

// ── WASM fallback for the compiled-js shim ──────────────────────────────────
// Resolved through the `@ship/sail-compiler-wasm` package, NOT a hardcoded
// `packages/sail/compiler-wasm/pkg` path — so it survives the package moving.
let wasmCompile: ((source: string, optsJson: string) => unknown) | null = null
function ensureWasm(): (source: string, optsJson: string) => unknown {
  if (wasmCompile) return wasmCompile
  let jsPath: string
  let wasmPath: string
  try {
    jsPath = require.resolve('@ship/sail-compiler-wasm')
    wasmPath = require.resolve('@ship/sail-compiler-wasm/sail_compiler_bg.wasm')
  } catch {
    throw new Error('[test-setup/vitest] sail-compiler-wasm not built. Run `just sail-wasm`.')
  }
  const mod = require(jsPath) as {
    initSync: (input: { module: Buffer }) => unknown
    compile: (source: string, optsJson: string) => unknown
  }
  mod.initSync({ module: readFileSync(wasmPath) })
  wasmCompile = mod.compile
  return wasmCompile
}

/** Compile a `.sail` file to its emitted JS module. Throws on compile error.
 *  Light DOM is the default. Pass `shadow: true` only for residual foreign-
 *  content containment tests that still attach a shadow root. */
export function compileSailToJs(sailPath: string, opts: { shadow?: boolean } = {}): string {
  const compile = ensureWasm()
  const source = readFileSync(sailPath, 'utf8')
  const compileOpts: Record<string, unknown> = { mode: 'auto' }
  if (opts.shadow) compileOpts.shadow = true
  const result = compile(source, JSON.stringify(compileOpts)) as
    | { kind: 'ok'; js: string }
    | { kind: 'err'; message: string; line: number | null; col: number | null }
  if (result.kind === 'err') {
    throw new Error(
      `[test-setup/vitest] sail-compiler error in ${sailPath} (${result.line ?? '?'}:${result.col ?? '?'}): ${result.message}`,
    )
  }
  return result.js
}

function compileSail(sailPath: string, stem: string): void {
  const hasBin = existsSync(compilerBin)
  const cmd = hasBin ? compilerBin : 'cargo'
  const args = hasBin
    ? ['build', sailPath, '--component-name', stem, '--no-dts']
    : ['run', '--quiet', '-p', 'sail-compiler', '--', 'build', sailPath, '--component-name', stem, '--no-dts']
  spawnBoundedOrThrow(cmd, args, `sail-compiler ${sailPath}`, { cwd: repoRoot })
}

function mirrorArtifactFor(sourceDir: string, stem: string): string {
  return resolve(mirrorRoot, relative(repoRoot, sourceDir), `${stem}.compiled.js`)
}

// ── Shared Vite plugin factories ────────────────────────────────────────────
//
// `pinImportMetaUrlPlugin` and `shadowForComponentCompilePlugin` were
// previously copy-pasted into every component package that mounts
// renderComponent-based tests (sail-media, sail-primitives, ship-viewer).
// Export them here so each vitest.config.ts calls the factory instead.

/**
 * Pin `import.meta.url` in spec files to the literal `file://` URL of the
 * test file, BEFORE Vite's `assetImportMetaUrl` plugin rewrites
 * `new URL('<lit>', import.meta.url)` to `self.location` (an http URL under
 * happy-dom, which `readFileSync` rejects).
 *
 * @param name - Component name used as the plugin name prefix (e.g. `'sail-media'`).
 */
export function pinImportMetaUrlPlugin(name: string): Plugin {
  return {
    name: `${name}:pin-import-meta-url`,
    enforce: 'pre',
    transform(code: string, id: string) {
      const path = id.split('?', 1)[0] ?? id
      // Narrow on purpose: only spec files directly under a test/ dir — the
      // byte-identical behavior of the inline plugins this factory replaced
      // (SD-7). A broad /\.test\.ts$/ would also rewrite engine/helper specs
      // that never asked for the pin.
      if (!/[\\/]test[\\/][^\\/]+\.test\.ts$/.test(path)) return null
      if (!code.includes('import.meta.url')) return null
      const literal = JSON.stringify(pathToFileURL(path).href)
      return { code: code.split('import.meta.url').join(literal), map: null }
    },
  }
}

/**
 * Residual test helper: inject `shadow: true` into component-mode compile
 * opts for packages that still exercise foreign-content shadow wrappers.
 * Light DOM is the Sail product model; prefer tests that query the host.
 * Component mode only — route compiles are untouched.
 *
 * @param name - Component name used as the plugin name prefix (e.g. `'sail-media'`).
 */
export function shadowForComponentCompilePlugin(name: string): Plugin {
  return {
    name: `${name}:shadow-for-component-compile`,
    enforce: 'pre',
    transform(code: string, id: string) {
      if (!/[\\/]test-utils[\\/]compile-sail\.ts$/.test(id)) return null
      const needle = 'setupUrl: opts.setupMarker,'
      if (!code.includes(needle)) return null
      return {
        code: code.replace(needle, `${needle}\n    shadow: opts.componentName != null,`),
        map: null,
      }
    },
  }
}

// The @ship/* specifiers that renderRoute/renderComponent (and the SAIL runtime's
// internal subpaths) need pinned to ONE resolved instance. Subpaths first (most
// specific) so they win over the bare-name aliases. A JS-only consumer may not
// ship every runtime subpath (`./route`, `./virtual` were dropped in the
// sail→deck closure), so `resolveSailAliases` skips whatever doesn't resolve and
// lets node resolution cover the rest — keeping one factory across repos.
const SUI_ALIAS_SPECS = [
  '@ship/sail-dom/test-utils',
  '@ship/sail-runtime/template-hooks',
  '@ship/deck-router',
  '@ship/sail-runtime/route',
  '@ship/sail-runtime/virtual',
  '@ship/sail-runtime/internal',
  '@ship/sail-runtime/actions',
  '@ship/sail-runtime',
  '@ship/sail-dom',
] as const

function resolveSailAliases(): Record<string, string> {
  const out: Record<string, string> = {}
  for (const spec of SUI_ALIAS_SPECS) {
    try {
      out[spec] = require.resolve(spec)
    } catch {
      /* not present in this repo — let node resolution handle it */
    }
  }
  return out
}

// ── Workspace-wide @ship/* alias table (exports-map driven) ─────────────────
//
// `resolveSailAliases` above pins a CURATED set of specifiers through
// `require.resolve` (node's own `exports` resolution) — that's why every
// component package authored with `sailComponentVitestConfig()` resolves subpath
// imports fine. Two hand-written vitest configs did NOT go through the factory:
// test-setup's own and deck-eval's. They listed the runtime by BARE name plus a
// couple of subpaths and silently missed the rest (`/route`, `/server`, `/dev`,
// `/patch`, `/virtual`, and `@ship/tokens/themes`). An unlisted subpath like
// `@ship/sail-runtime/route` then fell through to the bare `@ship/sail-runtime`
// string alias and resolved to `…/src/index.ts/route` — a path that doesn't
// exist — so vitest reported "Failed to resolve import".
//
// `shipWorkspaceAliases()` derives the COMPLETE alias table straight from every
// workspace package's `exports` map, so every published entry (bare, subpath,
// or wildcard) maps to its real source file with zero per-config maintenance.
// It reads the `exports` maps but never mutates them — they stay correct for
// publish; this only mirrors them for vitest.

/** Expand the repo-root `workspaces` globs to a `name → package dir` map for
 *  every `@ship/*` workspace package. Only a trailing `/*` glob is expanded
 *  (the sole form the repo uses); anything unreadable is skipped. */
function discoverShipPackages(): Map<string, string> {
  const out = new Map<string, string>()
  let globs: string[] = []
  try {
    const rootPkg = JSON.parse(readFileSync(resolve(repoRoot, 'package.json'), 'utf8')) as {
      workspaces?: string[] | { packages?: string[] }
    }
    const ws = rootPkg.workspaces
    globs = Array.isArray(ws) ? ws : (ws?.packages ?? [])
  } catch {
    return out
  }
  const dirs: string[] = []
  for (const glob of globs) {
    if (glob.endsWith('/*')) {
      const base = resolve(repoRoot, glob.slice(0, -2))
      try {
        for (const entry of readdirSync(base, { withFileTypes: true }))
          if (entry.isDirectory()) dirs.push(resolve(base, entry.name))
      } catch {
        /* missing workspace dir — skip */
      }
    } else {
      dirs.push(resolve(repoRoot, glob))
    }
  }
  for (const dir of dirs) {
    try {
      const pkg = JSON.parse(readFileSync(resolve(dir, 'package.json'), 'utf8')) as { name?: string }
      if (typeof pkg.name === 'string' && pkg.name.startsWith('@ship/')) out.set(pkg.name, dir)
    } catch {
      /* not a package dir — skip */
    }
  }
  return out
}

/** Resolve one `exports` value to its target path string. Plain-string targets
 *  pass through; a conditional object picks the first source-like condition. */
function pickExportTarget(value: unknown): string | null {
  if (typeof value === 'string') return value
  if (value && typeof value === 'object') {
    const conds = value as Record<string, unknown>
    for (const key of ['import', 'module', 'browser', 'default', 'node', 'require', 'types']) {
      const target = conds[key]
      if (typeof target === 'string') return target
      if (target && typeof target === 'object') {
        const nested = pickExportTarget(target)
        if (nested) return nested
      }
    }
  }
  return null
}

const escapeRegExp = (s: string): string => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')

/**
 * The full `resolve.alias` table for a vitest run, derived from every
 * `@ship/*` workspace package's `exports` map. Each export key becomes an alias:
 * `.` → the bare package name, `./route` → `<name>/route`, and a wildcard like
 * `./lessons/*` → a regex that captures the trailing segment. Subpath entries
 * sort BEFORE the bare name (longest `find` first) so `@ship/sail-runtime/route`
 * can never fall through to the `@ship/sail-runtime` alias. Location-independent:
 * every path is resolved from the package's real directory (walked from the
 * repo root), so a config that spreads this list works from any package.
 */
export function shipWorkspaceAliases(): Array<{ find: string | RegExp; replacement: string }> {
  const stringAliases: Array<{ find: string; replacement: string }> = []
  const regexAliases: Array<{ find: RegExp; replacement: string }> = []
  for (const [name, dir] of discoverShipPackages()) {
    let exportsMap: Record<string, unknown> | undefined
    try {
      const pkg = JSON.parse(readFileSync(resolve(dir, 'package.json'), 'utf8')) as {
        exports?: Record<string, unknown>
      }
      exportsMap = pkg.exports
    } catch {
      /* unreadable — skip */
    }
    if (!exportsMap || typeof exportsMap !== 'object') continue
    for (const [key, value] of Object.entries(exportsMap)) {
      const target = pickExportTarget(value)
      if (!target) continue
      // '.' → bare name, './route' → '<name>/route', './lessons/*' → wildcard.
      const spec = key === '.' ? name : name + key.slice(1)
      if (key.includes('*')) {
        const findPrefix = spec.slice(0, spec.indexOf('*'))
        const replacement = resolve(dir, target).replace('*', '$1')
        regexAliases.push({ find: new RegExp(`^${escapeRegExp(findPrefix)}(.*)$`), replacement })
      } else {
        stringAliases.push({ find: spec, replacement: resolve(dir, target) })
      }
    }
  }
  // Longest `find` first → a subpath alias always wins over the bare-name one.
  stringAliases.sort((a, b) => b.find.length - a.find.length)
  return [...regexAliases, ...stringAliases]
}

export interface SailComponentVitestOptions {
  /** Extra `resolve.alias` entries merged over the SAIL defaults. */
  alias?: Record<string, string>
  /** Override the default test setup files. */
  setupFiles?: string[]
  /** Override the default include globs. */
  include?: string[]
  /** DOM environment. Defaults to `jsdom`. Use `happy-dom` for packs whose
   *  components inject modern scoped CSS — jsdom 25's parser throws
   *  "Could not parse CSS stylesheet" on mount; happy-dom tolerates it. */
  environment?: 'jsdom' | 'happy-dom'
}

/**
 * The one config every SAIL component package's `vitest.config.ts` should use:
 *
 * ```ts
 * import { sailComponentVitestConfig } from '@ship/test-setup/vitest'
 * export default sailComponentVitestConfig()
 * ```
 */
/** The SAIL source loader — deck-parity, one compiler, one controller path.
 *
 *  `.compiled.js` is the published-name convention for a sibling `.sail`; the
 *  `.sail` compiles in-process through the linked addon with the source's own
 *  policy (`sail.config.json` walk-up), so the floor applies here exactly as it
 *  does in deck and the CLI, and an unmigrated component fails with the
 *  compiler's own message. A source's `<controller>` block is served as the
 *  virtual `<file>.sail.controller.ts` — the specifier deck and the CLI emit —
 *  which vite transforms as TS and whose imports resolve like any module's.
 *
 *  Every suite that touches a `.sail` needs this, not just the component packs:
 *  the runtime's integration tests compile real fixtures and import the result.
 */
export function sailSourceLoaderPlugin(): Plugin {
  return {
  name: 'sail-source-loader',
  enforce: 'pre' as const,
  resolveId(id: string, importer?: string) {
    if (!importer) return null
    if (id.endsWith('.sail.controller.ts')) {
      return resolve(dirname(importer), id)
    }
    if (id.endsWith('.compiled.js')) {
      const artifact = resolve(dirname(importer), id)
      const sail = artifact.replace(/\.compiled\.js$/, '.sail')
      if (existsSync(sail)) return sail
    }
    return null
  },
  load(id: string) {
    if (id.endsWith('.sail.controller.ts')) {
      const sail = id.slice(0, -'.controller.ts'.length)
      const sailStem = basename(sail, '.sail')
      // Same component-vs-route classification the main `.sail` branch below
      // uses. Without it every hyphenated component whose `<scope>` declares
      // `props:` alongside an inline `<controller>` block hits the route/page
      // rejection (`component_meta` is None, so a route's "props has no parent
      // to fill it" check fires) — the controller-only compile call never
      // told the compiler this was a component at all.
      const sailIsComponent = sailStem.includes('-') && sailStem !== 'page' && sailStem !== 'index'
      const out = JSON.parse(
        nativeCompileJson(
          readFileSync(sail, 'utf8'),
          JSON.stringify({
            sourcePath: sail,
            componentName: sailIsComponent ? sailStem : undefined,
          }),
        ),
      ) as { kind: string; controller?: string | null; message?: string }
      if (out.kind !== 'ok') throw new Error(`[sail] ${sail}: ${out.message}`)
      if (!out.controller) throw new Error(`[sail] ${id}: no inline <controller> in ${sail}`)
      return out.controller
    }
    if (!id.endsWith('.sail')) return null
    const source = readFileSync(id, 'utf8')
    const stem = basename(id, '.sail')
    const isComponent = stem.includes('-') && stem !== 'page' && stem !== 'index'
    // A source's controller is its `<controller>` block, served as the
    // virtual `<file>.sail.controller.ts`. A sibling on disk is a compile
    // error (`sail-paired-controller`), so there is nothing else to look for.
    const hasInline = /<controller[\s>]/.test(source)
    const setupUrl = hasInline ? `./${basename(id)}.controller.ts` : undefined
    const out = JSON.parse(
      nativeCompileJson(
        source,
        JSON.stringify({
          componentName: isComponent ? stem : undefined,
          sourcePath: id,
          setupUrl,
        }),
      ),
    ) as { kind: string; js?: string; message?: string; line?: number }
    if (out.kind !== 'ok') {
      throw new Error(`[sail] ${id}:${out.line ?? ''} ${out.message}`)
    }
    return out.js
  },
  }
}

export function sailComponentVitestConfig(opts: SailComponentVitestOptions = {}) {
  return defineConfig({
    // The honesty-guard setup file lives in @ship/test-setup, outside each
    // consuming package's root. Vite's dev FS guard (`/@fs/…`) blocks reads
    // above the package dir by default, so allow-list the repo root.
    server: { fs: { allow: [repoRoot] } },
    resolve: {
      // Resolved by package name → point at the framework source, move-proof.
      // `resolveSailAliases` skips any specifier not present in the consuming
      // repo, so the same factory works in a JS-only carve-off.
      alias: {
        ...resolveSailAliases(),
        ...opts.alias,
      },
    },
    test: {
      include: opts.include ?? ['test/**/*.test.ts', 'src/**/*.test.ts'],
      environment: opts.environment ?? 'jsdom',
      // Honesty guard first, then the package's own setup. `hasAssertions()`
      // fails any test whose executed body ran zero assertions.
      setupFiles: [assertHonestySetup, ...(opts.setupFiles ?? ['./test/setup.ts'])],
      ...SHIP_VITEST_DEFAULTS,
    },
    plugins: [sailSourceLoaderPlugin()],
  })
}
