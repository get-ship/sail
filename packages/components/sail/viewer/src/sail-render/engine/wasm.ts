// Engine-side re-export of the framework-pure WASM bootstrap, plus the
// running-wasm staleness note (build-spec §WASM-in-browser path / Staleness).
//
// The engine NEVER imports a wasm URL or `fetch`. It receives a bound
// `compile()` via `prop:compile={fn}` (identity) and drives it. This module
// re-exports the bootstrap so a host SPA shell wires init from one place, and
// the engine's own types reference one `CompileFn` shape.
//
// Staleness: the Node-only `warnIfStale` fs-mtime check does not exist in a
// browser. The concrete browser check is a VERSION compare — the running
// wasm's embedded `_compilerVersion` (the compiler emits it as a leading
// `/*! _compilerVersion … */` banner on every module, and the CEM /
// `.sail.meta.json` carry the same shape) against the catalog entries'
// `_compilerVersion`. The SPA shell owns surfacing a banner on mismatch; the
// engine exposes the parser so the host reads one canonical version off any
// compiled module rather than re-deriving it.

export { ensureWasm, isWasmReady, makeCompile } from '../../wasm-bootstrap'
export type { CompileFn, WasmCompileOpts, WasmCompileResult } from '../../wasm-bootstrap'

/** The `_compilerVersion` the compiler embeds on every emitted module. */
export interface CompilerVersion {
  cargo: string
  git: string
  dirty: boolean
  builtAt: string
}

const VERSION_BANNER = /\/\*!\s*_compilerVersion\s*(\{[^]*?\})\s*\*\//

/**
 * Parse the leading `_compilerVersion` banner off a compiled module's JS. The
 * host SPA compares this running-compiler version against the catalog's
 * `_compilerVersion` to detect a stale registry. Returns `null` when the
 * banner is absent (a non-compiler-emitted string).
 */
export function readCompilerVersion(js: string): CompilerVersion | null {
  const m = VERSION_BANNER.exec(js)
  if (!m || !m[1]) return null
  try {
    return JSON.parse(m[1]) as CompilerVersion
  } catch {
    return null
  }
}

/** True when two compiler versions differ enough that the registry the catalog
 *  was built against is stale relative to the running wasm. `git` is the
 *  authoritative discriminator; `dirty` upgrades any match to a mismatch. */
export function isStale(running: CompilerVersion | null, catalog: CompilerVersion | null): boolean {
  if (!running || !catalog) return false
  if (running.dirty || catalog.dirty) return running.git !== catalog.git || running.builtAt !== catalog.builtAt
  return running.git !== catalog.git
}
