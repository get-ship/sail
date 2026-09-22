/**
 * Pure-safe base64 helpers (SPEC-L2-PACKAGE-1 / Pure ambient).
 *
 * Controllers must not name `btoa` / `Buffer` under Pure (DOM / Node).
 * Structural `globalThis` access keeps product code ESNext-only.
 */

/**
 * Encode a UTF-8 string to standard base64.
 * Prefers host `btoa` (browsers); falls back to Node `Buffer` when present.
 */
export function utf8ToBase64(text: string): string {
  const g = globalThis as {
    btoa?: (data: string) => string
    TextEncoder?: new () => { encode: (s: string) => Uint8Array }
    Buffer?: {
      from: (data: string, enc: string) => { toString: (enc: string) => string }
    }
  }
  if (typeof g.btoa === 'function') {
    // btoa requires a binary string; encode UTF-8 code units explicitly.
    const Encoder = g.TextEncoder
    if (typeof Encoder === 'function') {
      const bytes = new Encoder().encode(text)
      let bin = ''
      for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]!)
      return g.btoa(bin)
    }
    return g.btoa(text)
  }
  if (g.Buffer && typeof g.Buffer.from === 'function') {
    return g.Buffer.from(text, 'utf8').toString('base64')
  }
  throw new Error('utf8ToBase64: no base64 encoder on this host')
}
