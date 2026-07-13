// Native-backed test helper. Compiles `.sui` source through the SAME
// `sui-compiler` crate the daemon compiles through (deck-core/src/view.rs),
// loaded in-process as a Node-API `.node` addon, then evaluates the emitted
// module so vitest tests assert against the production emit shape rather than
// a parallel JS parser.
//
// Why native, not the wasm pkg: one `cargo build` feeds both the daemon and
// this harness, so there is no second artifact to go stale and no separate
// build-identity stamp — the drift class #255 names is gone. The browser
// playground (viewer/wasm-bootstrap.ts + lsp-browser) keeps the wasm surface;
// this path is the Node test harness only.
//
// The legacy `@ship/sui-dom/legacy` path is gone — every directive must
// round-trip through Rust now. This helper is the only test-time bridge
// between `.sui` source and a mounted DOM tree.

import { existsSync, readdirSync, statSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

import * as suiRuntime from '@ship/sui-runtime'
import * as suiDom from '@ship/sui-dom'
// The runtime's `/route` subpath provides a passive `RouteSignals` reader
// built from a URL — exactly the shape the compiled view's
// `import { route as __routeFactory } from "@ship/sui-runtime/route"`
// expects after the eval layer strips imports. The deck router owns the
// *policy* (history, popstate, route table, `navigate`); the runtime
// owns the *mechanism*. The test harness needs only the mechanism, and
// `@ship/sui-runtime/route` is sui ⊥ deck by construction.
import { route as makeRoute } from '@ship/sui-runtime/route'
import type { RouteSignals } from '@ship/sui-runtime'
import '@ship/sui-runtime/template-hooks'
/* ----------------------- native addon load ----------------------- */

const HERE = dirname(fileURLToPath(import.meta.url))
// HERE = packages/sui/dom/src/test-utils; repo root is five dirs up
// (test-utils → src → dom → sui → packages → repo root).
const REPO_ROOT = resolve(HERE, '../../../../..')
const COMPILER_SRC = resolve(REPO_ROOT, 'crates/sui/compiler/src')

// Platform cdylib name `cargo build` produces for crates/sui/compiler-napi.
// Node loads a napi cdylib directly via `process.dlopen` (it exports the
// N-API init symbol), so there is no separate `.node` rename step — the
// harness tracks `target/` exactly as `just build` leaves it.
const ADDON_FILE =
  process.platform === 'win32'
    ? 'sui_compiler_napi.dll'
    : process.platform === 'darwin'
      ? 'libsui_compiler_napi.dylib'
      : 'libsui_compiler_napi.so'

type NativeCompile = (source: string, optsJson: string) => string

let nativeCompile: NativeCompile | null = null

function ensureNative(): NativeCompile {
  if (nativeCompile) return nativeCompile
  const addonPath = resolveAddonPath()
  assertFresh(addonPath)
  const mod = { exports: {} as { compile?: NativeCompile } }
  // dlopen the cargo-built cdylib as a Node-API addon.
  ;(process as unknown as { dlopen: (m: object, p: string) => void }).dlopen(mod, addonPath)
  if (typeof mod.exports.compile !== 'function') {
    throw new Error(`[sui-compiler-napi] addon at ${addonPath} has no compile() export`)
  }
  nativeCompile = mod.exports.compile
  return nativeCompile
}

// Locate the cdylib `cargo build` produces. Honour SUI_COMPILER_ADDON for an
// explicit override (CI artifact, cross-build); otherwise prefer release over
// debug under the repo's own target dir — same precedence the bench harness
// uses for the bin (scripts/precompile.mjs).
function resolveAddonPath(): string {
  const override = process.env.SUI_COMPILER_ADDON
  if (override) {
    if (!existsSync(override)) {
      throw new Error(`[sui-compiler-napi] SUI_COMPILER_ADDON=${override} does not exist`)
    }
    return override
  }
  const release = resolve(REPO_ROOT, 'target/release', ADDON_FILE)
  const debug = resolve(REPO_ROOT, 'target/debug', ADDON_FILE)
  if (existsSync(release)) return release
  if (existsSync(debug)) return debug
  throw new Error(
    `[sui-compiler-napi] addon not built. Run from the repo root:\n` +
      `  cargo build -p sui-compiler-napi\n` +
      `(or \`just sui-napi\`), or set SUI_COMPILER_ADDON to a built .node/.so.`,
  )
}

// Drift guard (generalizes the old warnIfStale). With napi there is no second
// artifact and no embedded registry — staleness is purely "addon older than
// compiler source". If any file under crates/sui/compiler/src is newer than
// the built addon, the harness would test an out-of-date compiler; warn loudly
// to rebuild. Advisory (warn, not throw): a watch loop may race a rebuild, and
// failing the whole suite on an mtime skew is worse than a visible warning.
function assertFresh(addonPath: string): void {
  try {
    const addonMtime = statSync(addonPath).mtimeMs
    const newest = newestMtime(COMPILER_SRC)
    if (newest > addonMtime) {
      const mins = Math.max(1, Math.round((newest - addonMtime) / 60_000))
      console.warn(
        `[sui-compiler-napi] STALE: crates/sui/compiler/src has changes newer ` +
          `than the built addon (~${mins}m). The harness is testing an out-of-date ` +
          `compiler. Run \`cargo build -p sui-compiler-napi\` (or \`just sui-napi\`) to rebuild.`,
      )
    }
  } catch {
    /* source tree not locatable (published layout) — skip the freshness check */
  }
}

// Newest mtime of any file under `dir` (recursive). Bounded by the compiler
// `src/` tree — a few hundred small files, walked once per worker.
function newestMtime(dir: string): number {
  let newest = 0
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const p = resolve(dir, entry.name)
    if (entry.isDirectory()) {
      newest = Math.max(newest, newestMtime(p))
    } else {
      newest = Math.max(newest, statSync(p).mtimeMs)
    }
  }
  return newest
}

/* ----------------------------- compile API ----------------------------- */

export interface CompileOpts {
  mode?: 'view' | 'component' | 'auto'
  componentName?: string
  /** Marker passed to the compiler as the setup_url; eval rewrites the
   *  resulting `import setup from "<marker>"` to the injected setup. */
  setupMarker?: string
  /** On-disk path of the `.sui` source. Pass it when the test reads a
   *  checked-in `.sui` off disk (e.g. `readFileSync(here('foo.sui'))`) so the
   *  compiler can locate the paired `<stem>.controller.ts` and run the four
   *  controller-aware passes (scope_bound, scope_member, bind_this_target,
   *  on_callable) — the napi path then rejects exactly what the daemon + CLI
   *  do (#263-A). Omit for synthetic in-memory sources. */
  sourcePath?: string
}

export interface CompiledSui {
  js: string
  css: string | null
  dts: string | null
  ssrHtml: string | null
}

export function compileSui(source: string, opts: CompileOpts = {}): CompiledSui {
  const compile = ensureNative()
  const optsJson = JSON.stringify({
    mode: opts.mode ?? 'auto',
    componentName: opts.componentName,
    setupUrl: opts.setupMarker,
    sourcePath: opts.sourcePath,
  })
  // The addon returns the WasmResult-shaped discriminated object as a JSON
  // string (one parse here); the {kind:'ok'|'err',...} shape is byte-identical
  // to the wasm surface, so the branch below is unchanged.
  const result = JSON.parse(compile(source, optsJson)) as
    | { kind: 'ok'; js: string; css: string | null; dts: string | null; ssrHtml: string | null }
    | { kind: 'err'; message: string; line: number | null; col: number | null; category: string | null }
  if (result.kind === 'err') {
    const where = result.line != null ? ` (${result.line}:${result.col ?? 0})` : ''
    throw new Error(`sui-compiler: ${result.message}${where}`)
  }
  return { js: result.js, css: result.css, dts: result.dts, ssrHtml: result.ssrHtml }
}

// The module shape a compiled view's
// `import { route as __routeFactory } from "@ship/sui-runtime/route"`
// resolves to after `stripImports` rewrites it to `__route.route`. Backed
// by the runtime's `/route` subpath — see the import note at the top.
const routeModule: { route: typeof makeRoute } = { route: makeRoute }
/* ----------------------------- eval API ----------------------------- */

const SETUP_MARKER = '__test_setup__'

type UserSetup = ((ctx: unknown) => Record<string, unknown> | void) | { setup: (ctx: unknown) => unknown } | undefined

interface EvalDeps {
  __runtime: typeof suiRuntime
  __dom: typeof suiDom
  __route: typeof routeModule
  __setup: { setup: (ctx: unknown) => unknown } | null
}

function stripImports(js: string): string {
  return js
    // import { …names… } from "@ship/sui-runtime/internal" (the compiler's
    // emit specifier; internal re-exports the full runtime surface)
    .replace(
      /^\s*import\s*\{\s*([^}]+)\s*\}\s*from\s*["']@ship\/sui-runtime\/internal["']\s*;?\s*$/gm,
      'const { $1 } = __runtime;',
    )
    // import { …names… } from "@ship/sui-runtime"
    .replace(
      /^\s*import\s*\{\s*([^}]+)\s*\}\s*from\s*["']@ship\/sui-runtime["']\s*;?\s*$/gm,
      'const { $1 } = __runtime;',
    )
    // import { …names… } from "@ship/sui-dom"
    .replace(
      /^\s*import\s*\{\s*([^}]+)\s*\}\s*from\s*["']@ship\/sui-dom["']\s*;?\s*$/gm,
      'const { $1 } = __dom;',
    )
    // import { route as __routeFactory } from "@ship/sui-runtime/route"
    // (the specifier the compiler emits — see crates/sui/compiler/src/emit/js/imports.rs).
    .replace(
      /^\s*import\s*\{\s*route\s+as\s+__routeFactory\s*\}\s*from\s*["']@ship\/sui-runtime\/route["']\s*;?\s*$/gm,
      'const __routeFactory = __route.route;',
    )
    // side-effect: import "@ship/sui-runtime"
    .replace(/^\s*import\s+["']@ship\/sui-runtime["']\s*;?\s*$/gm, '')
    // side-effect: import "@ship/sui-runtime/template-hooks"
    .replace(/^\s*import\s+["']@ship\/sui-runtime\/template-hooks["']\s*;?\s*$/gm, '')
    // import setup from "<marker>"
    .replace(/^\s*import\s+setup\s+from\s+["'][^"']+["']\s*;?\s*$/gm, 'const setup = __setup;')
}

/** Raw native compile — `(source, optsJson)` through the linked compiler
 *  addon. The vitest source loader builds on this so tests compile exactly
 *  what deck compiles. */
export function nativeCompileJson(source: string, optsJson: string): string {
  return ensureNative()(source, optsJson)
}

/** Evaluate a compiled JS module body and return its default export. */
export function evalCompiledJs(
  js: string,
  setup?: UserSetup,
): { setup: (ctx: unknown) => Record<string, unknown> & { __mount: (c: HTMLElement) => () => void } } {
  const body = stripImports(js).replace(/\bexport\s+default\s+/g, 'return ')
  const setupObj = wrapSetup(setup)
  const fn = new Function('__runtime', '__dom', '__route', '__setup', body)
  const deps: EvalDeps = {
    __runtime: suiRuntime,
    __dom: suiDom,
    __route: routeModule,
    __setup: setupObj,
  }
  return fn(deps.__runtime, deps.__dom, deps.__route, deps.__setup) as ReturnType<typeof evalCompiledJs>
}

function wrapSetup(setup: UserSetup): { setup: (ctx: unknown) => unknown } | null {
  if (!setup) return null
  if (typeof setup === 'function') {
    return { setup: (ctx) => setup(ctx) ?? {} }
  }
  return setup
}

/* ----------------------------- mount API ----------------------------- */

export interface MountOpts {
  container: HTMLElement
  ship?: unknown
  route?: RouteSignals
  navigate?: (path: string) => void
  /** When the source declares `<prop>`s, compile as a custom element
   *  with this tag name. The compiled module's top-level
   *  `customElements.define(<name>, …)` registers the element; the
   *  test then instantiates it via `document.createElement(<name>)`. */
  componentName?: string
}

export interface MountHandle {
  dispose: () => void
}

/** Compile + eval + mount in one call. Mirrors the legacy `hydrate(template, setup, opts)` shape. */
export function compileAndMount(source: string, setup: UserSetup, opts: MountOpts): MountHandle {
  const usesSetup = setup != null
  const compileOpts: CompileOpts = usesSetup ? { setupMarker: SETUP_MARKER } : {}
  if (opts.componentName) {
    compileOpts.componentName = opts.componentName
    compileOpts.mode = 'component'
  }
  const compiled = compileSui(source, compileOpts)

  // Component mode: the compiled module is a side-effecting
  // `component(tag, def)` call that registers a custom element. No
  // setup/mount cycle — the test instantiates the element directly
  // via `document.createElement(tag)`.
  if (opts.componentName) {
    evalCompiledJs(compiled.js, setup)
    return { dispose: () => {} }
  }

  const def = evalCompiledJs(compiled.js, setup)
  const ctx = {
    ship: (opts.ship ?? {}) as suiRuntime.ShipHost,
    route: opts.route ?? makeRoute({ url: new URL('http://localhost/') }),
    navigate: opts.navigate ?? (() => {}),
  }
  const scope = def.setup(ctx)
  const before = new Set(Array.from(opts.container.childNodes))
  const cleanup = scope.__mount(opts.container)
  return {
    dispose: () => {
      try { cleanup() } catch {}
      // Mirror legacy hydrate: remove every node mount appended so
      // `container.children` is empty after dispose.
      for (const n of Array.from(opts.container.childNodes)) {
        if (!before.has(n)) n.parentNode?.removeChild(n)
      }
    },
  }
}
