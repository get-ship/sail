// <sui-cmdk> — declarative/SSR decode (the sui-tree/sui-combobox pattern).
//
// The live `items` IDENTITY property (a JS `CmdkItem[]`) is the load-bearing
// path. This parser exists ONLY for the declarative/SSR fallback: a
// `<sui-cmdk items='[…]'>` with no parent controller. The compiler's <prop>
// vocabulary is `string | bool | number` (`type="json"` does not compile), so
// structured items arrive as a JSON string parsed here.
//
// PURE TypeScript — no runtime/SDK imports.

/** A command-palette entry. Only `id` + `label` are required. */
export interface CmdkItem {
  id: string
  label: string
  /** Emitted in `sui-cmdk-select`; defaults to `id`. */
  value?: string
  /** A `<sui-icon>` name rendered at the row's lead. */
  icon?: string
  /** Trailing hint text (e.g. a keyboard shortcut). */
  hint?: string
  /** Section heading; a new heading row is inserted when it changes. */
  group?: string
  /** Extra text folded into the substring match beyond `label`. */
  keywords?: string
  /** Rendered dimmed; skipped by keyboard nav + click. */
  disabled?: boolean
}

/** Coerce an arbitrary value to a string (null/undefined → `''`). */
export const str = (v: unknown): string => (v == null ? '' : String(v))

/** Normalize one raw entry into a CmdkItem (drops entries with no id/label). */
export function normalizeItem(entry: unknown, i: number): CmdkItem | null {
  if (entry == null || typeof entry !== 'object') return null
  const o = entry as Record<string, unknown>
  const id = o.id != null ? String(o.id) : o.value != null ? String(o.value) : String(i)
  if (!id) return null
  const item: CmdkItem = { id, label: o.label != null ? String(o.label) : id }
  if (o.value != null) item.value = String(o.value)
  if (o.icon != null) item.icon = String(o.icon)
  if (o.hint != null) item.hint = String(o.hint)
  if (o.group != null) item.group = String(o.group)
  if (o.keywords != null) item.keywords = String(o.keywords)
  if (o.disabled != null) item.disabled = Boolean(o.disabled)
  return item
}

/** Parse the `items='[…]'` JSON attribute (SSR fallback). Never throws. */
export function parseJsonItems(raw: string): CmdkItem[] {
  if (!raw) return []
  let parsed: unknown
  try {
    parsed = JSON.parse(raw)
  } catch {
    return []
  }
  if (!Array.isArray(parsed)) return []
  return parsed
    .map((entry, i) => normalizeItem(entry, i))
    .filter((it): it is CmdkItem => it !== null)
}
