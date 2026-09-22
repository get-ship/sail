// sail-media engine — shared types for the MIME registry + dispatcher.
//
// PURE TypeScript. No `@ship/sail-runtime`, no I/O, no DOM. The reactive slice
// (`descriptor.ts`) imports `signal`/`derived`; these type shapes are the
// contract every other engine file resolves against.
//
// Authority: build-spec §"API sketch (the load-bearing surface)".

/** The content KINDS this pack renders — the `{#match}` arms in the dispatcher.
 *  `unsupported` is the fallback when nothing matches. */
export type ViewerKind =
  | 'pdf'
  | 'image'
  | 'media'
  | 'tabular'
  | 'markdown'
  | 'code'
  | 'json'
  | 'diff'
  | 'unsupported'

/** What the CONSUMER hands the dispatcher. Bytes are ALREADY RESOLVED — the
 *  viewer never fetches. Detection prefers explicit `contentType`, then the
 *  `filename` extension, then a magic-byte sniff of `bytes`. */
export interface ViewerSource {
  /** e.g. 'application/pdf', 'text/csv;charset=utf-8' — params stripped. */
  contentType?: string
  /** e.g. 'report.pdf' — extension fallback. */
  filename?: string
  /** Resolved text (markdown/code/json/diff/csv). */
  text?: string
  /** Resolved binary (pdf/image/media) — sniffed, NOT fetched. */
  bytes?: ArrayBuffer | Uint8Array
  /** Object URL / data URL the consumer created (img/video src). */
  url?: string
  /** sail-diff's existing from/to contract surfaces here. */
  from?: string
  to?: string
}

/** Which signal won detection — recorded so a consumer can debug a mis-route. */
export type DetectReason = 'content-type' | 'extension' | 'magic-bytes' | 'fallback' | 'none'

/** The resolved decision the dispatcher renders. */
export interface SourceDescriptor {
  kind: ViewerKind
  /** Tag name, e.g. 'sail-media-image' | 'sail-table' | 'sail-json-view'; null when
   *  unsupported. */
  renderer: string | null
  /** Canonical mime (charset/params stripped). */
  normalizedType: string
  reason: DetectReason
}

/** A registry ENTRY — adding a content type is ONE entry, never a dispatcher
 *  edit. Predicates are pure and ordered; the first match at the highest-
 *  confidence available signal wins. */
export interface RegistryEntry {
  /** Stable id (the kind, usually). */
  id: string
  kind: ViewerKind
  /** The sail-* tag to render. */
  renderer: string | null
  matchesType?: (normalizedType: string) => boolean
  matchesExt?: (ext: string) => boolean
  /** Magic-byte sniff (e.g. %PDF, PNG/JPEG sig). */
  matchesBytes?: (head: Uint8Array) => boolean
}

export interface CreateMimeRegistryOptions {
  /** Override/extend the default table (prepended — wins over defaults). */
  entries?: RegistryEntry[]
  /** Default null → 'unsupported' empty state. */
  fallbackRenderer?: string | null
}
