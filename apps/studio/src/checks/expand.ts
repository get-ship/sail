/**
 * Explicit expanded map for rule groups.
 *
 * createTable `expanded: true` is expand-all; bare `toggleExpanded(id)` is a
 * no-op in that state. Always use a Record map so collapse/expand is real.
 */

/** Build { [groupRowId]: open } from group ids + collapsed rule names. */
export function buildExpandedMap(
  groups: Array<{ id: string; rule: string }>,
  collapsedRules: ReadonlySet<string>,
): Record<string, boolean> {
  const map: Record<string, boolean> = {}
  for (const g of groups) {
    map[g.id] = !collapsedRules.has(g.rule)
  }
  return map
}

/** Toggle a rule in the collapsed set (returns a new Set). */
export function toggleCollapsedRule(
  collapsedRules: ReadonlySet<string>,
  rule: string,
): Set<string> {
  const next = new Set(collapsedRules)
  if (next.has(rule)) next.delete(rule)
  else next.add(rule)
  return next
}

/**
 * Collect group rows from a createTable-like rows() list.
 * Group headers are always present in the flatten even when collapsed.
 */
export function groupMetaFromRows(
  rows: Array<{
    id: string
    getIsGrouped: () => boolean
    getGroupingValue: (col: string) => unknown
    getValue: (col: string) => unknown
  }>,
): Array<{ id: string; rule: string }> {
  const out: Array<{ id: string; rule: string }> = []
  for (const row of rows) {
    if (!row.getIsGrouped()) continue
    const rule = String(
      row.getGroupingValue('rule') ?? row.getValue('rule') ?? '',
    )
    out.push({ id: row.id, rule })
  }
  return out
}
