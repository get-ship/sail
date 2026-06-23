// @ship/sui-media/engine — the PURE detection/parse/resolution engine, with NO
// component registration side effects. The `@ship/sui-media` package barrel
// (src/index.ts) re-exports this AND imports the renderer `.compiled.js` (which
// `customElements.define()` on import); consumers that want only the headless
// engine — e.g. the @ship/viewer router's dispatcher controller, which resolves
// a descriptor without needing the leaf renderers registered in its own module
// — import from here to avoid pulling the component registrations in.

export { createMimeRegistry, type MimeRegistry } from './create-mime-registry'
export { resolveSource } from './resolve'
export {
  normalizeContentType,
  extensionOf,
  headBytes,
  sniffMagicBytes,
  sniffTextBytes,
  sniffSvgText,
  sniffUnifiedDiffText,
} from './detect'
export {
  parseCsv,
  CsvParseError,
  type CsvParseOptions,
  type CsvParseResult,
  type CsvColumn,
  type CsvRow,
  type CsvDelimiter,
} from './csv-parse'
export {
  emitViewerLoad,
  emitViewerError,
  emitViewerProgress,
  type ViewerRendererProps,
  type ViewerRendererEvents,
} from './renderer-contract'
export type {
  ViewerKind,
  ViewerSource,
  SourceDescriptor,
  DetectReason,
  RegistryEntry,
  CreateMimeRegistryOptions,
} from './types'
