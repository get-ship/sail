// @ship/sui-media — SUI media-renderer pack (the PURE leaf content renderers
// plus the pure detection/parse engine).
//
// Each <tag>.compiled.js calls customElements.define() on import, so a single
// `import '@ship/sui-media'` registers every leaf renderer tag globally:
//   sui-media-image (raster + SVG, zoom/pan, gallery)
//   sui-media-csv   (CSV/TSV → composed sui-table)
//   sui-media-av    (video/audio + optional waveform)
//
// The renderers receive already-resolved bytes/text via `prop:src` identity and
// never fetch (the ViewerRenderer contract). They compose @ship/sui-primitives
// (sui-media-csv windows sui-table), a declared dependency the host registers
// via `import '@ship/sui-primitives'`.
//
// The kind→TAG dispatch table (which renderer a kind routes to) is NOT here —
// that decision lives in the @ship/viewer router. This pack exports only the
// PURE resolution engine (createMimeRegistry, MIME detect, CSV parse) and the
// ViewerRenderer event-contract helpers, which the router composes.
import './sui-media-image/sui-media-image.compiled.js'
import './sui-media-csv/sui-media-csv.compiled.js'
import './sui-media-av/sui-media-av.compiled.js'

// Pure engine surface — consumed by the @ship/viewer router (which seeds the
// registry with its kind→TAG table) and by any host that wants headless
// detection. Re-exported from the dedicated `./engine` barrel (also available
// as the `@ship/sui-media/engine` subpath for consumers that want the engine
// WITHOUT the component-registration side effects above).
export * from './engine'
