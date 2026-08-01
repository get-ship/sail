/** Shared sail-tree identity assignment (createTable → host.tree). */
export function assignTreeHost<T>(
  sel: string,
  table: { /* Table identity */ } & object,
): void {
  if (typeof document === 'undefined') return
  const el = document.querySelector(sel) as (HTMLElement & { tree?: object }) | null
  if (el && el.tree !== table) el.tree = table
}
