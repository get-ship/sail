// snippet.ts — the paste-able-snippet default-diff builder + the ONE clipboard
// helper for the whole pack (build-spec §The viewing components #5).
//
// This is the studio `snippet.ts` LOGIC reproduced over the CEM `attributes[]`
// shape: emit ONLY props differing from the declared default; booleans use
// bare-attribute presence (omitted when false/default); strings are
// attr-escaped; the inner content is composed slot markup (or a humanized
// label). Pure transforms — no DOM, no state, no events.
//
// `copyText` is the single `navigator.clipboard` write site in the pack
// (sui-render's folded error branch reuses it — one way to do things). It takes
// the `Navigator` as a param so it stays testable and never assumes a global.

/** A CEM `attributes[]` entry — the `name`/`type:{text}`/`default`/`values`
 *  shape the host bakes from each component's `.custom-elements.json`. The
 *  default-diff reads `default` to decide what to omit. */
export interface CemAttr {
  name: string
  type?: { text?: string }
  default?: string
  values?: string[]
}

const BOOL_TYPE_TEXT = new Set(['boolean', 'Boolean', 'bool'])

function isBoolAttr(attr: CemAttr | undefined): boolean {
  const t = attr?.type?.text
  return t !== undefined && BOOL_TYPE_TEXT.has(t)
}

/** Heuristic for degraded (no-CEM) mode: a value that reads as a boolean. */
function looksBool(v: unknown): v is boolean {
  return v === true || v === false
}

function escapeAttr(v: string): string {
  return v.replace(/&/g, '&amp;').replace(/"/g, '&quot;')
}

/** A humanized fallback for the snippet's inner content when no slot markup is
 *  supplied: `sui-button` → `Button`. Mirrors studio `slotLabel`. */
export function slotLabel(tag: string): string {
  const base = tag
    .replace(/^sui-/, '')
    .replace(/^ship-/, '')
    .replace(/-/g, ' ')
    .trim()
  return base ? base.charAt(0).toUpperCase() + base.slice(1) : 'Preview'
}

/**
 * Build a paste-able `<tag …>inner</tag>` snippet from the current prop values.
 *
 * - With a CEM (`props`): default-diff — emit only props that differ from the
 *   declared default; booleans → bare-attribute presence (omitted when
 *   false/default).
 * - Without a CEM (`props` is `undefined`/empty): degraded mode — emit ALL
 *   current values (no default to diff against), still skipping false booleans
 *   and empty strings (they carry no information).
 */
export function buildSnippet(
  tag: string,
  values: Record<string, unknown>,
  props: readonly CemAttr[] | undefined,
  slotContent: string | undefined,
): string {
  const cemByName = new Map<string, CemAttr>()
  for (const p of props ?? []) cemByName.set(p.name, p)
  const hasCem = (props?.length ?? 0) > 0

  const attrs: string[] = []
  const emit = (name: string, value: unknown, attr: CemAttr | undefined): void => {
    // Boolean: presence semantics. Omit when false, or (CEM mode) when it
    // matches a true default that's already implied.
    if (isBoolAttr(attr) || (!hasCem && looksBool(value))) {
      const truthy = value === true || value === 'true'
      const defaultTrue = attr?.default === 'true'
      if (truthy && !defaultTrue) attrs.push(name)
      return
    }
    if (value == null || value === '') return
    const str = String(value)
    // CEM mode: omit a value equal to its declared default. Degraded mode: no
    // default known, so every non-empty value is emitted.
    if (hasCem && attr?.default !== undefined && str === String(attr.default)) return
    attrs.push(`${name}="${escapeAttr(str)}"`)
  }

  // Iterate the CEM order when present (stable, declaration order); otherwise
  // the current-values order (degraded mode).
  const names = hasCem ? (props ?? []).map((p) => p.name) : Object.keys(values)
  for (const name of names) emit(name, values[name], cemByName.get(name))

  const attrStr = attrs.length ? ' ' + attrs.join(' ') : ''
  const inner = slotContent !== undefined && slotContent !== '' ? slotContent.trim() : slotLabel(tag)
  return `<${tag}${attrStr}>${inner}</${tag}>`
}

/**
 * The ONE clipboard write in the pack. Writes `text` to `nav.clipboard` and
 * resolves `true` on success, `false` (never a throw) when the API is absent or
 * the write is denied — so the caller can drive transient feedback off the
 * result without a try/catch at every call site.
 */
export async function copyText(text: string, nav: Navigator = navigator): Promise<boolean> {
  const writer = nav?.clipboard?.writeText
  if (typeof writer !== 'function') return false
  try {
    await nav.clipboard.writeText(text)
    return true
  } catch {
    return false
  }
}
