// Per-icon lucide loader for `<sui-icon>`.
//
// The daemon owns the icon set: it embeds `lucide-icons.json` and serves one
// icon at a time from `/_deck/icons/<name>.svg` (deck-core::serve_sui_icon). A
// `<sui-icon name="search">` mount triggers a single tiny GET — typically
// <1KB, browser-HTTP-cached on subsequent mounts. The retired model fetched
// the whole ~410KB registry up-front on first mount, then resolved client-side
// against the loaded map.
//
// Aliases (`git` → `git-branch`, etc.) are resolved SERVER-side now (see
// `ICON_ALIASES` in deck-core/src/assets.rs) so the client stays alias-unaware
// and the wire URL is stable across alias renames.
//
// Tests prime the cache up-front from the committed JSON so the controller
// reaches its render path synchronously — no jsdom fetch needed.

import { DECK_PREFIX } from '@ship/sdk-client'

const ICON_URL_PREFIX = `${DECK_PREFIX}/icons/`

// Per-name cache keyed on the URL-bound name (after alias resolution if the
// caller did it; aliases are also handled by the daemon, so an aliased miss
// resolves transparently). A `null` value means "fetched and confirmed
// missing" — second lookups should NOT re-fetch.
type Entry = string | null
type CacheState = {
  resolved: Map<string, Entry>
  inflight: Map<string, Promise<Entry>>
}
const GLOBAL_KEY = '__shipIconCache__'
const g = globalThis as unknown as { [GLOBAL_KEY]?: CacheState }
function state(): CacheState {
  if (!g[GLOBAL_KEY]) g[GLOBAL_KEY] = { resolved: new Map(), inflight: new Map() }
  return g[GLOBAL_KEY]
}

/** Test-only: seed the cache so jsdom renders synchronously without fetch. */
export function primeIconCache(name: string, svg: string | null): void {
  state().resolved.set(name, svg)
}

/** Looked-up icon SVG (full `<svg>…</svg>`), or `null` if confirmed missing. */
export function getCachedIcon(name: string): Entry | undefined {
  return state().resolved.get(name)
}

/** Single-flight fetch of one icon. Resolves to the standalone SVG markup the
 *  daemon returns, or `null` for a 404 / network error (the caller renders
 *  the slot fallback in that case). */
export function fetchIcon(name: string): Promise<Entry> {
  const s = state()
  if (s.resolved.has(name)) return Promise.resolve(s.resolved.get(name) ?? null)
  const pending = s.inflight.get(name)
  if (pending) return pending
  const p: Promise<Entry> = fetch(ICON_URL_PREFIX + encodeURIComponent(name) + '.svg')
    .then((r) => (r.ok ? r.text() : null))
    .catch(() => null)
    .then((svg) => {
      s.resolved.set(name, svg)
      s.inflight.delete(name)
      return svg
    })
  s.inflight.set(name, p)
  return p
}

/** The URL the daemon serves a given icon from. Exported for callers that
 *  want a `<link rel="prefetch">` hint, or want to use the URL as `<img src>`
 *  (note: `currentColor` won't follow through `<img>` — use the SVG inline). */
export function iconUrl(name: string): string {
  return ICON_URL_PREFIX + encodeURIComponent(name) + '.svg'
}
