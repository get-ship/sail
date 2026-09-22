/**
 * Pure-safe byte → object-URL helpers (SPEC-L2-PACKAGE-1 P1).
 *
 * Controllers must not name `Blob` / `URL.createObjectURL` under Pure.
 * These APIs wrap the platform under the hood via structural `globalThis`
 * access so product code stays ESNext-only.
 */

type Bytes = ArrayBuffer | ArrayBufferView

function toBufferSource(data: Bytes): BlobPart {
  if (ArrayBuffer.isView(data)) {
    // Copy into a fresh ArrayBuffer so BlobPart typing is clean across TS targets.
    const view = data as ArrayBufferView
    const copy = new ArrayBuffer(view.byteLength)
    new Uint8Array(copy).set(new Uint8Array(view.buffer, view.byteOffset, view.byteLength))
    return copy
  }
  return data
}

/**
 * Mint a blob: object URL from bytes. Caller must `revokeObjectUrl` when done.
 * Optional MIME `type` is forwarded to the underlying Blob.
 */
export function objectUrlFromBytes(data: Bytes, type?: string): string {
  const g = globalThis as {
    Blob?: new (parts: BlobPart[], opts?: { type?: string }) => Blob
    URL?: { createObjectURL?: (b: Blob) => string }
  }
  if (typeof g.Blob !== 'function' || typeof g.URL?.createObjectURL !== 'function') {
    throw new Error('objectUrlFromBytes: Blob/URL.createObjectURL unavailable on this host')
  }
  const parts: BlobPart[] = [toBufferSource(data)]
  const blob = type ? new g.Blob(parts, { type }) : new g.Blob(parts)
  return g.URL.createObjectURL(blob)
}

/** Revoke a URL previously returned by {@link objectUrlFromBytes}. */
export function revokeObjectUrl(url: string): void {
  const g = globalThis as { URL?: { revokeObjectURL?: (u: string) => void } }
  g.URL?.revokeObjectURL?.(url)
}
