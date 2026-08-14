// sail-table engine — SSR/declarative fallback decode (the sail-combobox pattern).
//
// `prop:table={table}` identity binding is the load-bearing path; the engine is
// NEVER serialized. These parsers exist ONLY for the declarative/SSR fallback:
// a `<sail-table variant="data" rows='[…]' columns='[…]'>` with no parent
// controller. The compiler's `<prop>` vocabulary is `string | bool | number`
// (`type="json"` does not compile), so structured data arrives as a JSON string
// parsed here — exactly as `sail-combobox` decodes its `options` attribute.
//
// PURE TypeScript — no runtime/SDK imports.
//
// Authority: build-spec §"Prop wiring (LAYER 2 — JSON-string fallback)", §Props.

import type { AccessorColumn, ColumnDef } from './types'

export type DecodedRow = Record<string, unknown> & { id: string }

/** Parse the `rows` string prop into row records. Each row needs an `id`. */
export function parseJsonRows(raw: string): DecodedRow[] {
  if (!raw) return []
  let parsed: unknown
  try {
    parsed = JSON.parse(raw)
  } catch {
    return []
  }
  if (!Array.isArray(parsed)) return []
  return parsed
    .filter((r): r is Record<string, unknown> => r != null && typeof r === 'object')
    .map((r, i) => ({ ...r, id: r.id != null ? String(r.id) : String(i) }))
}

/** Parse the `columns` string prop into accessor column defs. Accepts the
 *  older `{ field, label }` floor and maps it to `{ id, accessorKey, header }`,
 *  or a full `{ id, accessorKey, header }` object. */
export function parseJsonColumns(raw: string): ColumnDef<DecodedRow>[] {
  if (!raw) return []
  let parsed: unknown
  try {
    parsed = JSON.parse(raw)
  } catch {
    return []
  }
  if (!Array.isArray(parsed)) return []
  return parsed
    .map((entry): ColumnDef<DecodedRow> | null => {
      if (entry == null || typeof entry !== 'object') return null
      const o = entry as Record<string, unknown>
      const field = o.accessorKey ?? o.field ?? o.id
      if (field == null) return null
      const key = String(field)
      const col: AccessorColumn<DecodedRow> = {
        id: o.id != null ? String(o.id) : key,
        accessorKey: key,
        header: o.header != null ? String(o.header) : o.label != null ? String(o.label) : key,
      }
      return col
    })
    .filter((c): c is ColumnDef<DecodedRow> => c !== null)
}
