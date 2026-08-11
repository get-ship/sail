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
  type CompileFn,
  type PreviewMount,
  type CodeEditorHost,
} from '@ship/sail-viewer/same-doc-mount'
