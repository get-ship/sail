// Prop attribute decoders. The compiler emits a decoder per prop based
// on its declared type. Strings come off the DOM as `string | null`;
// these turn that into the typed value the script expects.
//
// See docs/ship-set/01-language-spec.md § Prop types.

/**
 * Function that turns a raw HTML attribute string (or `null` for an
 * absent attribute) into the typed value the script block expects.
 * Each `Decoder<T>` is a pure function — no side effects, no DOM reads.
 */
export type Decoder<T> = (raw: string | null) => T

/**
 * Identity-style decoder. `null` (missing attribute) decodes to `''`
 * so prop signals never carry `null` for string-typed props.
 *
 * @param raw Raw attribute value or `null` if absent.
 * @returns The string, or `''` if missing.
 *
 * @example
 * ```ts
 * decodeString('hello') // 'hello'
 * decodeString(null)    // ''
 * ```
 */
export const decodeString: Decoder<string> = (raw) => raw ?? ''

/**
 * Number decoder. Empty or missing attributes decode to `NaN` — let the
 * author guard with `Number.isNaN` rather than silently choosing `0`.
 *
 * @param raw Raw attribute value or `null` if absent.
 * @returns Parsed number, or `NaN` for missing/empty input.
 *
 * @example
 * ```ts
 * decodeNumber('42')   // 42
 * decodeNumber('')     // NaN
 * decodeNumber(null)   // NaN
 * ```
 */
export const decodeNumber: Decoder<number> = (raw) => {
  if (raw === null || raw === '') return NaN
  return Number(raw)
}

/**
 * Boolean decoder using HTML attribute-presence semantics.
 *
 * - Missing attribute (`null`) → `false`
 * - Present with empty value (`""`) → `true` (e.g. `<x disabled>`)
 * - `"true"` / `"false"` (case-insensitive) → respective boolean
 * - Anything else present → `true`
 *
 * @param raw Raw attribute value or `null` if absent.
 * @returns The decoded boolean.
 *
 * @example
 * ```ts
 * decodeBoolean(null)    // false
 * decodeBoolean('')      // true  — `<x disabled>`
 * decodeBoolean('false') // false
 * decodeBoolean('on')    // true
 * ```
 */
export const decodeBoolean: Decoder<boolean> = (raw) => {
  if (raw === null) return false
  const v = raw.toLowerCase()
  if (v === 'false') return false
  return true
}

/**
 * Build a decoder that parses JSON from an attribute. Empty / missing
 * attributes and parse errors both yield `fallback`, so prop signals
 * never carry `undefined` and never throw on bad input.
 *
 * @param fallback Value to use when the attribute is missing, empty,
 *   or fails to parse.
 * @returns A `Decoder<T>` configured with the given fallback.
 *
 * @example
 * ```ts
 * const decodeItems = decodeJson<string[]>([])
 * decodeItems('["a","b"]') // ['a', 'b']
 * decodeItems(null)        // []
 * decodeItems('not-json')  // []
 * ```
 */
export function decodeJson<T>(fallback: T): Decoder<T> {
  return (raw) => {
    if (raw === null || raw === '') return fallback
    try {
      return JSON.parse(raw) as T
    } catch {
      return fallback
    }
  }
}

/**
 * Build a decoder that allows only a fixed set of string literals.
 * Anything outside the allowed list (including `null`) decodes to
 * `fallback` — useful for variant props like `size` or `tone`.
 *
 * @param allowed Tuple of accepted string literals.
 * @param fallback Value used when the input isn't in `allowed`.
 * @returns A `Decoder<T>` validated against the allowed set.
 *
 * @example
 * ```ts
 * const decodeSize = decodeEnum(['sm', 'md', 'lg'] as const, 'md')
 * decodeSize('lg')   // 'lg'
 * decodeSize('huge') // 'md'
 * ```
 */
export function decodeEnum<T extends string>(allowed: readonly T[], fallback: T): Decoder<T> {
  const set = new Set<string>(allowed)
  return (raw) => (raw !== null && set.has(raw) ? (raw as T) : fallback)
}
