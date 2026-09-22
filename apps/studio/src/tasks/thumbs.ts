/**
 * Client-side evidence thumbs for task render artifacts.
 * Cached at attach time (base64 we already have); keyed by artifact id + path.
 */

const thumbs = new Map<string, string>()

/** Normalize to a data-URL usable as <img src>. */
export function toDataUrl(mime: string, raw: string): string {
  const m = mime || 'image/png'
  if (!raw) return ''
  if (raw.startsWith('data:')) return raw
  // Strip whitespace/newlines some encoders leave.
  const b64 = raw.replace(/\s+/g, '')
  return `data:${m};base64,${b64}`
}

export function rememberArtifactThumb(
  key: string,
  mime: string,
  contentBase64OrDataUrl: string,
): void {
  const k = key?.trim()
  if (!k) return
  const src = toDataUrl(mime || 'image/png', contentBase64OrDataUrl)
  if (!src) return
  thumbs.set(k, src)
}

export function artifactThumbSrc(...keys: (string | null | undefined)[]): string {
  for (const k of keys) {
    if (!k) continue
    const hit = thumbs.get(k)
    if (hit) return hit
  }
  return ''
}

export function __resetThumbsForTests(): void {
  thumbs.clear()
}
