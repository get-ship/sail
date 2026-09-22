/** @ship/sail-components/viewers — mermaid + component viewer domain barrel. */
import '../mermaid/src/index.ts'
import '../viewer/src/index.ts'
export {
  ensureWasm,
  isWasmReady,
  makeCompile,
  type CompileFn,
  type WasmCompileOpts,
  type WasmCompileResult,
} from '../viewer/src/wasm-bootstrap.ts'
export * from '../viewer/src/same-doc-mount.ts'
