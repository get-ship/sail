/**
 * Assign createTable identity onto a findings host.
 *
 * Ambient `document.querySelector` host mutation is banned
 * (`sail-no-ambient-dom`). Until templates bind `prop:table={table}`, this
 * is a no-op.
 *
 * // SAIL-LINT-CHAT: migrate findings table host to prop:table={table} identity
 * // binding in checks-surface.sail + vm return; delete this assign helper once
 * // bound.
 */

export function assignTableHost(
  _sel: string,
  _table: { /* Table identity */ } & object,
): void {
  /* intentional no-op — see SAIL-LINT-CHAT above */
}
