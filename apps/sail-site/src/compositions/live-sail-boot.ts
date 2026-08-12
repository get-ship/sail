// Live Sail boot — app-facing re-export of the same-document preview mount.
//
// DOM / script-inject substrate lives in `@ship/sail-viewer/same-doc-mount`
// (framework package; DOM lib allowed). This composition stays Pure: no
// ambient god-globals. Routes keep importing from this path.

export {
  createDebouncer,
  extractController,
  tsToEsm,
  runInHost,
  installConsoleCapture,
  type CompileResult,
  type PreviewMount,
  type CodeEditorHost,
} from '../vendor/viewer-mount'

// CompileFn + wasm seam share one type (wasm-bootstrap) so routes never need
// `as unknown as` when storing makeCompile() in a signal.
export { ensureWasm, makeCompile, type CompileFn } from '../vendor/viewer-wasm'
