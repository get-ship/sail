// sui-media engine — the PURE resolution function. Given an ordered entry list
// (the kind→renderer table the consumer owns) and a resolved source, decide the
// descriptor. PURE: detection only — no I/O, no DOM, no `@ship/sail-runtime`, no
// knowledge of WHICH tags exist (the entry list carries that). The dispatch
// decision (the default kind→TAG table) lives in the consumer that owns the
// routing; this module only inspects signals and walks the supplied entries.
//
// Authority: build-spec §"Resolution order is FIXED and validated".

import type { RegistryEntry, SourceDescriptor, ViewerSource } from './types'
import { extensionOf, headBytes, normalizeContentType, sniffMagicBytes, sniffTextBytes } from './detect'

const UNSUPPORTED = (fallbackRenderer: string | null, normalizedType: string): SourceDescriptor => ({
  kind: 'unsupported',
  renderer: fallbackRenderer,
  normalizedType,
  reason: fallbackRenderer ? 'fallback' : 'none',
})

/** Resolve a source against an ordered entry list. PURE one-shot — the
 *  precedence is FIXED: content-type, then filename extension, then a
 *  magic-byte sniff of the resolved bytes. First entry that matches at the
 *  highest-confidence available signal wins. */
export function resolveSource(
  entries: readonly RegistryEntry[],
  source: ViewerSource | null,
  fallbackRenderer: string | null,
): SourceDescriptor {
  if (source == null) return UNSUPPORTED(fallbackRenderer, '')

  const normalizedType = normalizeContentType(source.contentType)

  // Tier 1 — content-type (highest confidence).
  if (normalizedType) {
    for (const e of entries) {
      if (e.matchesType?.(normalizedType)) {
        return { kind: e.kind, renderer: e.renderer, normalizedType, reason: 'content-type' }
      }
    }
  }

  // Tier 2 — filename extension.
  const ext = extensionOf(source.filename)
  if (ext) {
    for (const e of entries) {
      if (e.matchesExt?.(ext)) {
        return { kind: e.kind, renderer: e.renderer, normalizedType, reason: 'extension' }
      }
    }
  }

  // Tier 3 — magic-byte sniff of resolved bytes.
  const head = headBytes(source.bytes)
  if (head) {
    for (const e of entries) {
      if (e.matchesBytes?.(head)) {
        return { kind: e.kind, renderer: e.renderer, normalizedType, reason: 'magic-bytes' }
      }
    }
  }

  // Tier 4 — text sniff for SVG/diff carried as `text` (no bytes/type/ext).
  if (source.text) {
    const textHead = new TextEncoder().encode(source.text.slice(0, 512))
    for (const e of entries) {
      if (e.matchesBytes?.(textHead)) {
        return { kind: e.kind, renderer: e.renderer, normalizedType, reason: 'magic-bytes' }
      }
    }
  }

  return UNSUPPORTED(fallbackRenderer, normalizedType)
}
