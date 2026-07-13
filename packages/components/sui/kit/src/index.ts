// @ship/sui-kit — the general L2 collection. Each `<tag>.compiled.js` calls
// `customElements.define()` on import; a single `import '@ship/sui-kit'`
// registers every tag, and the per-tag resolver tree-shakes unused ones.

// <sui-icon> — named-glyph icon (lucide/app sets) over the L1 <icon> sizing
// wrapper. MIGRATION HOLD: recovered verbatim from the pre-sweep baseline;
// carries a <style> block until its port is ruled through the gap ledger.
import './sui-icon/sui-icon.compiled.js'

export { primeIconCache } from './sui-icon/registry'
