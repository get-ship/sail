// @ship/sail-mermaid — SUI pack, framework-pure.
//
// One element: <sui-mermaid>. The compiled `.compiled.js` calls
// `customElements.define()` on import, so a single `import '@ship/sail-mermaid'`
// registers the tag globally before any view template instantiates it.
//
// Mermaid is heavy; this pack is its own deck bundle (lazy: true) and the
// controller dynamically imports 'mermaid' on first render, so the dep is
// only fetched when a diagram actually needs to draw.

import './sui-mermaid/sui-mermaid.compiled.js'
