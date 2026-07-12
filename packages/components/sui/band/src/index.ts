// @ship/sui-band — SUI pack, framework-pure.
//
// One element: <sui-band>. The compiled `.compiled.js` calls
// `customElements.define()` on import, so a single `import '@ship/sui-band'`
// registers the tag globally before any view template instantiates it.
//
// The band is a marquee: it scrolls its light-DOM content in a seamless
// loop. The component owns the seam math — it clones the content until the
// -50% translate loop can never show a gap or a bare stretch at any
// viewport, so the author writes the content ONCE (no manual duplication).

import './sui-band/sui-band.compiled.js'
