/**
 * Shared theme catalog for Sail apps and the marketing site.
 *
 * Every theme is a full aesthetic bundle — palette, surfaces, radii,
 * shadows, motion, typography, and interaction strength. Dark is a
 * **mode** (`data-mode`), not a twin theme id.
 *
 * Palette definitions: `./themes.css` (per-theme mode overrides) and
 * `./tokens.css` (base). Consumers need this module for theme ids,
 * human labels, swatches, and apply helpers.
 */

// ThemeId is generated from model/themes via `sail-compiler tokens`.
import type { ThemeId } from './generated/unions'
import {
  THEME_META,
  themeMeta as generatedThemeMeta,
  type ThemeMeta,
} from './generated/themes-meta'
export type { ThemeId, ThemeMeta }

/**
 * Theme shipping tier.
 *
 * - `core` — ships with every Sail project (identity + neutral). Keep small.
 * - `catalog` — full aesthetic library (studio / marketing / opt-in).
 */
export type ThemeTier = 'core' | 'catalog'

export interface Theme {
  id: ThemeId
  label: string
  /**
   * Whether the **default mode** is a dark surface.
   * Mode switches use `applyMode` / `data-mode`, not twin theme ids.
   */
  dark: boolean
  /** Shipping tier — core defaults vs catalog extras. */
  tier: ThemeTier
  /** Preview swatches for picker UIs — [background, card, primary, accent]. */
  swatches: readonly [string, string, string, string]
  /** Mode names from DTCG `modes` (light/dark at minimum). */
  modes: readonly string[]
  defaultMode: string
}

/** Hand labels + swatches keyed by generated ThemeId. Meta (modes/dark) is generated. */
const THEME_PRESENTATION: Record<
  string,
  {
    label: string
    tier: ThemeTier
    swatches: readonly [string, string, string, string]
  }
> = {
  sail: {
    label: 'Sail',
    tier: 'core',
    swatches: [
      'oklch(0.985 0.008 250)',
      'oklch(1 0 0)',
      'oklch(0.58 0.18 245)',
      'oklch(0.62 0.14 195)',
    ],
  },
  ship: {
    label: 'SHIP',
    tier: 'catalog',
    swatches: [
      'oklch(0.99 0.001 106.423)',
      'oklch(1 0 0)',
      'oklch(0.67 0.16 58)',
      'oklch(0.61 0.23 303)',
    ],
  },
  neutral: {
    label: 'Neutral',
    tier: 'core',
    swatches: [
      'oklch(0.985 0.002 264)',
      'oklch(1 0 0)',
      'oklch(0.27 0.020 264)',
      'oklch(0.55 0.090 255)',
    ],
  },
  paper: {
    label: 'Paper',
    tier: 'catalog',
    swatches: [
      'oklch(0.97 0.014 80)',
      'oklch(0.99 0.010 80)',
      'oklch(0.55 0.14 35)',
      'oklch(0.48 0.09 55)',
    ],
  },
  halo: {
    label: 'Halo (Soft)',
    tier: 'catalog',
    swatches: [
      'oklch(0.92 0.008 230)',
      'oklch(0.92 0.008 230)',
      'oklch(0.62 0.14 50)',
      'oklch(0.55 0.05 230)',
    ],
  },
  console: {
    label: 'Console',
    tier: 'catalog',
    swatches: [
      'oklch(0.13 0.008 220)',
      'oklch(0.16 0.010 220)',
      'oklch(0.78 0.20 145)',
      'oklch(0.78 0.16 195)',
    ],
  },
  contrast: {
    label: 'High Contrast',
    tier: 'catalog',
    swatches: [
      'oklch(0.09 0 0)',
      'oklch(0.15 0 0)',
      'oklch(0.87 0.18 85)',
      'oklch(0.99 0 0)',
    ],
  },
  glass: {
    label: 'Glass',
    tier: 'catalog',
    swatches: [
      'oklch(0.95 0.04 230)',
      'oklch(0.99 0.005 240)',
      'oklch(0.62 0.16 50)',
      'oklch(0.62 0.14 280)',
    ],
  },
  brut: {
    label: 'Brutalist',
    tier: 'catalog',
    swatches: [
      'oklch(0.97 0 0)',
      'oklch(1 0 0)',
      'oklch(0.14 0 0)',
      'oklch(0.62 0.24 28)',
    ],
  },
  '8bit': {
    label: '8-Bit',
    tier: 'catalog',
    swatches: [
      'oklch(0.18 0.05 265)',
      'oklch(0.24 0.06 265)',
      'oklch(0.72 0.21 25)',
      'oklch(0.82 0.20 95)',
    ],
  },
  synthwave: {
    label: 'Synthwave',
    tier: 'catalog',
    swatches: [
      'oklch(0.16 0.06 285)',
      'oklch(0.22 0.08 285)',
      'oklch(0.65 0.28 340)',
      'oklch(0.75 0.22 190)',
    ],
  },
}

function buildThemes(): Theme[] {
  return THEME_META.map((meta) => {
    const pres = THEME_PRESENTATION[meta.id] ?? {
      label: meta.id
        .split('-')
        .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
        .join(' '),
      tier: 'catalog' as ThemeTier,
      swatches: [
        'oklch(0.95 0 0)',
        'oklch(1 0 0)',
        'oklch(0.5 0.1 250)',
        'oklch(0.55 0.1 280)',
      ] as const,
    }
    const modes =
      meta.modes.length > 0 ? meta.modes : (['light', 'dark'] as const)
    return {
      id: meta.id as ThemeId,
      label: pres.label,
      dark: meta.dark,
      tier: pres.tier,
      swatches: pres.swatches,
      modes: [...modes],
      defaultMode: meta.defaultMode ?? 'light',
    }
  })
}

/** Themes that ship into a default project stamp. Keep this set tight. */
export const CORE_THEME_IDS = ['sail', 'neutral'] as const satisfies readonly ThemeId[]

export const THEMES: Theme[] = buildThemes()

const THEME_BY_ID: Record<string, Theme> = Object.fromEntries(
  THEMES.map((t) => [t.id, t]),
)

/** Themes that ship with a default project (tier: core). */
export function coreThemes(): Theme[] {
  return THEMES.filter((t) => t.tier === 'core')
}

/** Catalog aesthetics — available in studio / marketing, not default stamps. */
export function catalogThemes(): Theme[] {
  return THEMES.filter((t) => t.tier === 'catalog')
}

export function isCoreTheme(id: ThemeId): boolean {
  return THEME_BY_ID[id]?.tier === 'core'
}

/** Modes-in-file primary default — Sail brand (not monorepo ship). */
export const DEFAULT_THEME: ThemeId = 'sail' as ThemeId
export const THEME_STORAGE_KEY = 'ship-site-theme'
/** Persisted mode key. */
export const THEME_MODE_STORAGE_KEY = 'ship-site-theme-mode'
export const DEFAULT_MODE = 'light'

export const isValidTheme = (value: unknown): value is ThemeId =>
  typeof value === 'string' && value in THEME_BY_ID

/** True when the theme's default mode is dark (not "currently in dark mode"). */
export const isDarkTheme = (id: ThemeId): boolean =>
  THEME_BY_ID[id]?.dark ?? false

/**
 * Document paint modes only. `"system"` is a **preference** that resolves
 * before paint — never a `data-mode` value.
 */
export type PaintMode = 'light' | 'dark'

/**
 * User mode preference stored in `THEME_MODE_STORAGE_KEY`.
 * Preference may be `system`; paint is always light|dark.
 */
export type ModePreference = 'light' | 'dark' | 'system'

/**
 * Map a mode preference (including `"system"`) to a document paint mode.
 * Aligns with studio `documentModeString`: **never** returns `"system"`.
 */
export function paintMode(mode: string | null | undefined): PaintMode {
  if (mode === 'dark') return 'dark'
  if (mode === 'light') return 'light'
  if (mode === 'system') {
    if (typeof window !== 'undefined' && typeof window.matchMedia === 'function') {
      try {
        return window.matchMedia('(prefers-color-scheme: dark)').matches
          ? 'dark'
          : 'light'
      } catch {
        /* matchMedia can throw in odd envs */
      }
    }
    return 'light'
  }
  // Unknown / empty → light (safe document default).
  return 'light'
}

/**
 * Read the persisted mode **preference** (`light` | `dark` | `system`).
 * Invalid / missing values normalize to `light` (or a paint mode when the
 * stored string resolves via `paintMode` to light|dark but is not a pref).
 */
export function readModePreference(): ModePreference {
  const stored = readStoredMode()
  if (stored === 'light' || stored === 'dark' || stored === 'system') return stored
  if (stored) {
    // Legacy / unknown paint strings → treat as explicit light|dark preference.
    return paintMode(stored)
  }
  return 'light'
}

/**
 * Subscribe to OS `prefers-color-scheme` changes.
 *
 * Invokes `cb` with the resolved **paint** mode only while the stored
 * preference is `system` (explicit light/dark preferences ignore OS flips).
 * Returns an unsubscribe function. No-op when `matchMedia` is unavailable.
 *
 * Callers that want to repaint should typically: `applyMode('system')` or
 * `paintMode('system')` + set `dataset.mode` without clearing preference.
 */
export function subscribeSystemPreference(
  cb: (paint: PaintMode) => void,
): () => void {
  if (
    typeof window === 'undefined' ||
    typeof window.matchMedia !== 'function'
  ) {
    return () => {}
  }
  let mql: MediaQueryList
  try {
    mql = window.matchMedia('(prefers-color-scheme: dark)')
  } catch {
    return () => {}
  }
  const handler = (): void => {
    if (readModePreference() !== 'system') return
    cb(paintMode('system'))
  }
  // Safari < 14: addListener; modern: addEventListener
  if (typeof mql.addEventListener === 'function') {
    mql.addEventListener('change', handler)
    return () => mql.removeEventListener('change', handler)
  }
  mql.addListener(handler)
  return () => mql.removeListener(handler)
}

/**
 * Apply a theme (+ optional mode) to the given root element (or
 * documentElement by default).
 *
 * Runtime model: **only** `data-theme` + `data-mode`. No `.dark` class.
 * `data-mode` is always `light` | `dark` — never `"system"`.
 * Returns the applied ThemeId.
 */
export function applyTheme(
  id: unknown,
  root: HTMLElement = document.documentElement,
  mode?: string | null,
): ThemeId {
  const next: ThemeId = isValidTheme(id) ? id : DEFAULT_THEME
  const resolvedMode = paintMode(resolveMode(next, mode))
  root.dataset.theme = next
  root.dataset.mode = resolvedMode
  // Explicitly clear legacy .dark if present (no dual mechanism).
  root.classList.remove('dark')
  const dark = resolvedMode === 'dark'
  if (typeof document !== 'undefined' && typeof CustomEvent !== 'undefined') {
    document.dispatchEvent(
      new CustomEvent('ship:theme:change', {
        detail: { id: next, dark, mode: resolvedMode },
      }),
    )
  }
  return next
}

/**
 * Modes declared for a theme (from generated meta). Empty ⇒ treat as light|dark
 * for legacy; single-mode themes return one entry.
 */
export function modesForTheme(id: string): readonly string[] {
  const meta = lookupThemeMeta(id)
  const modes = meta?.modes?.length
    ? meta.modes
    : THEME_BY_ID[id]?.modes?.length
      ? THEME_BY_ID[id]!.modes
      : (['light', 'dark'] as const)
  return modes
}

/** True when the theme declares the paint mode. */
export function themeSupportsMode(id: string, mode: string): boolean {
  const paint = mode === 'system' ? 'system' : mode
  if (paint === 'system') {
    // System preference is allowed only when the theme supports both paint modes
    // that system might resolve to — require both light and dark.
    const m = modesForTheme(id)
    return m.includes('light') && m.includes('dark')
  }
  return modesForTheme(id).includes(paint)
}

/** Resolve mode: explicit arg → stored → themes-meta default → DEFAULT_MODE.
 * Always clamps to a mode the theme declares (single-mode lock).
 */
function resolveMode(id: ThemeId, mode?: string | null): string {
  const modes = modesForTheme(id)
  const meta = lookupThemeMeta(id)
  const fallback =
    (meta?.defaultMode && modes.includes(meta.defaultMode)
      ? meta.defaultMode
      : null) ??
    THEME_BY_ID[id]?.defaultMode ??
    (modes.includes('light') ? 'light' : modes[0]) ??
    DEFAULT_MODE

  const clamp = (raw: string): string => {
    if (raw === 'system') {
      // Only honor system when both paint modes exist; else fall back.
      if (modes.includes('light') && modes.includes('dark')) return 'system'
      return fallback
    }
    if (modes.includes(raw)) return raw
    return fallback
  }

  if (typeof mode === 'string' && mode.length > 0) return clamp(mode)
  const stored = readStoredMode()
  // Stored "system" is a preference, not a paint key — resolve via paintMode later.
  if (stored && stored !== 'system' && modes.includes(stored)) return stored
  if (stored === 'system') return clamp('system')
  return fallback
}

/** Meta lookup from generated themes-meta. */
function lookupThemeMeta(
  id: string,
): { modes: readonly string[]; defaultMode: string | null } | undefined {
  return generatedThemeMeta(id)
}

export function readStoredMode(): string | null {
  if (typeof localStorage === 'undefined') return null
  try {
    return localStorage.getItem(THEME_MODE_STORAGE_KEY)
  } catch {
    return null
  }
}

/**
 * Persist a mode **preference** (`light` | `dark` | `system`).
 * Does not paint — pair with `applyMode` / `applyTheme` for document attrs.
 */
export function persistMode(mode: string): void {
  try {
    localStorage.setItem(THEME_MODE_STORAGE_KEY, mode)
  } catch {
    /* opaque origin */
  }
}

/**
 * Apply mode for the current theme (sets data-mode only).
 * Always paints `light` | `dark` — `"system"` resolves via prefers-color-scheme
 * and is never written to `data-mode` (studio / site parity).
 *
 * **Preference vs paint:** when `mode` is `"system"`, stores preference
 * `"system"` and paints the resolved light|dark. When `mode` is light|dark,
 * stores that preference and paints it (clamped by the active theme's modes).
 */
export function applyMode(
  mode: string,
  root: HTMLElement = document.documentElement,
): string {
  const id = (root.dataset.theme as ThemeId) || DEFAULT_THEME
  const preference: ModePreference =
    mode === 'system' || mode === 'light' || mode === 'dark'
      ? mode
      : paintMode(mode)

  // Pass preference through so resolveMode can honor system + single-mode clamp.
  applyTheme(id, root, preference)
  const paint = (root.dataset.mode as PaintMode) || paintMode(preference)

  // Persist preference intent — never collapse system → resolved paint.
  // Single-mode themes still clamp paint; preference may remain system
  // so returning to a dual-mode theme restores auto (U2.1b).
  persistMode(preference)
  return paint
}

/**
 * Read the persisted theme from localStorage, fall back to
 * DEFAULT_THEME. Safe to call server-side (returns default).
 */
export function readStoredTheme(): ThemeId {
  if (typeof localStorage === 'undefined') return DEFAULT_THEME
  try {
    const raw = localStorage.getItem(THEME_STORAGE_KEY)
    return isValidTheme(raw) ? raw : DEFAULT_THEME
  } catch {
    return DEFAULT_THEME
  }
}

/**
 * Theme persistence transport — the host's authoritative settings store.
 * sail/tokens is a framework package and names no daemon URL; the host
 * (shell) wires this to its settings service. Omitted ⇒ persistence is
 * localStorage-only.
 */
export interface ThemeTransport {
  /** Write-through the chosen theme. Fire-and-forget; failures ignored. */
  save: (id: ThemeId) => void | Promise<void>
  /** Read the authoritative theme (null if absent/unreachable). */
  load: () => Promise<string | null>
}

let themeTransport: ThemeTransport | null = null

/** Register (or clear) the theme transport. Call once at host startup. */
export function setThemeTransport(t: ThemeTransport | null): void {
  themeTransport = t
}

/**
 * Persist the current theme. localStorage is the tier-4 cache the
 * pre-paint boot script reads (must be synchronous); the host transport
 * (when registered) is the tier-2 authoritative store.
 */
export function persistTheme(id: ThemeId): void {
  try {
    localStorage.setItem(THEME_STORAGE_KEY, id)
  } catch {}
  void themeTransport?.save(id)
}

/**
 * Read the authoritative theme from the host transport and update the
 * local cache to match.
 */
export async function syncThemeFromSettings(): Promise<ThemeId | null> {
  if (!themeTransport) return null
  try {
    const raw = await themeTransport.load()
    if (!isValidTheme(raw)) return null
    try {
      localStorage.setItem(THEME_STORAGE_KEY, raw)
    } catch {}
    return raw
  } catch {
    return null
  }
}
