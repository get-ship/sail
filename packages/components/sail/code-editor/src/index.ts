// @ship/sail-components/code — SAIL package, framework-pure.
//
// One element: <sail-code-editor>. The compiled `.compiled.js` calls
// `customElements.define()` on import, so a single `import '@ship/sail-components/code'`
// registers the tag globally before any view template instantiates it.
//
// Pure presentational: no SDK reads, no ship-platform coupling. The host
// (sail-site or any SAIL app) consumes the surface as a primitive.

import './sail-code-editor/sail-code-editor.compiled.js'

// Re-export the imperative API for host-side wiring (the tutorial compile-error
// squiggle flow drives `setDiagnostics` from outside).
export type {
  SailDiagnostic,
  SailCompletionSource,
  SailCodeEditorApi,
} from './sail-code-editor/sail-code-editor.controller'

// Thin ops client (testable without mounting CodeMirror).
export {
  createOpsAuthoringClient,
  normalizeOpsBase,
  offsetToLineCol,
  parseOpsDiagnostics,
} from './sail-code-editor/sail-code-editor.ops'
export type {
  OpsAuthoringClient,
  OpsCompletionItem,
  OpsDiagnostic,
  OpsHoverResult,
} from './sail-code-editor/sail-code-editor.ops'
