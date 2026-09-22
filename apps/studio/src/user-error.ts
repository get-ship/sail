/**
 * Map caught exceptions to fixed user copy.
 * Raw machine messages (ops, refused, writeSource, …) never reach the UI.
 */
export function userError(e: unknown, fallback: string): string {
  try {
    if (typeof console !== 'undefined' && typeof console.warn === 'function') {
      console.warn('[studio]', e instanceof Error ? e.message : e)
    }
  } catch {
    /* ignore console failures */
  }
  return fallback
}
