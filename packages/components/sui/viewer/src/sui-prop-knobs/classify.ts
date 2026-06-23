// Pure classification + coercion for <sui-prop-knobs> (build-spec §2).
//
// CEM `attributes[]` → a FOUR-way classification. The `type.text` string is the
// discriminator (verified `cem.rs` `prop_type_text`: Number→'number',
// Bool→'boolean', String→'string', Union→the `'a' | 'b'` joined string with a
// separate `values: string[]`). No DOM here — a framework-pure transform.

/** A CEM declaration entry as emitted by `cem.rs` `attr_entry`. */
export interface CemAttr {
  name: string
  type: { text: string }
  default?: string
  values?: string[]
  description?: string
}

export type KnobKind = 'enum' | 'bool' | 'number' | 'string'

/** A classified knob the template iterates. The current value is exposed as a
 *  STABLE per-knob signal (`value`/`checked`) so the template's reactive
 *  zero-arg read `prop:value={knob.value()}` re-fires when the controlled value
 *  changes — independent of the keyed {#for} row identity (which is the prop
 *  name, never re-keyed on a value edit). */
export interface Knob {
  name: string
  kind: KnobKind
  options: string[]
  /** Stable signal: the current string-rendered value (enum / number / string). */
  value: () => string
  /** Stable signal: the current boolean value (bool knob). */
  checked: () => boolean
}

export type KnobValues = Record<string, unknown>

/** The largest legible variant of a size enum (studio legibility rule). */
const SIZE_ORDER = ['xs', 'sm', 'md', 'lg', 'xl']

/**
 * Classify ONE CEM attribute into one of the four knob kinds. Order matters:
 * a union (`values[]` present) is an enum regardless of `type.text`; only when
 * there are no enum members do we fall through to boolean / number / string.
 */
export function classifyKnob(attr: CemAttr): KnobKind {
  if (attr.values && attr.values.length > 0) return 'enum'
  const t = attr.type?.text ?? 'string'
  if (t === 'boolean') return 'bool'
  if (t === 'number') return 'number'
  return 'string'
}

/** Enum members for the segmented control (empty for non-enum kinds). */
export function enumOptions(attr: CemAttr): string[] {
  return attr.values && attr.values.length > 0 ? attr.values : []
}

/**
 * Coerce a raw control value to the knob's authored kind so a numeric prop
 * never receives a string and a boolean prop never receives `'true'`. This is
 * the load-bearing fix over studio's three-bucket classifier (which silently
 * mis-typed numbers).
 */
export function coerceValue(attr: CemAttr, raw: unknown): unknown {
  const kind = classifyKnob(attr)
  if (kind === 'bool') return raw === true || raw === 'true' || raw === ''
  if (kind === 'number') {
    const n = typeof raw === 'number' ? raw : Number(raw)
    return Number.isFinite(n) ? n : 0
  }
  // enum + string both ride as strings
  return raw == null ? '' : String(raw)
}

/** Coerce a CEM `default` string to the knob's kind for seeding. */
function coerceDefault(attr: CemAttr): unknown {
  const kind = classifyKnob(attr)
  const d = attr.default
  if (kind === 'bool') return d === 'true' || d === ''
  if (kind === 'number') {
    const n = d == null ? NaN : Number(d)
    return Number.isFinite(n) ? n : 0
  }
  return d ?? ''
}

/**
 * Seed the initial `propValues` from each attribute's declared default. An
 * enum named `size` (or `*-size`) with no explicit default prefers the largest
 * variant present (studio's legibility rule); other enums fall back to the
 * first member when no default is declared.
 */
export function seedDefaults(attrs: CemAttr[]): KnobValues {
  const out: KnobValues = {}
  for (const attr of attrs) {
    const kind = classifyKnob(attr)
    if (kind === 'enum') {
      const opts = enumOptions(attr)
      if (attr.default) {
        out[attr.name] = attr.default
      } else if (isSizeEnum(attr.name)) {
        out[attr.name] = largestSize(opts) ?? opts[0] ?? ''
      } else {
        out[attr.name] = opts[0] ?? ''
      }
    } else {
      out[attr.name] = coerceDefault(attr)
    }
  }
  return out
}

function isSizeEnum(name: string): boolean {
  return name === 'size' || name.endsWith('-size') || name.endsWith('Size')
}

/** The largest variant present in `opts` by the canonical size order. */
function largestSize(opts: string[]): string | undefined {
  let best: string | undefined
  let bestRank = -1
  for (const o of opts) {
    const rank = SIZE_ORDER.indexOf(o)
    if (rank > bestRank) {
      bestRank = rank
      best = o
    }
  }
  return best
}
