// Browser-safe diagnostics — backed by the Rust grammar via WASM.
//
// Calls `@ship/sui-compiler-wasm`'s `analyze` (the same `sail_compiler::analyze`
// the native LSP + agent tooling use), so the docs playground reports the SAME
// diagnostics as `cargo build` / `sui-lsp` — one grammar, no TypeScript parser
// fork. The compiler resolves positions (1-based line/col) already.
//
// `analyze` runs without a paired controller (the browser has no `.ts` on
// disk), so this covers template parse + semantic + lint diagnostics.

import initWasm, { analyze as wasmAnalyze } from '@ship/sui-compiler-wasm'

/** Shape consumed by `<sui-code-editor>.setDiagnostics()`. */
export interface SuiDiagnostic {
  line: number
  column?: number
  endLine?: number
  endColumn?: number
  severity: 'error' | 'warning' | 'info'
  message: string
  source?: string
}

/** One diagnostic as the WASM `analyze` serializes it (1-based positions). */
interface WasmDiagnostic {
  line: number
  col: number
  end_line: number
  end_col: number
  severity: 'error' | 'warning'
  message: string
  rule?: string
}
interface WasmAnalysis {
  diagnostics: WasmDiagnostic[]
}

// Lazy one-time WASM init — the worker is long-lived; init once, reuse.
//
// `wasmUrl` is REQUIRED. The deck bundler can't auto-resolve the `.wasm`
// URL the way Vite (the docs playground) does — the wasm-bindgen
// default is a build-time `new URL('./sui_compiler_bg.wasm', import.meta.url)`
// that gets erased when the module is inlined into the entry bundle.
// Callers MUST pass the canonical /sui_compiler_bg.wasm URL the deck
// router serves with `application/wasm` MIME. The default falls back to
// the same path for plain convenience; ship the URL through if you need
// a different mount.
let wasmReady: Promise<unknown> | null = null
function ensureWasm(wasmUrl: string): Promise<unknown> {
  if (!wasmReady) wasmReady = initWasm(wasmUrl)
  return wasmReady
}

/** Analyse a `.sui` document and map every diagnostic to the editor shape. */
export async function diagnose(
  source: string,
  wasmUrl: string = '/sui_compiler_bg.wasm',
): Promise<SuiDiagnostic[]> {
  await ensureWasm(wasmUrl)
  const analysis = wasmAnalyze(source) as WasmAnalysis
  return (analysis.diagnostics ?? []).map((d) => ({
    line: d.line,
    column: d.col,
    endLine: d.end_line,
    endColumn: d.end_col,
    severity: d.severity === 'warning' ? 'warning' : 'error',
    message: d.message,
    source: 'sui',
  }))
}
