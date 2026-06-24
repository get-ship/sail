/**
 * Shared theme catalog for the SHIP app and marketing site.
 *
 * Every theme is a full aesthetic bundle — palette, surfaces, radii,
 * shadows, motion, typography, and interaction strength. The picker is
 * flat: no tiers, no split. Pick one and the app *feels* different,
 * not just looks different.
 *
 * Palette definitions live alongside this module in `./themes.css`
 * (per-theme overrides) and `./tokens.css` (canonical ship-light +
 * ship-dark base). Consumers only need this module to know the theme
 * ids, human labels, and whether each theme should carry the canonical
 * `.dark` class.
 */

export type ThemeId =
  | 'ship-light'
  | 'ship-dark'
  | 'paper'
  | 'halo'
  | 'console'
  | 'contrast'
  | 'glass'
  | 'neutral-light'
  | 'neutral-dark'

export interface Theme {
  id: ThemeId
  label: string
  /** Whether the theme needs the canonical `.dark` class applied so
   *  shadow/elevation tokens from ./tokens.css flip too. */
  dark: boolean
  /** Preview swatches for picker UIs — [background, card, primary, accent].
   *  Mirrors the palette the theme actually paints with so cards can
   *  show at a glance what the theme looks like. */
  swatches: readonly [string, string, string, string]
}

export const THEMES: Theme[] = [
  {
    id: 'ship-light', label: 'SHIP Light', dark: false,
    swatches: [
      'oklch(0.99 0.001 106.423)', // background
      'oklch(1 0 0)',              // card
      'oklch(0.67 0.16 58)',       // primary — SHIP amber
      'oklch(0.61 0.23 303)',      // accent — purple
    ],
  },
  {
    id: 'ship-dark', label: 'SHIP Dark', dark: true,
    swatches: [
      'oklch(0.18 0.01 250)',      // background
      'oklch(0.18 0.01 50)',       // card
      'oklch(0.77 0.16 70)',       // primary — warm amber
      'oklch(0.3 0.05 300)',       // accent — muted purple
    ],
  },
  {
    id: 'paper', label: 'Paper', dark: false,
    swatches: [
      'oklch(0.97 0.014 80)',      // warm cream
      'oklch(0.99 0.010 80)',      // card
      'oklch(0.55 0.14 35)',       // terracotta primary
      'oklch(0.48 0.09 55)',       // muted accent
    ],
  },
  {
    id: 'halo', label: 'Halo (Soft)', dark: false,
    swatches: [
      'oklch(0.92 0.008 230)',     // soft mid-grey-blue surface
      'oklch(0.92 0.008 230)',     // same — depth from shadow, not hue
      'oklch(0.62 0.14 50)',       // muted orange primary
      'oklch(0.55 0.05 230)',      // accent
    ],
  },
  {
    id: 'console', label: 'Console', dark: true,
    swatches: [
      'oklch(0.13 0.008 220)',     // near-black ground
      'oklch(0.16 0.010 220)',     // tinted card
      'oklch(0.78 0.20 145)',      // phosphor green
      'oklch(0.78 0.16 195)',      // teal accent
    ],
  },
  {
    id: 'contrast', label: 'High Contrast', dark: true,
    swatches: [
      'oklch(0.09 0 0)',           // near-black
      'oklch(0.15 0 0)',           // card
      'oklch(0.87 0.18 85)',       // amber primary = accent
      'oklch(0.99 0 0)',           // pure white fg
    ],
  },
  {
    id: 'glass', label: 'Glass', dark: false,
    swatches: [
      'oklch(0.95 0.04 230)',      // soft tinted page backdrop
      'oklch(0.99 0.005 240)',     // frosted card surface
      'oklch(0.62 0.16 50)',       // muted brand orange
      'oklch(0.62 0.14 280)',      // iridescent violet accent
    ],
  },
  {
    // Brand-free default for building on SUI without adopting any
    // product identity. Neutral slate surfaces, a deep-slate "ink"
    // primary, one restrained slate-blue accent. Carries zero brand.
    id: 'neutral-light', label: 'Neutral Light', dark: false,
    swatches: [
      'oklch(0.985 0.002 264)',    // cool off-white background
      'oklch(1 0 0)',              // white card
      'oklch(0.27 0.020 264)',     // deep-slate ink primary
      'oklch(0.55 0.090 255)',     // restrained slate-blue accent
    ],
  },
  {
    id: 'neutral-dark', label: 'Neutral Dark', dark: true,
    swatches: [
      'oklch(0.18 0.008 264)',     // deep cool-slate background
      'oklch(0.22 0.010 264)',     // lifted slate card
      'oklch(0.92 0.008 264)',     // near-white ink primary
      'oklch(0.70 0.100 255)',     // restrained slate-blue accent
    ],
  },
]

const THEME_BY_ID: Record<string, Theme> = Object.fromEntries(
  THEMES.map((t) => [t.id, t]),
)

export const DEFAULT_THEME: ThemeId = 'ship-light'
export const THEME_STORAGE_KEY = 'ship-site-theme'

export const isValidTheme = (value: unknown): value is ThemeId =>
  typeof value === 'string' && value in THEME_BY_ID

export const isDarkTheme = (id: ThemeId): boolean =>
  THEME_BY_ID[id]?.dark ?? false

/**
 * Apply a theme to the given root element (or documentElement by
 * default). Sets `data-theme` and toggles the canonical `.dark` class.
 * Returns the applied ThemeId.
 */
export function applyTheme(
  id: unknown,
  root: HTMLElement = document.documentElement,
): ThemeId {
  const next: ThemeId = isValidTheme(id) ? id : DEFAULT_THEME
  const dark = isDarkTheme(next)
  root.dataset.theme = next
  root.classList.toggle('dark', dark)
  // Always announce theme application so chrome (status bar, devtools,
  // etc.) can repaint deterministically without a MutationObserver.
  if (typeof document !== 'undefined' && typeof CustomEvent !== 'undefined') {
    document.dispatchEvent(
      new CustomEvent('ship:theme:change', { detail: { id: next, dark } }),
    )
  }
  return next
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
 * sui/tokens is a framework package and names no daemon URL; the host
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
 * (when registered) is the tier-2 authoritative store. Write-through is
 * fire-and-forget — if the host is unreachable the local cache is still
 * correct and `syncThemeFromSettings()` reconciles on next load.
 */
export function persistTheme(id: ThemeId): void {
  if (typeof localStorage === 'undefined') return
  try {
    localStorage.setItem(THEME_STORAGE_KEY, id)
  } catch {}
  // Tier-2 write-through via the host transport. Fire-and-forget.
  void themeTransport?.save(id)
}

/**
 * Read the authoritative theme from the host transport and update the
 * local cache to match. Call on shell mount so another device's change
 * propagates. No transport registered (or unreachable) ⇒ localStorage
 * stays as-is.
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
