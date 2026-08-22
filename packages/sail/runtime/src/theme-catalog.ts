// theme-catalog — effective-emit-set resolution for the product theme picker.
//
// Extracted from the retired `use:themeSelect` action: these are pure data
// helpers (meta parsing, allowlist resolution, catalog filtering) with no DOM
// writes and no element knowledge. The `<theme-picker>` controller consumes
// them; deck keeps stamping `<meta name="sail-effective-themes">`.
import { THEMES, type Theme, type ThemeId } from '@ship/tokens/themes'

/** Theme id shape aligned with sail-compiler `validate_theme_id` (no path chars). */
const SAFE_THEME_ID = /^[a-z0-9][a-z0-9-]*$/

function isSafeThemeIdToken(id: string): boolean {
  return SAFE_THEME_ID.test(id) && !id.includes('..')
}

/**
 * Per-action ops session for the product grid.
 * - `undefined` — no ops path / unrestricted after fail-open
 * - `'pending'` — ops in flight (fail-closed empty grid)
 * - `string[]` — resolved allowlist (may be empty)
 */
export type EffectiveSessionIds = readonly string[] | 'pending' | undefined

/**
 * Test / host override for the product grid allowlist.
 * - `null` → resolve from meta / ops / full THEMES
 * - `[]` → empty product grid (explicit thin)
 * - `['neutral','paper']` → only those catalog rows
 */
let effectiveThemeIdsOverride: readonly string[] | null = null

/** Set product-grid theme ids (tests / host). Pass `null` to clear. */
export function setEffectiveThemeIds(ids: readonly string[] | null): void {
  effectiveThemeIdsOverride = ids
}

/**
 * Parse effective theme ids from a meta content string.
 * `"all"` → null (full catalog). Empty / whitespace → empty list.
 */
export function parseEffectiveThemeIdsContent(raw: string | null | undefined): string[] | null {
  if (raw == null) return null
  const trimmed = raw.trim()
  if (!trimmed) return []
  if (trimmed === 'all') return null
  return trimmed
    .split(/[,\s]+/)
    .map((s) => s.trim())
    .filter((id) => id.length > 0 && isSafeThemeIdToken(id))
}

function hasSailProjectMeta(): boolean {
  if (typeof document === 'undefined') return false
  const project = document
    .querySelector('meta[name="sail-project"]')
    ?.getAttribute('content')
    ?.replace(/\/$/, '')
  return Boolean(project && !project.includes('..'))
}

function hasEffectiveThemesMeta(): boolean {
  if (typeof document === 'undefined') return false
  return Boolean(document.querySelector('meta[name="sail-effective-themes"]'))
}

/**
 * Read stamped effective ids.
 * `null` means unrestricted (full `THEMES`).
 *
 * Order: override → meta → session (ops) → unrestricted.
 * Session `'pending'` yields an empty allowlist (fail-closed while ops loads).
 */
export function readEffectiveThemeIds(sessionIds?: EffectiveSessionIds): string[] | null {
  if (effectiveThemeIdsOverride !== null) {
    return [...effectiveThemeIdsOverride].filter(isSafeThemeIdToken)
  }
  // Meta stamps win over stale ops session results (and over pending).
  if (typeof document !== 'undefined') {
    const meta = document.querySelector('meta[name="sail-effective-themes"]')
    if (meta) {
      return parseEffectiveThemeIdsContent(meta.getAttribute('content'))
    }
  }
  if (sessionIds === 'pending') return []
  if (sessionIds != null) {
    return [...sessionIds].filter(isSafeThemeIdToken)
  }
  return null
}

/**
 * Themes shown in the product picker grid.
 *
 * Filters `@ship/tokens` `THEMES` by the effective emit set when known
 * (effective ∩ presentation catalog). Project-only stems without a THEMES
 * entry are not rendered. Lookup helpers (`isValidTheme`, modes) still use
 * the full catalog.
 */
export function effectiveProductThemes(sessionIds?: EffectiveSessionIds): Theme[] {
  const ids = readEffectiveThemeIds(sessionIds)
  if (ids === null) return THEMES
  if (ids.length === 0) return []
  const allow = new Set(ids)
  return THEMES.filter((t) => allow.has(t.id as ThemeId))
}

/** Best-effort ops fetch when meta is absent (DEV host with SailTheme). */
export async function tryFetchEffectiveIdsFromOps(): Promise<string[] | null> {
  if (typeof fetch === 'undefined' || typeof document === 'undefined') return null
  const project = document
    .querySelector('meta[name="sail-project"]')
    ?.getAttribute('content')
    ?.replace(/\/$/, '')
  if (!project || project.includes('..')) return null
  try {
    const res = await fetch('/sail.ops.v1.SailTheme/EffectiveIds', {
      method: 'POST',
      headers: { 'content-type': 'application/json', accept: 'application/json' },
      body: JSON.stringify({ project }),
      cache: 'no-store',
    })
    if (!res.ok) return null
    const json = (await res.json().catch(() => null)) as {
      ok?: boolean
      all?: unknown
      code?: unknown
    } | null
    if (!json || json.code || !json.ok || !Array.isArray(json.all)) return null
    const ids = json.all.map((x) => String(x)).filter((id) => isSafeThemeIdToken(id))
    return ids
  } catch {
    return null
  }
}

export { hasSailProjectMeta, hasEffectiveThemesMeta }
