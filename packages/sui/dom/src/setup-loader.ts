// Resolver for the paired `.ts` setup file referenced by a `.sui`
// view. Given the URL of the template, look for a sibling file with
// the same base name and a `.ts` extension; dynamic-import it and
// return the default export.
//
// The interpreter never invokes this on its own — `hydrate` accepts
// an already-resolved `setup` thunk. This helper exists so tooling
// (devserver, view loader) can do the resolution in one place.

/**
 * Locate the `.ts` setup file paired with a `.sui` template and
 * dynamic-import it.
 *
 * Resolution rules:
 *  - Replace the trailing `.sui` extension with `.ts`.
 *  - If `setUrl` lacks `.sui`, append `.ts` to its directory and
 *    treat the trailing segment as the setup file name.
 *  - Use `globalThis.fetch` (or a custom `loader`) to verify the
 *    paired file exists; if it returns 404 the loader resolves with
 *    `undefined` instead of throwing.
 *
 * The dynamic import is wrapped in a try/catch — modules that fail to
 * import surface their error via the returned promise so callers can
 * decide whether to render anyway or short-circuit.
 *
 * @param setUrl Absolute or root-relative URL of the `.sui` file.
 * @param loader Optional dynamic-import shim. Defaults to `import(...)`.
 * @returns The module's `default` export when present and callable,
 *   otherwise `undefined`.
 *
 * @example
 * ```ts
 * const setup = await loadSetup('/views/agents/view.sui')
 * hydrate(template, setup, { container })
 * ```
 */
export async function loadSetup(
  setUrl: string,
  loader: (path: string) => Promise<unknown> = (p) => import(/* @vite-ignore */ p),
): Promise<(() => unknown) | undefined> {
  const tsUrl = pairedTsPath(setUrl)
  let mod: { default?: unknown } | undefined
  try {
    mod = (await loader(tsUrl)) as { default?: unknown }
  } catch {
    return undefined
  }
  const def = mod?.default
  if (typeof def === 'function') return def as () => unknown
  if (def && typeof def === 'object' && 'setup' in def) {
    const setup = (def as { setup?: unknown }).setup
    if (typeof setup === 'function') return setup as () => unknown
  }
  return undefined
}

/** Compute the paired `.ts` path for a given `.sui` URL. */
export function pairedTsPath(setUrl: string): string {
  if (setUrl.endsWith('.sui')) {
    return setUrl.slice(0, -'.sui'.length) + '.ts'
  }
  if (setUrl.endsWith('.set')) {
    return setUrl.slice(0, -'.set'.length) + '.ts'
  }
  return setUrl + '.ts'
}
