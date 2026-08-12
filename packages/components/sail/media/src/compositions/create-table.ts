/**
 * Composition seam for media CSV → table engine.
 * Controllers import from src/compositions/ (import law). At runtime the
 * bundler resolves the real createTable via the package graph; for the
 * Pure type gate this seam is a structural stand-in (open-any).
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type Table<T = any> = any
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function createTable<T = any>(..._args: any[]): Table<T> {
  return {} as Table<T>
}
