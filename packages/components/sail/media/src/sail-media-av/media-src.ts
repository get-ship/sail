/** Pure source helpers for sail-media-av (url mint, content-type sniff, bytes). */

import type { ViewerSource } from '../engine/types'

export type MediaSrc = ViewerSource | string | null

/** Resolve a media url: pre-resolved `url`, else object URL from `bytes`. */
export function mediaUrlOf(src: MediaSrc): { url: string; created: boolean } {
  if (src == null) return { url: '', created: false }
  if (typeof src === 'string') return { url: src, created: false }
  if (src.url) return { url: src.url, created: false }
  if (src.bytes) {
    const blob = new Blob([src.bytes as BlobPart], {
      ...(src.contentType ? { type: src.contentType } : {}),
    })
    return { url: URL.createObjectURL(blob), created: true }
  }
  return { url: '', created: false }
}

export function contentTypeOf(src: MediaSrc): string {
  if (src == null || typeof src === 'string') return ''
  return src.contentType ?? ''
}

export function bytesOf(src: MediaSrc): ArrayBuffer | Uint8Array | null {
  if (src == null || typeof src === 'string') return null
  return src.bytes ?? null
}
