import {
  applyTheme,
  applyMode,
  persistTheme,
  persistMode,
  THEMES,
  isValidTheme,
  paintMode,
  readModePreference,
  subscribeSystemPreference,
  themeSupportsMode,
  type Theme,
  type ThemeId,
  type PaintMode,
  type ModePreference,
} from '@ship/tokens/themes'
import type { Action } from '../action'
import { invokeCallback } from '../callback'

/**
 * `use:themeSelect` — product theme picker:
 *
 * Product grid uses the **effective emit set** (not always full `THEMES`):
 *   effective = project themes/*  ∪  allowlist(theme.enabled)
 * Grid rows = effective ∩ `@ship/tokens` `THEMES` presentation catalog
 * (project-only ids without presentation metadata are stamped server-side but
 * omitted from the product grid until they have THEMES entries).
 *
 * Source order (first match wins):
 * 1. `setEffectiveThemeIds` test/host override
 * 2. `sail-effective-themes` meta (deck / generate stamp — preferred first paint)
 * 3. Per-action ops `SailTheme/EffectiveIds` session result
 * 4. Full `THEMES` when unrestricted (no stamp, no project, marketing default)
 *
 * When `sail-project` is present but effective meta is not, the grid is
 * **fail-closed** (empty) until ops resolves; ops failure then fail-opens to
 * full `THEMES`.
 *
 * Full catalog UI:
 * - Mode segment is a **catalog filter tab only** (Light | Dark | System):
 *   - Light → themes that declare light
 *   - Dark → themes that declare dark
 *   - System → dual-mode themes only (can honor prefers-color-scheme)
 * - Tabs NEVER applyMode / applyTheme / persist mode or mutate document paint
 * - Live [theme=…] specimens paint in the filter specimen mode
 * - Selecting a **theme row** applies theme + mode intent from the active filter
 * - Selecting a theme does **not** auto-close the popover
 *
 * modeOnly:
 * - Icon trigger cycles light → dark → system via applyMode (mode switcher surface)
 *
 * Never paints data-mode=system. U2.1c: no-op under studioPreview.
 */

let uid = 0

const PAINT_MODES = new Set<PaintMode>(['light', 'dark'])

/** Theme id shape aligned with sail-compiler `validate_theme_id` (no path chars). */
/** Catalog theme ids — digit-leading allowed (e.g. `8bit`). Path-like rejected elsewhere. */
const SAFE_THEME_ID = /^[a-z0-9][a-z0-9-]*$/

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

function isSafeThemeIdToken(id: string): boolean {
  return SAFE_THEME_ID.test(id) && !id.includes('..')
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
  return THEMES.filter((t) => allow.has(t.id))
}

/** Best-effort ops fetch when meta is absent (DEV host with SailTheme). */
async function tryFetchEffectiveIdsFromOps(): Promise<string[] | null> {
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
    const ids = json.all
      .map((x) => String(x))
      .filter((id) => isSafeThemeIdToken(id))
    return ids
  } catch {
    return null
  }
}

const isModeOnlyHost = (root: HTMLElement): boolean => {
  if (root.hasAttribute('modeOnly') || root.hasAttribute('modeonly')) return true
  if ((root as HTMLElement & { modeOnly?: boolean }).modeOnly === true) return true
  const v = root.getAttribute('mode-only')
  if (v === null) return false
  // bare `mode-only`, `mode-only=""`, `mode-only="true"` all mean on
  return v !== 'false' && v !== '0'
}

const isInlineHost = (root: HTMLElement): boolean => {
  if (!root.hasAttribute('inline')) return false
  const v = root.getAttribute('inline')
  return v !== 'false' && v !== '0'
}

const isModePreference = (raw: string): raw is ModePreference =>
  raw === 'light' || raw === 'dark' || raw === 'system'

export const themeSelect: Action<void> = (node) => {
  const root = node as HTMLElement

  const fullTrigger = root.querySelector<HTMLButtonElement>(
    '[data-picker-trigger="full"], button[popovertarget]:not([data-mode-only-trigger])',
  )
  const modeTrigger = root.querySelector<HTMLButtonElement>(
    '[data-picker-trigger="mode"], [data-mode-only-trigger]',
  )
  const legacyTrigger =
    fullTrigger ??
    modeTrigger ??
    root.querySelector<HTMLButtonElement>(
      'button[popovertarget], button[type="button"]:not([data-mode])',
    )

  const panel = root.querySelector<HTMLElement>(
    '[data-theme-listbox], [popover], [role="listbox"]',
  )
  // Closed multi-part anatomy: stamp listbox on the panel (not free author role=).
  if (panel && !panel.hasAttribute('role')) panel.setAttribute('role', 'listbox')

  let modeOnly = isModeOnlyHost(root)
  let inline = isInlineHost(root)

  const themeGrid = root.querySelector<HTMLElement>('[data-theme-grid]')
  const modeBar = root.querySelector<HTMLElement>('[data-mode-bar]')
  // Filter bar = tablist; mode buttons = tabs (stamped here, not in .sail markup).
  if (modeBar) {
    if (!modeBar.hasAttribute('role')) modeBar.setAttribute('role', 'tablist')
    for (const tab of modeBar.querySelectorAll<HTMLElement>('button[data-mode]')) {
      if (!tab.hasAttribute('role')) tab.setAttribute('role', 'tab')
    }
  }
  const modeLockBanner = root.querySelector<HTMLElement>('[data-mode-lock-banner]')
  const selectedLabel = root.querySelector<HTMLElement>('[data-selected-label]')
  const selectedTier = root.querySelector<HTMLElement>('[data-selected-tier]')
  const catalogLabel = root.querySelector<HTMLElement>('[data-catalog-label]')
  const pickerHeader = root.querySelector<HTMLElement>('[data-picker-header]')

  const valueOf = (el: Element): string => el.getAttribute('value') ?? ''
  const modeOf = (el: Element): string => el.getAttribute('data-mode') ?? ''

  const options = (): HTMLElement[] =>
    Array.from(root.querySelectorAll<HTMLElement>('[role="option"][value]'))

  /** Filter tabs only — scoped to [data-mode-bar], never mode-only icons. */
  const modeButtons = (): HTMLElement[] => {
    if (!modeBar) return []
    return Array.from(
      modeBar.querySelectorAll<HTMLElement>(
        'button[data-mode], [role="tab"][data-mode], [role="button"][data-mode]',
      ),
    )
  }

  const modesFor = (id: ThemeId): readonly PaintMode[] => {
    const meta = THEMES.find((t) => t.id === id)
    const raw = meta?.modes?.length ? meta.modes : (['light', 'dark'] as const)
    return raw.filter((m): m is PaintMode => PAINT_MODES.has(m as PaintMode))
  }

  const currentTheme = (): ThemeId => {
    const attr =
      root.getAttribute('value') || document.documentElement.getAttribute('theme')
    return isValidTheme(attr) ? attr : 'sail'
  }

  const currentPaint = (): PaintMode => {
    const fromDom = document.documentElement.dataset.mode
    if (fromDom === 'light' || fromDom === 'dark') return fromDom
    if (fromDom === 'system' || fromDom) {
      const healed = paintMode(fromDom)
      if (document.documentElement.dataset.mode !== healed) {
        document.documentElement.dataset.mode = healed
      }
      return healed
    }
    const meta = THEMES.find((t) => t.id === currentTheme())
    return paintMode(meta?.defaultMode ?? 'light')
  }

  // Catalog filter follows **live paint** so an unpersisted / config-dark boot
  // still shows dark-capable themes (incl. 8bit). Storage preference alone can
  // lag the DOM after external applyTheme.
  let catalogFilter: ModePreference = currentPaint()
  /** Last observed document paint — used to detect external theme/mode flips. */
  let lastPaint: PaintMode = currentPaint()

  /**
   * Per-action ops EffectiveIds session.
   * `undefined` = no ops restriction; `'pending'` = fail-closed; `string[]` = allowlist.
   */
  let sessionEffectiveIds: EffectiveSessionIds = undefined

  /** Detached option skeleton so empty→non-empty grid rebuilds work. */
  let optionTemplate: HTMLElement | null = null

  let trigger: HTMLButtonElement | null = null
  const applyModeChrome = (): void => {
  if (modeOnly) {
    if (modeTrigger && fullTrigger && modeTrigger !== fullTrigger) {
      fullTrigger.setAttribute('hidden', '')
      fullTrigger.style.display = 'none'
      fullTrigger.removeAttribute('popovertarget')
      fullTrigger.setAttribute('aria-hidden', 'true')
      fullTrigger.tabIndex = -1
      modeTrigger.removeAttribute('hidden')
      modeTrigger.removeAttribute('aria-hidden')
      modeTrigger.style.display = ''
      modeTrigger.removeAttribute('popovertarget')
      modeTrigger.tabIndex = 0
      trigger = modeTrigger
    } else {
      const t = modeTrigger ?? fullTrigger ?? legacyTrigger
      t?.removeAttribute('popovertarget')
      t?.removeAttribute('hidden')
      t?.removeAttribute('aria-hidden')
      if (t) {
        t.style.display = ''
        t.tabIndex = 0
      }
      trigger = t
    }
    if (panel) {
      panel.hidden = true
      panel.setAttribute('inert', '')
    }
    if (themeGrid) themeGrid.hidden = true
    if (pickerHeader) pickerHeader.hidden = true
    if (modeBar) modeBar.hidden = true
    if (catalogLabel) catalogLabel.hidden = true
    root.setAttribute('data-mode-only', '1')
  } else {
    // Site chrome: only the full catalog trigger. Mode-only is for demos.
    if (modeTrigger) {
      modeTrigger.setAttribute('hidden', '')
      modeTrigger.setAttribute('aria-hidden', 'true')
      modeTrigger.style.display = 'none'
      modeTrigger.tabIndex = -1
    }
    fullTrigger?.removeAttribute('hidden')
    fullTrigger?.style.removeProperty('display')
    fullTrigger?.removeAttribute('aria-hidden')
    if (panel) {
      const id = `sail-theme-panel-${++uid}`
      panel.id = id
      panel.removeAttribute('inert')
      panel.hidden = false
      ;(fullTrigger ?? legacyTrigger)?.setAttribute('popovertarget', id)
    }
    trigger = fullTrigger ?? legacyTrigger
    root.removeAttribute('data-mode-only')
  }
  }

  const applyInlineChrome = (): void => {
    if (modeOnly) {
      root.removeAttribute('data-inline')
      return
    }
    if (inline) {
      fullTrigger?.setAttribute('hidden', '')
      if (fullTrigger) {
        fullTrigger.style.display = 'none'
        fullTrigger.setAttribute('aria-hidden', 'true')
      }
      if (panel) {
        panel.removeAttribute('popover')
        panel.hidden = false
        panel.removeAttribute('inert')
      }
      root.setAttribute('data-inline', '1')
    } else {
      fullTrigger?.removeAttribute('hidden')
      fullTrigger?.removeAttribute('aria-hidden')
      fullTrigger?.style.removeProperty('display')
      if (panel && !panel.hasAttribute('popover')) {
        panel.setAttribute('popover', '')
        const id = panel.id || `sail-theme-panel-${++uid}`
        panel.id = id
        ;(fullTrigger ?? legacyTrigger)?.setAttribute('popovertarget', id)
      }
      root.removeAttribute('data-inline')
    }
  }

  applyModeChrome()
  applyInlineChrome()

  const currentPreference = (): ModePreference => readModePreference()

  /** Does this theme belong in the catalog under the active filter tab? */
  const themeMatchesFilter = (id: ThemeId, filter: ModePreference): boolean => {
    const modes = modesFor(id)
    if (filter === 'system') {
      return modes.includes('light') && modes.includes('dark')
    }
    return modes.includes(filter)
  }

  /**
   * Preview paint for a row under the current catalog filter.
   * Light → light; Dark → dark; System → current document paint.
   */
  const swatchModeFor = (
    id: ThemeId,
    filter: ModePreference,
    chromePaint: PaintMode,
  ): PaintMode => {
    const modes = modesFor(id)
    if (filter === 'system') {
      if (modes.includes(chromePaint)) return chromePaint
      return modes[0] ?? 'light'
    }
    if (modes.includes(filter)) return filter
    return modes[0] ?? 'light'
  }

  const preferenceLabel = (pref: ModePreference, paint: PaintMode): string => {
    if (pref === 'system') return `System (${paint})`
    return pref === 'dark' ? 'Dark' : 'Light'
  }

  const catalogTitle = (filter: ModePreference): string => {
    if (filter === 'system') return 'Dual-mode themes'
    if (filter === 'dark') return 'Dark themes'
    return 'Light themes'
  }

  /** Mode intent to apply when selecting a theme under the active filter. */
  const modeIntentForSelect = (themeId: ThemeId, filter: ModePreference): ModePreference => {
    const modes = modesFor(themeId)
    if (filter === 'system') {
      if (modes.includes('light') && modes.includes('dark')) return 'system'
      return modes[0] ?? 'light'
    }
    if (modes.includes(filter)) return filter
    return modes[0] ?? 'light'
  }

  const ensureCatalog = (): void => {
    if (!themeGrid || modeOnly) return

    // Seed markup is a single `<button value="sail">` WITHOUT role=option —
    // configureOption stamps role after clone. Scan grid values, not role.
    const byId = new Map<string, HTMLElement>()
    for (const o of Array.from(
      themeGrid.querySelectorAll<HTMLElement>('button[value], [role="option"][value]'),
    )) {
      const v = valueOf(o)
      if (v) byId.set(v, o)
    }
    // Capture a detached template before leftovers are removed so an empty
    // product set can later rebuild rows (override/meta/ops transitions).
    if (!optionTemplate) {
      const live = byId.values().next().value as HTMLElement | undefined
      if (live) optionTemplate = live.cloneNode(true) as HTMLElement
    }
    if (!optionTemplate) return

    // Product grid = effective emit set ∩ THEMES presentation catalog.
    for (const theme of effectiveProductThemes(sessionEffectiveIds)) {
      let opt = byId.get(theme.id)
      if (!opt) {
        opt = optionTemplate.cloneNode(true) as HTMLElement
        themeGrid.appendChild(opt)
      } else {
        byId.delete(theme.id)
      }
      configureOption(opt, theme)
    }

    for (const leftover of byId.values()) leftover.remove()
  }

  const configureOption = (opt: HTMLElement, theme: Theme): void => {
    opt.setAttribute('role', 'option')
    opt.setAttribute('type', 'button')
    opt.setAttribute('value', theme.id)
    opt.title = theme.label

    // Name once — inside the live specimen (theme heading face).
    const swatchTitle = opt.querySelector<HTMLElement>('[data-swatch-title]')
    if (swatchTitle) swatchTitle.textContent = theme.label
    const optionLabel = opt.querySelector<HTMLElement>('[data-option-label]')
    if (optionLabel) optionLabel.textContent = theme.label

    // Live specimen: themes.css scopes tokens via theme + data-mode.
    // Always stamp data-mode here — a missing mode lets :root[data-mode=dark]
    // descendant rules paint light-filter cards with the page's dark ink.
    const preview = opt.querySelector<HTMLElement>('[theme]')
    if (preview) {
      preview.setAttribute('theme', theme.id)
      preview.setAttribute('data-mode', 'light')
      preview.removeAttribute('hidden')
    }

    // Optional static chip row fallback (if markup still has data-swatch-row).
    const chipRow = opt.querySelector<HTMLElement>('[data-swatch-row]')
    if (chipRow && theme.swatches?.length) {
      // Prefer chromatic pair first: primary, accent, then surfaces.
      const ordered = [
        theme.swatches[2], // primary
        theme.swatches[3], // accent
        theme.swatches[0], // background
        theme.swatches[1], // card
      ]
      ordered.forEach((color, i) => {
        const chip = chipRow.children[i] as HTMLElement | undefined
        if (chip && color) chip.style.backgroundColor = color
      })
    }

    const modes = modesFor(theme.id)
    if (modes.length === 1) {
      opt.setAttribute('data-mode-lock', modes[0]!)
    } else {
      opt.removeAttribute('data-mode-lock')
    }
    opt.removeAttribute('data-mode-badge')
  }

  const syncModeOnlyIcons = (pref: ModePreference): void => {
    if (!modeTrigger) return
    for (const icon of modeTrigger.querySelectorAll<HTMLElement>('[data-mode-icon]')) {
      const kind = icon.getAttribute('data-mode-icon')
      icon.hidden = kind !== pref
    }
  }

  const sync = (): void => {
    ensureCatalog()

    const cur = currentTheme()
    const paint = currentPaint()
    // External theme/mode change (other writers): if the filter was still
    // tracking the previous paint, follow the new live paint so the dark list
    // is not empty when the document is dark.
    if (paint !== lastPaint && catalogFilter === lastPaint) {
      catalogFilter = paint
    }
    lastPaint = paint
    const pref = currentPreference()
    const themeMeta = THEMES.find((t) => t.id === cur)
    const name = themeMeta?.label ?? cur

    if (selectedLabel) selectedLabel.textContent = name
    if (selectedTier) {
      const tier = themeMeta?.tier
      selectedTier.textContent =
        tier === 'core' ? 'Core' : tier === 'catalog' ? 'Catalog' : ''
    }

    if (catalogLabel && !modeOnly) {
      catalogLabel.hidden = false
      catalogLabel.textContent = catalogTitle(catalogFilter)
    }

    // Filter tabs always fully visible in the full picker.
    if (modeBar && !modeOnly) {
      modeBar.hidden = false
      modeBar.setAttribute('aria-label', 'Filter themes by mode')
      modeBar.removeAttribute('data-mode-badge')
    }
    if (modeLockBanner) modeLockBanner.hidden = true

    if (trigger) {
      if (modeOnly) {
        const label = preferenceLabel(pref, paint)
        trigger.setAttribute('aria-label', `Color mode: ${label}`)
        trigger.title = `Color mode: ${label} — click to cycle`
        trigger.setAttribute('data-mode-preference', pref)
        trigger.setAttribute('data-mode-paint', paint)
        syncModeOnlyIcons(pref)
      } else {
        trigger.setAttribute(
          'aria-label',
          `Theme: ${name} (${preferenceLabel(pref, paint)})`,
        )
        trigger.title = `Theme: ${name} · ${preferenceLabel(pref, paint)}`
      }
    }

    for (const o of options()) {
      const id = valueOf(o)
      const on = id === cur
      o.setAttribute('aria-selected', on ? 'true' : 'false')
      o.setAttribute('tabindex', on ? '0' : '-1')
      const check = o.querySelector<HTMLElement>('[data-selected-check]')
      if (check) check.hidden = !on

      // Catalog filter: hide themes that don't support this tab.
      // Force display:none — flex/layout classes can ignore bare [hidden] alone.
      if (!modeOnly && isValidTheme(id)) {
        const show = themeMatchesFilter(id, catalogFilter)
        o.hidden = !show
        o.style.display = show ? '' : 'none'
        if (show) {
          o.removeAttribute('aria-hidden')
          o.removeAttribute('data-filtered-out')
        } else {
          o.setAttribute('aria-hidden', 'true')
          o.setAttribute('data-filtered-out', '')
        }
      } else {
        o.hidden = false
        o.style.display = ''
        o.removeAttribute('data-filtered-out')
      }

      const preview = o.querySelector<HTMLElement>('[theme]')
      if (preview && isValidTheme(id)) {
        preview.setAttribute('theme', id)
        preview.setAttribute('data-mode', swatchModeFor(id, catalogFilter, paint))
      } else if (preview) {
        preview.setAttribute('data-mode', paint)
      }
    }

    // Filter tabs: aria-selected for sx when({selected}) PLUS inline chrome
    // so the active tab is always legible (sx alone has failed across themes).
    // Never paint the document from a tab click.
    for (const b of modeButtons()) {
      const m = modeOf(b)
      if (modeOnly) {
        b.hidden = true
        b.style.display = 'none'
        continue
      }
      if (m === 'system' || PAINT_MODES.has(m as PaintMode)) {
        b.hidden = false
        b.style.display = ''
        const on = catalogFilter === m
        b.setAttribute('aria-pressed', on ? 'true' : 'false')
        b.setAttribute('aria-selected', on ? 'true' : 'false')
        b.toggleAttribute('data-active', on)
        b.removeAttribute('selected')
        // Inline selected chrome (popover tokens — readable on every host theme).
        if (on) {
          b.style.color = 'var(--theme-picker-foreground)'
          b.style.backgroundColor = 'var(--theme-picker-option-hover-bg)'
          b.style.borderBottomColor = 'var(--primary)'
          b.style.fontWeight = 'var(--font-weight-semibold)'
        } else {
          b.style.color = ''
          b.style.backgroundColor = ''
          b.style.borderBottomColor = ''
          b.style.fontWeight = ''
        }
        continue
      }
      b.hidden = true
      b.style.display = 'none'
    }

    root.setAttribute('data-mode-preference', pref)
    root.setAttribute('data-mode-paint', paint)
    root.setAttribute('data-theme-active', cur)
    root.setAttribute('data-catalog-filter', catalogFilter)
  }

  const studioPreviewActive = (): boolean =>
    typeof document !== 'undefined' &&
    document.documentElement.dataset.studioPreview === '1'

  const selectTheme = (v: string): void => {
    if (!isValidTheme(v)) return
    if (studioPreviewActive()) return
    // When the emit set is known (including ops-pending empty), only product
    // grid ids may be applied — blocks off-set picks during fail-closed wait.
    const emitIds = readEffectiveThemeIds(sessionEffectiveIds)
    if (emitIds !== null && !emitIds.includes(v)) return
    if (!effectiveProductThemes(sessionEffectiveIds).some((t) => t.id === v)) return
    root.setAttribute('value', v)
    const intent = modeIntentForSelect(v, catalogFilter)
    applyTheme(v, document.documentElement, intent)
    persistTheme(v)
    // Keep storage coherent with the filter intent used for this selection.
    persistMode(intent)
    sync()
    invokeCallback(root, 'onThemeChange', {
      theme: v,
      mode: paintMode(document.documentElement.dataset.mode),
      preference: readModePreference(),
    })
    // Keep the catalog open so users can compare themes (product request).
  }

  /**
   * Filter tab click — local catalog state only.
   * FORBIDDEN: applyMode, applyTheme, persistMode, document dataset mutation,
   * theme-change paint events.
   */
  const selectCatalogFilter = (raw: string): void => {
    if (!raw || !isModePreference(raw)) return
    if (catalogFilter === raw) {
      // Still re-sync so aria-selected / display stay correct after re-open.
      sync()
      return
    }
    catalogFilter = raw
    // User intentionally browsed another mode tab — pin lastPaint so sync
    // does not immediately snap the filter back to live paint.
    lastPaint = currentPaint()
    root.setAttribute('data-catalog-filter', catalogFilter)
    sync()
  }

  const cycleModeOnly = (): void => {
    if (studioPreviewActive()) return
    const theme = currentTheme()
    const allowed = modesFor(theme)
    const dual = themeSupportsMode(theme, 'system')
    const pref = currentPreference()
    let next: ModePreference
    if (!dual) {
      // Single-mode theme: stay on its only mode (no fake cycle).
      next = allowed[0] ?? 'light'
    } else if (pref === 'light') {
      next = 'dark'
    } else if (pref === 'dark') {
      next = 'system'
    } else {
      next = 'light'
    }
    const paint = applyMode(next)
    sync()
    invokeCallback(root, 'onThemeChange', {
      theme: currentTheme(),
      mode: paint,
      preference: readModePreference(),
    })
  }

  const optionFrom = (e: Event): HTMLElement | null => {
    const o =
      (e.target as Element | null)?.closest<HTMLElement>('[role="option"][value]') ?? null
    return o && root.contains(o) ? o : null
  }

  const modeFrom = (e: Event): HTMLElement | null => {
    // Only tabs inside the mode bar (filter tabs), not stray data-mode nodes.
    const m =
      (e.target as Element | null)?.closest<HTMLElement>(
        '[data-mode-bar] button[data-mode], [data-mode-bar] [role="tab"][data-mode], [data-mode-bar] [role="button"][data-mode]',
      ) ?? null
    if (!m || !root.contains(m) || m.matches('[role="option"]')) return null
    return m
  }

  const isTriggerClick = (e: Event): boolean => {
    if (!trigger) return false
    return e.target === trigger || trigger.contains(e.target as Node)
  }

  const onClick = (e: MouseEvent): void => {
    // mode-only: cycle on any click of the mode trigger (even if nested span)
    if (modeOnly) {
      if (isTriggerClick(e) || (modeTrigger && modeTrigger.contains(e.target as Node))) {
        e.preventDefault()
        e.stopPropagation()
        cycleModeOnly()
        return
      }
      return
    }

    const modeBtn = modeFrom(e)
    if (modeBtn) {
      e.preventDefault()
      selectCatalogFilter(modeOf(modeBtn))
      return
    }
    const o = optionFrom(e)
    if (o && !o.hidden) selectTheme(valueOf(o))
  }

  const onKeydown = (e: KeyboardEvent): void => {
    if (modeOnly) {
      if (
        (isTriggerClick(e) || (modeTrigger && modeTrigger.contains(e.target as Node))) &&
        (e.key === 'Enter' || e.key === ' ')
      ) {
        e.preventDefault()
        cycleModeOnly()
      }
      return
    }

    const modeBtn = modeFrom(e)
    if (modeBtn && (e.key === 'Enter' || e.key === ' ')) {
      e.preventDefault()
      selectCatalogFilter(modeOf(modeBtn))
      return
    }
    const o = optionFrom(e)
    if (!o || o.hidden) return
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      selectTheme(valueOf(o))
      return
    }
    const opts = options().filter((x) => !x.hidden)
    const i = opts.indexOf(o)
    let j = -1
    if (e.key === 'ArrowDown' || e.key === 'ArrowRight') j = (i + 1) % opts.length
    else if (e.key === 'ArrowUp' || e.key === 'ArrowLeft')
      j = (i - 1 + opts.length) % opts.length
    else if (e.key === 'Home') j = 0
    else if (e.key === 'End') j = opts.length - 1
    if (j >= 0) {
      e.preventDefault()
      opts[j]!.focus()
    }
  }

  const onToggle = (e: Event): void => {
    if ((e as ToggleEvent).newState !== 'open') return
    // Re-sync list + preview modes; do not re-apply theme.
    sync()
    const cur = currentTheme()
    const visible = options().filter((x) => !x.hidden)
    ;(
      visible.find((o) => valueOf(o) === cur) ??
      visible[0] ??
      modeButtons()[0]
    )?.focus()
  }

  const onThemeEvent = (): void => {
    sync()
  }

  const unsubSystem = subscribeSystemPreference((nextPaint) => {
    if (studioPreviewActive()) return
    const theme = currentTheme()
    if (!themeSupportsMode(theme, 'system')) return
    const rootEl = document.documentElement
    if (rootEl.dataset.mode !== nextPaint) {
      applyTheme(theme, rootEl, 'system')
      sync()
      invokeCallback(root, 'onThemeChange', {
        theme,
        mode: nextPaint,
        preference: 'system',
      })
    }
  })

  // Direct listener on mode trigger so mode-only can't miss nested glyph clicks
  // if delegation is blocked by stopPropagation elsewhere.
  const onModeTriggerClick = (e: Event): void => {
    if (!modeOnly) return
    e.preventDefault()
    e.stopPropagation()
    cycleModeOnly()
  }
  modeTrigger?.addEventListener('click', onModeTriggerClick)

  root.addEventListener('click', onClick)
  root.addEventListener('keydown', onKeydown)
  panel?.addEventListener('toggle', onToggle)
  document.addEventListener('ship:theme:change', onThemeEvent)

  // When effective meta is not stamped but sail-project is, resolve via ops
  // (same SailTheme/EffectiveIds as theme-session). Fail-closed until settled.
  let cancelled = false
  if (
    !modeOnly &&
    effectiveThemeIdsOverride === null &&
    !hasEffectiveThemesMeta() &&
    hasSailProjectMeta()
  ) {
    sessionEffectiveIds = 'pending'
    void tryFetchEffectiveIdsFromOps().then((ids) => {
      if (cancelled) return
      // Always settle session — even if a host override is active — so clearing
      // the override later is not stuck on permanent fail-closed `'pending'`.
      // Override / meta still win in readEffectiveThemeIds over session.
      if (hasEffectiveThemesMeta()) {
        sessionEffectiveIds = undefined
      } else if (ids === null) {
        // Ops unavailable — fail-open to full catalog after pending.
        sessionEffectiveIds = undefined
      } else {
        sessionEffectiveIds = ids
      }
      sync()
    })
  }

  sync()

  const hostObs = new MutationObserver(() => {
    const nextMode = isModeOnlyHost(root)
    const nextInline = isInlineHost(root)
    const chromeChanged = nextMode !== modeOnly || nextInline !== inline
    modeOnly = nextMode
    inline = nextInline
    if (chromeChanged) {
      applyModeChrome()
      applyInlineChrome()
    }
    const v = root.getAttribute('value')
    if (v && isValidTheme(v) && !studioPreviewActive()) {
      applyTheme(v, document.documentElement, modeIntentForSelect(v, catalogFilter))
    }
    sync()
  })
  hostObs.observe(root, {
    attributes: true,
    attributeFilter: ['value', 'inline', 'mode-only', 'modeOnly', 'modeonly'],
  })

  return {
    destroy: () => {
      cancelled = true
      hostObs.disconnect()
      root.removeEventListener('click', onClick)
      root.removeEventListener('keydown', onKeydown)
      panel?.removeEventListener('toggle', onToggle)
      modeTrigger?.removeEventListener('click', onModeTriggerClick)
      document.removeEventListener('ship:theme:change', onThemeEvent)
      unsubSystem()
    },
  }
}
