// @ship/sail-code-editor — SUI pack, framework-pure.
//
// One element: <sui-code-editor>. The compiled `.compiled.js` calls
// `customElements.define()` on import, so a single `import '@ship/sail-code-editor'`
// registers the tag globally before any view template instantiates it.
//
// Pure presentational: no SDK reads, no ship-platform coupling. The host
// (sui-site or any SUI app) consumes the surface as a primitive.

import './sui-code-editor/sui-code-editor.compiled.js'

// Re-export the imperative API for host-side wiring (the tutorial compile-error
// squiggle flow drives `setDiagnostics` from outside).
export type {
  SuiDiagnostic,
  SuiCompletionSource,
  SuiCodeEditorApi,
} from './sui-code-editor/sui-code-editor.controller'
