// sui-tree — SSR/declarative fallback decode (the sui-combobox pattern).
//
// `prop:tree={tree}` IDENTITY binding is the load-bearing path; the engine is
// NEVER serialized. These parsers exist ONLY for the declarative/SSR fallback:
// a `<sui-tree nodes='[…]'>` with no parent controller. The compiler's `<prop>`
// vocabulary is `string | bool | number` (`type="json"` does not compile), so
// structured tree data arrives as a JSON string parsed here.
//
// PURE TypeScript — no runtime/SDK imports.
//
// Authority: build-spec §"Prop wiring (LAYER 2 — JSON-string fallback)", §Props,
// Slice 1.

import type { TreeNode } from '../sui-tree-view/sui-tree-view.controller'
import type { ColumnDef } from '../sui-table/engine/types'

export type DecodedTreeNode = TreeNode & Record<string, unknown>

/** Parse the `nodes` string prop into a `TreeNode[]`. Each node needs an `id`;
 *  malformed JSON yields `[]` (never throws). Children recurse via `children`. */
export function parseJsonNodes(raw: string): DecodedTreeNode[] {
  if (!raw) return []
  let parsed: unknown
  try {
    parsed = JSON.parse(raw)
  } catch {
    return []
  }
  return normalizeNodes(parsed)
}

function normalizeNodes(value: unknown): DecodedTreeNode[] {
  if (!Array.isArray(value)) return []
  const out: DecodedTreeNode[] = []
  for (let i = 0; i < value.length; i++) {
    const entry = value[i]
    if (entry == null || typeof entry !== 'object') continue
    const o = entry as Record<string, unknown>
    const id = o.id != null ? String(o.id) : String(i)
    const node: DecodedTreeNode = {
      ...o,
      id,
      label: o.label != null ? String(o.label) : id,
    }
    if (o.children != null) node.children = normalizeNodes(o.children)
    out.push(node)
  }
  return out
}

/** Parse the `columns` string prop into accessor column defs (tree-grid only).
 *  Accepts the `{ field, label }` floor and the full `{ id, accessorKey, header }`
 *  shape; malformed JSON yields `[]`. */
export function parseJsonColumns(raw: string): ColumnDef<DecodedTreeNode>[] {
  if (!raw) return []
  let parsed: unknown
  try {
    parsed = JSON.parse(raw)
  } catch {
    return []
  }
  if (!Array.isArray(parsed)) return []
  return parsed
    .map((entry): ColumnDef<DecodedTreeNode> | null => {
      if (entry == null || typeof entry !== 'object') return null
      const o = entry as Record<string, unknown>
      const field = o.accessorKey ?? o.field ?? o.id
      if (field == null) return null
      const key = String(field)
      return {
        id: o.id != null ? String(o.id) : key,
        accessorKey: key as keyof DecodedTreeNode & string,
        header: o.header != null ? String(o.header) : o.label != null ? String(o.label) : key,
      }
    })
    .filter((c): c is ColumnDef<DecodedTreeNode> => c !== null)
}
