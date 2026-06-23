// Browser-safe diagnostics — backed by the Rust grammar via WASM.
//
// Calls `@ship/sui-compiler-wasm`'s `analyze` (the same `sui_compiler::analyze`
// the native LSP + agent tooling use), so the docs playground reports the SAME
// diagnostics as `cargo build` / `sui-lsp` — one grammar, no TypeScript parser
// fork. The compiler resolves positions (1-based line/col) already.
//
// `analyze` runs without a paired controller (the browser has no `.ts` on
// disk), so this covers template parse + semantic + lint diagnostics.

import initWasm, { analyze as wasmAnalyze } from '@ship/sui-compiler-wasm'

/** Shape consumed by `<ship-code-editor>.setDiagnostics()`. */
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
// Called with no argument so the wasm-bindgen shim self-resolves the wasm via
// `new URL('sui_compiler_bg.wasm', import.meta.url)` — works under both Vite
// (docs playground) and Bun.build (the shell's ship-code-editor bundle); a
// Vite-only `?url` import broke the latter.
let wasmReady: Promise<unknown> | null = null
function ensureWasm(): Promise<unknown> {
  if (!wasmReady) wasmReady = initWasm()
  return wasmReady
}

/** Analyse a `.sui` document and map every diagnostic to the editor shape. */
export async function diagnose(source: string): Promise<SuiDiagnostic[]> {
  await ensureWasm()
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
