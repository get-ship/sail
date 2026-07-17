// Framework-pure WASM bootstrap for the browser compile path.
//
// `@ship/sail-dom/test-utils`'s `ensureWasm()` is Node-only — it
// `readFileSync`s `../../../compiler-wasm/pkg/sui_compiler_bg.wasm` and
// `initSync`s the bytes. That path does not exist in a browser. This module is
// the browser analogue: it instantiates the same `@ship/sail-compiler-wasm`
// module from a URL the HOST supplies, runs exactly ONCE (module-scope
// `wasmReady` guard mirroring `ensureWasm`), and exposes the bound `compile()`.
//
// PURITY SEAM (the whole reason this is a separate module):
//   - `wasmUrl` is a PARAMETER. This module never hardcodes a path and never
//     names a daemon / `/api` / `/_deck` URL. The host SPA shell resolves the
//     URL (deck `?url` asset or `FRAMEWORK_ASSETS` — see the pack build-spec,
//     §WASM-in-browser path) and hands it in.
//   - The single `fetch` below is the ONLY `fetch` permitted in the entire
//     pack, and it fetches exactly the injected `wasmUrl`, nothing else.
//   - The bound `compile` is handed DOWN to the primitives via
//     `prop:compile={fn}` (identity) so `sui-render` / `sui-component-viewer`
//     never import the wasm or know a URL — Cardinal 4 keeps primitives
//     backend-free.

import init, { initSync, compile as wasmCompile } from '@ship/sail-compiler-wasm'

/** The wasm `compile(source, optsJson)` return — `kind:'ok'` carries the
 *  emitted module surfaces; `kind:'err'` carries a structured diagnostic the
 *  engine renders in its compile-error branch (no exception across the seam). */
export type WasmCompileResult =
  | { kind: 'ok'; js: string; css: string | null; dts: string | null; ssrHtml: string | null }
  | { kind: 'err'; message: string; line: number | null; col: number | null; category: string | null }

/** Compile opts mirror `@ship/sail-dom/test-utils`'s `CompileOpts`. */
export interface WasmCompileOpts {
  mode?: 'view' | 'component' | 'auto'
  componentName?: string
  /** Marker passed as the setup_url; a host that injects a setup rewrites the
   *  resulting `import setup from "<marker>"` to its own setup module. */
  setupUrl?: string
  /** The Node test harness (`@ship/sail-dom/test-utils` `compileSui`) reads the
   *  marker under `setupMarker`; the production wasm reads `setupUrl`. The
   *  engine passes both so a single opts object works across either compile. */
  setupMarker?: string
  /** Component-mode shadow flag. The wasm defaults it to `false` (light DOM) —
   *  so a tag-mode mount of a shadow primitive must pass `true`, else the
   *  versioned element compiles without a shadow root: no `<slot>`, no scoped
   *  `<style>`, just bare light content. */
  shadow?: boolean
}

/** The bound compile function the engine drives. Pure: source in, result out —
 *  no `fetch`, no path, no daemon. This is what flows via `prop:compile={fn}`. */
export type CompileFn = (source: string, opts?: WasmCompileOpts) => WasmCompileResult

// Module-scope once-guard. A second `ensureWasm` call after the first resolves
// is a no-op (mirrors the Node harness `wasmReady` flag). Concurrent first
// callers share the single in-flight init promise so two early compiles never
// double-instantiate.
let wasmReady = false
let initInFlight: Promise<void> | null = null

/**
 * Instantiate the compiler wasm from `wasmUrl`, exactly once.
 *
 * Strategy (A preferred): `await init(wasmUrl)` — the package's
 * `__wbg_init as default` fetches + `WebAssembly.instantiateStreaming`
 * internally (the single permitted network access; the URL is the injected
 * param). Strategy (B fallback): an environment whose `instantiateStreaming`
 * rejects a non-`application/wasm` content type falls back to an explicit
 * `fetch(wasmUrl) → arrayBuffer() → initSync({ module: bytes })` — the same
 * call shape the Node harness uses, just over the network.
 *
 * @param wasmUrl resolved by the host (deck `?url` asset / framework asset).
 *                NEVER a hardcoded path; NEVER a daemon/`/api` URL.
 */
export async function ensureWasm(wasmUrl: string): Promise<void> {
  if (wasmReady) return
  if (initInFlight) return initInFlight

  initInFlight = (async () => {
    try {
      // (A) streaming — preferred. `init` resolves the URL itself.
      await init(wasmUrl)
    } catch {
      // (B) explicit fetch + sync init — the harness call shape, over the wire.
      // This is the ONLY `fetch` in the pack; the URL is the injected param.
      const bytes = await fetch(wasmUrl).then((r) => r.arrayBuffer())
      initSync({ module: bytes })
    }
    wasmReady = true
  })()

  try {
    await initInFlight
  } finally {
    initInFlight = null
  }
}

/** True once the wasm has been instantiated. */
export function isWasmReady(): boolean {
  return wasmReady
}

/**
 * Build the bound `compile()` the engine passes down via `prop:compile`.
 * `ensureWasm(wasmUrl)` MUST have completed first (the engine awaits it before
 * the first `compileSui` — Slice 1). Calling `compile` before init throws from
 * the wasm glue, which is the correct loud failure (not a silent path).
 */
export function makeCompile(): CompileFn {
  return (source, opts = {}) => {
    const optsJson = JSON.stringify({
      mode: opts.mode ?? 'auto',
      componentName: opts.componentName,
      setupUrl: opts.setupUrl,
      shadow: opts.shadow ?? false,
    })
    return wasmCompile(source, optsJson) as WasmCompileResult
  }
}

// Test-only reset so a boundary/unit test can re-exercise the once-guard
// without a fresh module graph. NOT used by any primitive at runtime.
export function __resetWasmForTest(): void {
  wasmReady = false
  initInFlight = null
}
