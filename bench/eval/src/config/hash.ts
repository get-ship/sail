/**
 * Content hashing for profiles (immutability mid-batch).
 */

import { createHash } from 'node:crypto'

/** Stable JSON stringify (sorted keys). */
export function stableStringify(value: unknown): string {
  return JSON.stringify(sortKeys(value))
}

function sortKeys(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(sortKeys)
  if (value && typeof value === 'object') {
    const obj = value as Record<string, unknown>
    const out: Record<string, unknown> = {}
    for (const k of Object.keys(obj).sort()) {
      out[k] = sortKeys(obj[k])
    }
    return out
  }
  return value
}

export function sha256Hex(text: string): string {
  return createHash('sha256').update(text, 'utf8').digest('hex')
}

export function contentHash(value: unknown): string {
  return sha256Hex(stableStringify(value))
}

export function fileSha256(buf: Buffer | string): string {
  return createHash('sha256').update(buf).digest('hex')
}
