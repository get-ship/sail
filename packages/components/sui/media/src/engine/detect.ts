// sui-media engine — content detection (extension + magic-byte + content-type
// normalization). PURE: string/byte inspection only, no I/O, no DOM, no
// `@ship/sail-runtime`. The consumer already resolved the bytes — the sniffer
// inspects the supplied head, it never fetches.
//
// Authority: build-spec §"Resolution order is FIXED and validated",
// §"The default registry table (the eight kinds)".

import type { ViewerKind } from './types'

/** Normalize a content-type: strip the `;param=…` tail, trim, lowercase.
 *  `'text/csv;charset=utf-8'` → `'text/csv'`; blank → `''`. */
export function normalizeContentType(raw: string | undefined): string {
  if (!raw) return ''
  const semi = raw.indexOf(';')
  const base = semi >= 0 ? raw.slice(0, semi) : raw
  return base.trim().toLowerCase()
}

/** Lowercase extension WITHOUT the dot, e.g. `'report.PDF'` → `'pdf'`.
 *  Empty string when the name has no extension. */
export function extensionOf(filename: string | undefined): string {
  if (!filename) return ''
  const dot = filename.lastIndexOf('.')
  if (dot < 0 || dot === filename.length - 1) return ''
  return filename.slice(dot + 1).trim().toLowerCase()
}

/** Coerce a resolved binary source into the byte head the sniffers inspect. */
export function headBytes(bytes: ArrayBuffer | Uint8Array | undefined): Uint8Array | null {
  if (bytes == null) return null
  return bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes)
}

function startsWith(head: Uint8Array, sig: number[]): boolean {
  if (head.length < sig.length) return false
  for (let i = 0; i < sig.length; i++) {
    if (head[i] !== sig[i]) return false
  }
  return true
}

/** Does `head` contain `sig` within its first `window` bytes? (e.g. `ftyp`
 *  sits at offset 4 in MP4; `WEBP`/`WAVE` after the RIFF length.) */
function containsWithin(head: Uint8Array, sig: number[], window: number): boolean {
  const limit = Math.min(head.length - sig.length, window)
  for (let off = 0; off <= limit; off++) {
    let hit = true
    for (let i = 0; i < sig.length; i++) {
      if (head[off + i] !== sig[i]) {
        hit = false
        break
      }
    }
    if (hit) return true
  }
  return false
}

// ── Magic-byte signatures (ASCII codes spelled out for readability) ──────────
const PDF = [0x25, 0x50, 0x44, 0x46, 0x2d] // %PDF-
const PNG = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]
const JPEG = [0xff, 0xd8, 0xff]
const GIF87 = [0x47, 0x49, 0x46, 0x38, 0x37, 0x61] // GIF87a
const GIF89 = [0x47, 0x49, 0x46, 0x38, 0x39, 0x61] // GIF89a
const RIFF = [0x52, 0x49, 0x46, 0x46] // RIFF
const WEBP = [0x57, 0x45, 0x42, 0x50] // WEBP
const WAVE = [0x57, 0x41, 0x56, 0x45] // WAVE
const OGG = [0x4f, 0x67, 0x67, 0x53] // OggS
const ID3 = [0x49, 0x44, 0x33] // ID3
const FTYP = [0x66, 0x74, 0x79, 0x70] // ftyp

/** Sniff a resolved byte head and return the kind it signals, or null. PURE —
 *  inspects the supplied bytes only. RIFF disambiguates WEBP (image) vs WAVE
 *  (media) by the FourCC after the length. */
export function sniffMagicBytes(head: Uint8Array): ViewerKind | null {
  if (head.length === 0) return null
  if (startsWith(head, PDF)) return 'pdf'
  if (startsWith(head, PNG)) return 'image'
  if (startsWith(head, JPEG)) return 'image'
  if (startsWith(head, GIF87) || startsWith(head, GIF89)) return 'image'
  if (startsWith(head, RIFF)) {
    if (containsWithin(head, WEBP, 12)) return 'image'
    if (containsWithin(head, WAVE, 12)) return 'media'
    return null
  }
  if (startsWith(head, OGG)) return 'media'
  if (startsWith(head, ID3)) return 'media'
  // ISO-BMFF (mp4/m4a): `ftyp` box at offset 4.
  if (containsWithin(head, FTYP, 8)) return 'media'
  return null
}

/** Decode the head as UTF-8 (lossy) for text sniffs (SVG, unified diff). */
function decodeText(head: Uint8Array): string {
  try {
    return new TextDecoder('utf-8', { fatal: false }).decode(head)
  } catch {
    return ''
  }
}

/** SVG is text — sniff the opening `<svg` (after an optional XML prolog). */
export function sniffSvgText(text: string): boolean {
  const head = text.replace(/^﻿/, '').trimStart().slice(0, 256).toLowerCase()
  if (head.startsWith('<svg')) return true
  return head.startsWith('<?xml') && head.includes('<svg')
}

/** Unified-diff text sniff (`--- `/`+++ `/`@@`). */
export function sniffUnifiedDiffText(text: string): boolean {
  const head = text.slice(0, 512)
  return /^(@@ |--- |\+\+\+ |diff --git )/m.test(head)
}

/** Convenience: sniff text content held in a byte head (SVG/diff). */
export function sniffTextBytes(head: Uint8Array): ViewerKind | null {
  const text = decodeText(head)
  if (sniffSvgText(text)) return 'image'
  if (sniffUnifiedDiffText(text)) return 'diff'
  return null
}
