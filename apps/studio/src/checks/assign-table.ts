/**
 * Assign createTable identity onto a findings host.
 *
 * Ambient `document.querySelector` host mutation is banned
 * (`sail-no-ambient-dom`). Until templates bind `prop:table={table}`, this
 * is a no-op.
 */

export function assignTableHost(
  _sel: string,
  _table: { /* Table identity */ } & object,
): void {
  /* no-op until prop:table bind */
}
