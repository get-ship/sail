// Lucide icon map loader for vitest setups. Reads the committed
// `/home/madvib/dev/sui/crates/deck/deck-core/assets/lucide-icons.json` (the
// SAME embed the daemon serves at `/_deck/icons/{name}.svg`), wraps each
// fragment in the canonical `<svg>` envelope, and returns a
// `Map<name, full-svg-markup>` for the caller to push into
// `@ship/sui-primitives`'s per-icon cache.
//
// We do NOT depend on `@ship/sui-primitives` here — keeps test-setup free of
// component-package cycles. Callers pass `primeIconCache` in or iterate
// the map themselves.

import { existsSync, readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

/** Absolute location of the committed lucide registry — post-carve it lives
 *  in the external sui repo (`/home/madvib/dev/sui/crates/deck/deck-core/
 *  assets/lucide-icons.json` is committed source per sui's
 *  MIGRATION-NOTES.md). TEMP: absolute → `../sui/...` after layout
 *  reshuffle. */
const LUCIDE_ABS = '/home/madvib/dev/sui/crates/deck/deck-core/assets/lucide-icons.json'

/**
 * Resolve the committed `lucide-icons.json` independent of `process.cwd()`.
 * Post-carve the file lives at a fixed absolute path in the external sui
 * repo (no relative walk needed).
 */
export function defaultLucideIconsPath(): string {
  if (existsSync(LUCIDE_ABS)) return LUCIDE_ABS
  throw new Error(
    `lucide-icons.json not found at ${LUCIDE_ABS} — the carve points at the external sui repo; run \`cd /home/madvib/dev/sui && ls crates/deck/deck-core/assets/lucide-icons.json\` to confirm the source is present.`,
  )
}

/** Aliases mirrored from `deck-core/src/assets.rs::ICON_ALIASES`. The daemon
 *  resolves these server-side; tests prime the cache identically so a
 *  `<sui-icon name="git">` test renders the same SVG it would in production. */
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
