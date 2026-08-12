// @ship/sail-components/mermaid — SAIL package, framework-pure.
//
// One element: <sail-mermaid>. The compiled `.compiled.js` calls
// `customElements.define()` on import, so a single `import '@ship/sail-components/mermaid'`
// registers the tag globally before any view template instantiates it.
//
// Mermaid is heavy; this package is its own deck bundle (lazy: true) and the
// controller dynamically imports 'mermaid' on first render, so the dep is
// only fetched when a diagram actually needs to draw.

import './sail-mermaid/sail-mermaid.compiled.js'
