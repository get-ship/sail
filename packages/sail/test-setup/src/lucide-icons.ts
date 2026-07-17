// Lucide icon map loader for vitest setups. Reads the committed
// `crates/deck/deck-core/assets/lucide-icons.json` (the SAME embed the daemon
// serves at `/_deck/icons/{name}.svg`), wraps each fragment in the canonical
// `<svg>` envelope, and returns a `Map<name, full-svg-markup>` for the caller
// to push into `@ship/sail-primitives`'s per-icon cache.
//
// We do NOT depend on `@ship/sail-primitives` here — keeps test-setup free of
// component-package cycles. Callers pass `primeIconCache` in or iterate
// the map themselves.

import { existsSync, readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

/** The committed lucide registry lives in this same repo at
 *  `crates/deck/deck-core/assets/lucide-icons.json` (the SAME embed the daemon
 *  serves at `/_deck/icons/{name}.svg`). Resolved RELATIVE to this module so it
 *  survives the checkout moving on disk; `LUCIDE_ICONS_JSON` overrides for
 *  out-of-tree consumers (the published `@ship/test-setup` installed outside
 *  the sui repo, where the `crates/` sibling isn't present). */
const DEFAULT_LUCIDE = resolve(
  dirname(fileURLToPath(import.meta.url)),
  '../../../../crates/deck/deck-core/assets/lucide-icons.json',
)

/**
 * Resolve the committed `lucide-icons.json` independent of `process.cwd()` —
 * relative to this module (move-resilient), override via `LUCIDE_ICONS_JSON`.
 */
export function defaultLucideIconsPath(): string {
  const candidate = process.env.LUCIDE_ICONS_JSON ?? DEFAULT_LUCIDE
  if (existsSync(candidate)) return candidate
  throw new Error(
    `lucide-icons.json not found at ${candidate}. It lives in the sui repo at ` +
    `crates/deck/deck-core/assets/lucide-icons.json; set LUCIDE_ICONS_JSON when ` +
    `running @ship/test-setup outside the sui checkout.`,
  )
}

/** Aliases mirrored from `deck-core/src/assets.rs::ICON_ALIASES`. The daemon
 *  resolves these server-side; tests prime the cache identically so a
 *  `<sail-icon name="git">` test renders the same SVG it would in production. */
const ICON_ALIASES = [
  ['git', 'git-branch'],
  ['refresh', 'refresh-cw'],
  ['stop', 'stop-circle'],
  ['warning', 'triangle-alert'],
] as const

/** Wrap a lucide inner-svg fragment in the canonical envelope — kept in
 *  lockstep with `deck-core/src/assets.rs::serve_sui_icon`. */
function wrap(inner: string): string {
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" `
    + `stroke="currentColor" stroke-width="2" stroke-linecap="round" `
    + `stroke-linejoin="round">${inner}</svg>`
}

/** Read the committed lucide JSON + return a map of full-svg-markup per
 *  icon name (including alias names that point at their resolved svg). */
export function loadLucideIconMap(jsonPath: string): Map<string, string> {
  const icons: Record<string, string> = JSON.parse(readFileSync(jsonPath, 'utf8'))
  const out = new Map<string, string>()
  for (const [name, inner] of Object.entries(icons)) {
    out.set(name, wrap(inner))
  }
  for (const [alias, real] of ICON_ALIASES) {
    if (icons[real]) out.set(alias, wrap(icons[real]))
  }
  return out
}

/** Convenience: read the JSON + push every entry through `primer`. When no
 *  path is given, resolves the committed registry via [`defaultLucideIconsPath`]
 *  (cwd-independent) — the form every setup should use. */
export function primeLucideIcons(
  primer: (name: string, svg: string) => void,
  jsonPath: string = defaultLucideIconsPath(),
): void {
  for (const [name, svg] of loadLucideIconMap(jsonPath)) primer(name, svg)
}
