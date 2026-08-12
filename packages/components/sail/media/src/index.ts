// @ship/sail-media — SAIL media-renderer package (the PURE leaf content renderers
// plus the pure detection/parse engine).
//
// Each <tag>.compiled.js calls customElements.define() on import, so a single
// `import '@ship/sail-media'` registers every leaf renderer tag globally:
//   sail-media-image (raster + SVG, zoom/pan, gallery)
//   sail-media-csv   (CSV/TSV → composed sail-table)
//   sail-media-av    (video/audio + optional waveform)
//
// The renderers receive already-resolved bytes/text via `prop:src` identity and
// never fetch (the ViewerRenderer contract). They compose @ship/sail-primitives
// (sail-media-csv windows sail-table), a declared dependency the host registers
// via `import '@ship/sail-primitives'`.
//
// The kind→TAG dispatch table (which renderer a kind routes to) is NOT here —
// that decision lives in the @ship/viewer router. This package exports only the
// PURE resolution engine (createMimeRegistry, MIME detect, CSV parse) and the
// ViewerRenderer event-contract helpers, which the router composes.
import './sail-media-image/sail-media-image.compiled.js'
import './sail-media-csv/sail-media-csv.compiled.js'
import './sail-media-av/sail-media-av.compiled.js'

// Pure engine surface — consumed by the @ship/viewer router (which seeds the
// registry with its kind→TAG table) and by any host that wants headless
// detection. Re-exported from the dedicated `./engine` barrel (also available
// as the `@ship/sail-media/engine` subpath for consumers that want the engine
// WITHOUT the component-registration side effects above).
export * from './engine'
