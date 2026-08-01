/**
 * Assign createTable identity onto a findings host (explorer's assignTreeHost
 * pattern). Host property is `table` — the load-bearing sail-table binding.
 */
export function assignTableHost(
  sel: string,
  table: { /* Table identity */ } & object,
): void {
  if (typeof document === 'undefined') return
  const el = document.querySelector(sel) as
    | (HTMLElement & { table?: object })
    | null
  if (el && el.table !== table) el.table = table
}
