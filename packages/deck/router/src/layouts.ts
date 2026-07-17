// Layout-chain resolution — the canonical home (promoted from the shell's
// host-side `src/route/layouts.ts`). `mountRouter` walks the `LAYOUTS` table
// emitted by `crates/deck/deck-bundler/src/routes.rs` for the current URL,
// builds the chrome chain, and mounts the route content into the innermost
// `<div slot="route">`. Pure: same inputs, same outputs.
//
//   export const LAYOUTS: LayoutShape[] = [
//     { pathPrefix: '/studio', tag: 'sail-studio-chrome' },
//   ]

export interface LayoutShape {
  /** URL pathname prefix that triggers this layout. */
  pathPrefix: string
  /** Custom-element tag to mount (the chrome wrapper). */
  tag: string
}

/**
 * Walk LAYOUTS for the current URL. Longest-prefix match wins.
 *
 * Returns the chrome wrappers **deepest-first**: index 0 is the innermost
 * (closest to the route content), the last is the outermost (closest to the
 * document root). An empty return means "no chrome — mount the route content
 * directly into the root element." The function is pure; the caller builds the
 * chain and mounts.
 */
export function resolveLayoutChain(
  url: string | URL,
  layouts: readonly LayoutShape[],
): LayoutShape[] {
  const pathname = pathnameOf(url)
  const matches: LayoutShape[] = []
  for (const layout of layouts) {
    if (!layout || typeof layout.pathPrefix !== 'string' || layout.pathPrefix.length === 0) {
      continue
    }
    if (matchPrefix(pathname, layout.pathPrefix)) matches.push(layout)
  }
  matches.sort((a, b) => b.pathPrefix.length - a.pathPrefix.length)
  return matches
}

/**
 * Match a pathname against a layout prefix. The prefix matches if:
 *  - `pathname === prefix` (exact), or
 *  - `pathname` starts with `prefix + '/'` (e.g. `/studio/agents` matches `/studio`), or
 *  - `prefix === '/'` (matches every pathname).
 */
function matchPrefix(pathname: string, prefix: string): boolean {
  if (prefix === '/') return true
  if (pathname === prefix) return true
  return pathname.startsWith(prefix + '/')
}

/** Extract the pathname from a URL string or URL object. */
function pathnameOf(url: string | URL): string {
  if (typeof url === 'string') {
    if (url.startsWith('http://') || url.startsWith('https://') || url.startsWith('file://')) {
      try {
        return new URL(url).pathname
      } catch {
        return url
      }
    }
    return url.split('?')[0]?.split('#')[0] || '/'
  }
  return url.pathname
}
