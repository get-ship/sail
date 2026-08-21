// Native-backed test helper. Compiles `.sail` source through the SAME
// `sail-compiler` crate the daemon compiles through (deck-core/src/view.rs),
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
// The `@ship/sail-dom` retired import is gone — every directive must
// round-trip through Rust now. This helper is the only test-time bridge
// between `.sail` source and a mounted DOM tree.

import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs'
import { basename, dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

// `/internal` carries language + ABI so rewritten compiled imports resolve.
import * as sailRuntime from '@ship/sail-runtime/internal'
import * as sailActions from '@ship/sail-runtime/actions'
import * as sailMotion from '@ship/sail-runtime/motion'
import * as sailQuery from '@ship/sail-runtime/query'
import * as sailVirtual from '@ship/sail-runtime/virtual'
import * as sailDom from '@ship/sail-dom'
// The runtime's `/route` subpath provides a passive `RouteSignals` reader
// built from a URL — exactly the shape the compiled route's
// `import { route as __routeFactory } from "@ship/sail-runtime/route"`
// expects after the eval layer strips imports. The deck router owns the
// *policy* (history, popstate, route table, `navigate`); the runtime
// owns the *mechanism*. The test harness needs only the mechanism, and
// `@ship/sail-runtime/route` is sail ⊥ deck by construction.
import { route as makeRoute } from '@ship/sail-runtime/route'
import type { RouteSignals } from '@ship/sail-runtime/route'
import '@ship/sail-runtime/template-hooks'
/* ----------------------- native addon load ----------------------- */

const HERE = dirname(fileURLToPath(import.meta.url))
// HERE = packages/sail/dom/test; repo root is four dirs up
// (test → dom → sail → packages → repo root).
const REPO_ROOT = resolve(HERE, '../../../..')
const COMPILER_SRC = resolve(REPO_ROOT, 'crates/sail/compiler/src')
const LINT_SRC = resolve(REPO_ROOT, 'crates/sail/lint/src')
const STYLE_SRC = resolve(REPO_ROOT, 'crates/sail/spinnaker/src')

// Platform cdylib name `cargo build` produces for crates/sail/compiler-napi.
// Node loads a napi cdylib directly via `process.dlopen` (it exports the
// N-API init symbol), so there is no separate `.node` rename step — the
// harness tracks `target/` exactly as `just build` leaves it.
const ADDON_FILE =
  process.platform === 'win32'
    ? 'sail_compiler_napi.dll'
    : process.platform === 'darwin'
      ? 'libsail_compiler_napi.dylib'
      : 'libsail_compiler_napi.so'

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
    throw new Error(`[sail-compiler-napi] addon at ${addonPath} has no compile() export`)
  }
  nativeCompile = mod.exports.compile
  return nativeCompile
}

// Locate the cdylib `cargo build` produces. Honour SAIL_COMPILER_ADDON for an
// explicit override (CI artifact, cross-build); otherwise prefer release over
// debug under the repo's own target dir — same precedence the bench harness
// uses for the bin (scripts/precompile.ts).
function resolveAddonPath(): string {
  const override = process.env.SAIL_COMPILER_ADDON
  if (override) {
    if (!existsSync(override)) {
      throw new Error(`[sail-compiler-napi] SAIL_COMPILER_ADDON=${override} does not exist`)
    }
    return override
  }
  const release = resolve(REPO_ROOT, 'target/release', ADDON_FILE)
  const debug = resolve(REPO_ROOT, 'target/debug', ADDON_FILE)
  if (existsSync(release)) return release
  if (existsSync(debug)) return debug
  throw new Error(
    `[sail-compiler-napi] addon not built. Run from the repo root:\n` +
      `  cargo build -p sail-compiler-napi\n` +
      `(or \`just sail-napi\`), or set SAIL_COMPILER_ADDON to a built .node/.so.`,
  )
}

// Drift guard (generalizes the old warnIfStale). With napi there is no second
// artifact and no embedded registry — staleness is purely "addon older than
// compiler source". If any file under crates/sail/compiler/src is newer than
// the built addon, the harness would test an out-of-date compiler; warn loudly
// to rebuild. Advisory (warn, not throw): a watch loop may race a rebuild, and
// failing the whole suite on an mtime skew is worse than a visible warning.
function assertFresh(addonPath: string): void {
  try {
    const addonMtime = statSync(addonPath).mtimeMs
    const newest = Math.max(
      newestMtime(COMPILER_SRC),
      newestMtime(LINT_SRC),
      newestMtime(STYLE_SRC),
    )
    if (newest > addonMtime) {
      const mins = Math.max(1, Math.round((newest - addonMtime) / 60_000))
      // A stale compiler silently tests yesterday's language — halt, don't warn.
      throw new Error(
        `[sail-compiler-napi] STALE addon: crates/sail/{compiler,lint,style}/src changed ` +
          `~${mins}m after the built addon. Rebuild: cargo build -p sail-compiler-napi`,
      )
    }
  } catch (e) {
    if (e instanceof Error && e.message.includes('STALE addon')) throw e
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
  /** On-disk path of the `.sail` source. Pass it when the test reads a
   *  checked-in `.sail` off disk (e.g. `readFileSync(here('foo.sail'))`) so the
   *  compiler can locate the paired `<stem>.controller.ts` and run the four
   *  controller-aware passes (scope_bound, scope_member, bind_this_target,
   *  on_callable) — the napi path then rejects exactly what the daemon + CLI
   *  do (#263-A). Omit for synthetic in-memory sources. */
  sourcePath?: string
}

export interface CompiledSail {
  js: string
  css: string | null
  dts: string | null
  ssrHtml: string | null
  /** The SFC's inline `<controller>` body, when the source declares one. */
  controller: string | null
}

export function compileSail(source: string, opts: CompileOpts = {}): CompiledSail {
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
    | {
        kind: 'ok'
        js: string
        css: string | null
        dts: string | null
        ssrHtml: string | null
        controller: string | null
      }
    | { kind: 'err'; message: string; line: number | null; col: number | null; category: string | null }
  if (result.kind === 'err') {
    const where = result.line != null ? ` (${result.line}:${result.col ?? 0})` : ''
    throw new Error(`sail-compiler: ${result.message}${where}`)
  }
  return {
    js: result.js,
    css: result.css,
    dts: result.dts,
    ssrHtml: result.ssrHtml,
    controller: result.controller ?? null,
  }
}

// The module shape a compiled route's
// `import { route as __routeFactory } from "@ship/sail-runtime/route"`
// resolves to after `stripImports` rewrites it to `__route.route`. Backed
// by the runtime's `/route` subpath — see the import note at the top.
const routeModule: { route: typeof makeRoute } = { route: makeRoute }
/* ----------------------------- eval API ----------------------------- */

const SETUP_MARKER = '__test_setup__'

type UserSetup = ((ctx: unknown) => Record<string, unknown> | void) | { setup: (ctx: unknown) => unknown } | undefined

interface EvalDeps {
  __runtime: typeof sailRuntime
  __dom: typeof sailDom
  __route: typeof routeModule
  __setup: { setup: (ctx: unknown) => unknown } | null
}

function stripImports(js: string): string {
  return js
    // type-only imports (an SFC controller body is TS, not the emitted JS)
    .replace(/^\s*import\s+type\s+[^;\n]+;?\s*$/gm, '')
    .replace(/^\s*import\s*\{\s*type\s[^}]*\}\s*from\s*["'][^"']+["']\s*;?\s*$/gm, '')
    // import { …names… } from "@ship/sail-runtime/internal" (the compiler's
    // emit specifier; internal re-exports the full runtime surface)
    .replace(
      /^\s*import\s*\{\s*([^}]+)\s*\}\s*from\s*["']@ship\/sail-runtime\/internal["']\s*;?\s*$/gm,
      'const { $1 } = __runtime;',
    )
    // import { …names… } from "@ship/sail-runtime"
    .replace(
      /^\s*import\s*\{\s*([^}]+)\s*\}\s*from\s*["']@ship\/sail-runtime["']\s*;?\s*$/gm,
      'const { $1 } = __runtime;',
    )
    // import { …names… } from "@ship/sail-runtime/actions" (the compiler's
    // split specifier for `use:` action symbols — internal stamps resolve
    // exclusively here, so this needs its own dep, not __runtime)
    .replace(
      /^\s*import\s*\{\s*([^}]+)\s*\}\s*from\s*["']@ship\/sail-runtime\/actions["']\s*;?\s*$/gm,
      'const { $1 } = __actions;',
    )
    .replace(
      /^\s*import\s*\{\s*([^}]+)\s*\}\s*from\s*["']@ship\/sail-runtime\/motion["']\s*;?\s*$/gm,
      'const { $1 } = __motion;',
    )
    .replace(
      /^\s*import\s*\{\s*([^}]+)\s*\}\s*from\s*["']@ship\/sail-runtime\/query["']\s*;?\s*$/gm,
      'const { $1 } = __query;',
    )
    .replace(
      /^\s*import\s*\{\s*([^}]+)\s*\}\s*from\s*["']@ship\/sail-runtime\/virtual["']\s*;?\s*$/gm,
      'const { $1 } = __virtual;',
    )
    // import { …names… } from "@ship/sail-dom"
    .replace(
      /^\s*import\s*\{\s*([^}]+)\s*\}\s*from\s*["']@ship\/sail-dom["']\s*;?\s*$/gm,
      'const { $1 } = __dom;',
    )
    // import { route as __routeFactory } from "@ship/sail-runtime/route"
    // (the specifier the compiler emits — see crates/sail/compiler/src/emit/js/imports.rs).
    .replace(
      /^\s*import\s*\{\s*route\s+as\s+__routeFactory\s*\}\s*from\s*["']@ship\/sail-runtime\/route["']\s*;?\s*$/gm,
      'const __routeFactory = __route.route;',
    )
    // side-effect: import "@ship/sail-runtime"
    .replace(/^\s*import\s+["']@ship\/sail-runtime["']\s*;?\s*$/gm, '')
    // side-effect: import "@ship/sail-runtime/template-hooks"
    .replace(/^\s*import\s+["']@ship\/sail-runtime\/template-hooks["']\s*;?\s*$/gm, '')
    // import setup from "<marker>"
    .replace(/^\s*import\s+setup\s+from\s+["'][^"']+["']\s*;?\s*$/gm, 'const setup = __setup;')
}

/** Raw native compile — `(source, optsJson)` through the linked compiler
 *  addon. The vitest source loader builds on this so tests compile exactly
 *  what deck compiles. */
export function nativeCompileJson(source: string, optsJson: string): string {
  return ensureNative()(source, optsJson)
}

/** Evaluate an SFC's inline `<controller>` body and return its default export
 *  (the controller definition the compiled template imports as `setup`). */
export function evalInlineController(body: string): { setup: (ctx: unknown) => unknown } {
  const src = stripImports(body).replace(/\bexport\s+default\s+/g, 'return ')
  const fn = new Function(
    '__runtime',
    '__dom',
    '__route',
    '__setup',
    '__actions',
    '__motion',
    '__query',
    '__virtual',
    src,
  )
  return fn(
    sailRuntime,
    sailDom,
    routeModule,
    undefined,
    sailActions,
    sailMotion,
    sailQuery,
    sailVirtual,
  ) as { setup: (ctx: unknown) => unknown }
}

/** Evaluate a compiled JS module body and return its default export. */
export function evalCompiledJs(
  js: string,
  setup?: UserSetup,
): { setup: (ctx: unknown) => Record<string, unknown> & { __mount: (c: HTMLElement) => () => void } } {
  const mods: Record<string, unknown> = {}
  const withSail = js
    .replace(
      /^\s*import\s+(\w+)\s+from\s+["']([^"']+\.sail)["']\s*;?\s*$/gm,
      (_m, ident: string, spec: string) => {
        mods[ident] = loadSailMount(spec)
        return `const ${ident} = __mods[${JSON.stringify(ident)}];`
      },
    )
    .replace(
      /^\s*import\s+(\w+)\s+from\s+["']@sail\/element\/([a-z0-9-]+)["']\s*;?\s*$/gm,
      (_m, ident: string, tag: string) => {
        mods[ident] = loadSailMount(`@sail/element/${tag}`)
        return `const ${ident} = __mods[${JSON.stringify(ident)}];`
      },
    )
  const body = stripImports(withSail).replace(/\bexport\s+default\s+/g, 'return ')
  const setupObj = wrapSetup(setup)
  const fn = new Function(
    '__runtime',
    '__dom',
    '__route',
    '__setup',
    '__actions',
    '__motion',
    '__query',
    '__virtual',
    '__mods',
    body,
  )
  const deps: EvalDeps = {
    __runtime: sailRuntime,
    __dom: sailDom,
    __route: routeModule,
    __setup: setupObj,
  }
  return fn(
    deps.__runtime,
    deps.__dom,
    deps.__route,
    deps.__setup,
    sailActions,
    sailMotion,
    sailQuery,
    sailVirtual,
    mods,
  ) as ReturnType<typeof evalCompiledJs>
}

/** Locate the stdlib source for a logical `@sail/element/<tag>` id. */
const stdlibSourceCache = new Map<string, string>()
function stdlibElementSource(tag: string): string {
  const hit = stdlibSourceCache.get(tag)
  if (hit) return hit
  const direct = resolve(REPO_ROOT, 'crates/sail/compiler/stdlib', tag, `${tag}.sail`)
  if (existsSync(direct)) {
    const src = readFileSync(direct, 'utf8')
    stdlibSourceCache.set(tag, src)
    return src
  }
  // Family dirs may differ from the tag (`radio` → `radio-group/`).
  const root = resolve(REPO_ROOT, 'crates/sail/compiler/stdlib')
  const stack = [root]
  while (stack.length > 0) {
    const dir = stack.pop()!
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const p = resolve(dir, entry.name)
      if (entry.isDirectory()) stack.push(p)
      else if (entry.name === `${tag}.sail`) {
        const src = readFileSync(p, 'utf8')
        stdlibSourceCache.set(tag, src)
        return src
      }
    }
  }
  throw new Error(`[sail-compiler-napi] no stdlib source for @sail/element/${tag}`)
}

/** Resolve a default-import through `compile()`, not Deck. Handles both a
 *  filesystem `.sail` path and the bundler's logical `@sail/element/<tag>`. */
function loadSailMount(spec: string): unknown {
  const logical = /^@sail\/element\/([a-z0-9-]+)$/.exec(spec)
  const source = logical ? stdlibElementSource(logical[1]!) : readFileSync(spec, 'utf8')
  const sourcePath = logical
    ? `stdlib/${logical[1]}.sail`
    : spec
  const setupMarker = source.includes('<controller')
    ? `./${basename(sourcePath)}.controller.ts`
    : undefined
  const compiled = compileSail(source, { sourcePath, setupMarker })
  const setup = compiled.controller
    ? evalInlineController(compiled.controller)
    : undefined
  return evalCompiledJs(compiled.js, setup)
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
  /** Flat named providers (same surface as mount inject). */
  providers?: Record<string, unknown>
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

/** Compile + eval + mount in one call. Mirrors the `hydrate(template, setup, opts)` shape. */
export function compileAndMount(source: string, setup: UserSetup, opts: MountOpts): MountHandle {
  const usesSetup = setup != null
  const compileOpts: CompileOpts = usesSetup ? { setupMarker: SETUP_MARKER } : {}
  if (opts.componentName) {
    compileOpts.componentName = opts.componentName
    compileOpts.mode = 'component'
  }
  const compiled = compileSail(source, compileOpts)

  // Component mode: the compiled module is a side-effecting
  // `component(tag, def)` call that registers a custom element. No
  // setup/mount cycle — the test instantiates the element directly
  // via `document.createElement(tag)`.
  if (opts.componentName) {
    evalCompiledJs(compiled.js, setup)
    return { dispose: () => {} }
  }

  const def = evalCompiledJs(compiled.js, setup)
  const providers = opts.providers ?? {}
  const ctx = {
    ...providers,
    ...(Object.keys(providers).length > 0 ? { providers } : {}),
    route: opts.route ?? makeRoute({ url: new URL('http://localhost/') }),
    navigate: opts.navigate ?? (() => {}),
  }
  const scope = def.setup(ctx)
  const before = new Set(Array.from(opts.container.childNodes))
  const cleanup = scope.__mount(opts.container)
  return {
    dispose: () => {
      try { cleanup() } catch {}
      // Mirror hydrate: remove every node mount appended so
      // `container.children` is empty after dispose.
      for (const n of Array.from(opts.container.childNodes)) {
        if (!before.has(n)) n.parentNode?.removeChild(n)
      }
    },
  }
}
