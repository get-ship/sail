import { controller, signal, derived, effect, ref } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import {
  THEMES,
  applyTheme,
  persistTheme,
  readStoredTheme,
  isValidTheme,
  type ThemeId,
  type Theme,
} from '@ship/tokens/themes'
// `navigate` arrives via the SUI setup-time seam (see openCustomize below).
// `ViewScope<Scope>` does not currently expose it in its type, so the
// scope-factory arg carries an augmented type here. The runtime injects
// `navigate` into every scope (resolver-context `ctx.navigate`, with a
// no-op fallback) — this is the documented seam; the type gap is a known
// limitation, not a hidden cast.

import type { Scope } from './sui-theme-picker.sui'

// Group cards by personality. Mirrors apps/shell/src/main.ts mountThemePicker
// so the SUI port presents the catalog the same way the legacy shell does.
type Group = { label: string; ids: ReadonlyArray<ThemeId> }
const GROUPS: Group[] = [
  { label: 'Default', ids: ['ship-light', 'ship-dark'] },
  { label: 'Texture', ids: ['paper', 'halo'] },
  { label: 'Effects', ids: ['glass'] },
  { label: 'Special', ids: ['console', 'contrast'] },
]

const THEMES_BY_ID = new Map<ThemeId, Theme>(THEMES.map((t) => [t.id, t]))

interface ThemeGroup { label: string; themes: Theme[] }

type PopoverEl = HTMLElement & {
  showPopover?: () => void
  hidePopover?: () => void
}

// Inject view-tokens.css (the [data-theme-preview="X"] rules) into the
// picker's shadow root so previews apply to cards in shadow — document-level
// rules can't reach shadow children. The href is a ship-free seam: the
// `tokens-href` prop (default `/view-tokens.css`, host-overridable) names
// where the consuming app serves the preview stylesheet. SDK-free.
function ensureThemesCssInShadow(host: HTMLElement, href: string) {
  const root = host.shadowRoot
  if (!root || !href) return
  if (root.querySelector('link[data-sui-theme-preview-styles]')) return
  const link = document.createElement('link')
  link.rel = 'stylesheet'
  link.href = href
  link.setAttribute('data-sui-theme-preview-styles', '')
  root.appendChild(link)
}

export default controller({
  scope: (init: ViewScope<Scope>) => {
    // SUI setup-time seam: the resolver injects `navigate` into every scope
    // (with a no-op fallback when no host registered one). The public
    // `ViewScope<Scope>` type doesn't expose it, so we read it off `init`
    // once and capture in a local — no `as any`, just a precise seam type
    // that names the runtime contract.
    const { navigate } = init as ViewScope<Scope> & { navigate: (path: string) => void }
    const activeTheme = signal<ThemeId>(readStoredTheme())
    const panelOpen = signal(false)
    const groups = signal<ThemeGroup[]>(
      GROUPS.map((g) => ({
        label: g.label,
        themes: g.ids.map((id) => THEMES_BY_ID.get(id)).filter((t): t is Theme => !!t),
      })),
    )
    const dotColor = derived(() =>
      THEMES_BY_ID.get(activeTheme())?.swatches[2] ?? 'var(--primary, #3b82f6)',
    )
    // The shell shows the active theme's accent swatch on the trigger; the
    // marketing surface treats the trigger as text so it stays legible on the
    // translucent menubar (the amber swatch is low-contrast there).
    const triggerIconColor = derived(() =>
      init.surface() === 'marketing' ? 'currentColor' : dotColor(),
    )
    // The Customize footer routes to /theme-studio, which only exists in the
    // shell. Hide it on the static marketing site.
    const showCustomize = derived(() => init.surface() !== 'marketing')
    const triggerLabel = derived(() =>
      THEMES_BY_ID.get(activeTheme())?.label ?? 'Theme',
    )

    // Mount populates these so scope-side handlers can dispatch on the
    // host and drive the native popover. Templates never bind to this.
    // `tokensHref` carries the preview-stylesheet seam (the `tokens-href`
    // prop) through to mount's shadow injection. `scope` carries the
    // isolation seam (the `scope` prop) — a non-empty value routes every
    // theme write to the host element subtree only, never documentElement.
    const internals: {
      host: HTMLElement | null
      panel: PopoverEl | null
      tokensHref: () => string
      scope: () => string
    } = {
      host: null,
      panel: null,
      tokensHref: init.tokensHref,
      scope: init.scope,
    }

    function togglePanel(): void {
      const panel = internals.panel
      if (!panel) return
      if (panelOpen()) panel.hidePopover?.()
      else panel.showPopover?.()
    }

    function selectTheme(id: ThemeId): void {
      const next = applyTheme(id)
      const scoped = internals.scope().length > 0
      const host = internals.host
      // `scope` isolation: a non-empty scope routes the write to the host
      // element's subtree only (its `data-theme` + `.dark` class) and skips
      // the document-level write `applyTheme` performs. Persistence is
      // intentionally SKIPPED in scoped mode — a preview's transient
      // selection must never poison the global theme.
      if (scoped && host) {
        const dark = THEMES_BY_ID.get(next)?.dark ?? false
        host.setAttribute('data-theme', next)
        host.classList.toggle('dark', dark)
      }
      if (init.persist() && !scoped) persistTheme(next)
      activeTheme.set(next)
      if (host) {
        host.dispatchEvent(new CustomEvent('sui-theme-change', {
          detail: { id: next, dark: THEMES_BY_ID.get(next)?.dark ?? false },
          bubbles: true,
          composed: true,
        }))
      }
      internals.panel?.hidePopover?.()
    }

    const triggerBtn = ref<HTMLButtonElement>(null)
    const panelEl = ref<HTMLDivElement>(null)

    function openCustomize(): void {
      internals.panel?.hidePopover?.()
      navigate('/theme-studio')
    }

    return {
      activeTheme,
      panelOpen,
      groups,
      dotColor,
      triggerIconColor,
      showCustomize,
      triggerLabel,
      togglePanel,
      selectTheme,
      openCustomize,
      triggerBtn,
      panelEl,
      _internals: internals,
    }
  },

  mount(state, host) {
    // `mount`'s `state` is the unwrapped writable proxy: reading a signal
    // key yields its value, refs yield the element (or null), writes route
    // through `.set`. The raw signals — needed for reactive subscription
    // inside `effect` — live on `state.$signals`.
    const { panelEl: panelSignal, triggerBtn: triggerSignal,
      activeTheme: activeThemeSignal, panelOpen: panelOpenSignal } = state.$signals

    state._internals.host = host
    ensureThemesCssInShadow(host, state._internals.tokensHref())

    // Apply stored theme on first mount. Idempotent if the boot script
    // already applied it (re-applying is harmless). The picker also
    // emits `sui-theme-change` from `selectTheme()`, so sibling pickers
    // listening on the same channel stay in sync.
    // In `scope` mode the document must not be touched; sync the host's own
    // subtree (no read-side effect — the picker's `activeTheme` signal was
    // already seeded from `readStoredTheme()` in scope) and skip the write.
    if (state._internals.scope().length === 0) {
      applyTheme(readStoredTheme())
    }

    // Sync panel ref into internals + listen for native open/close so
    // panelOpen mirrors the popover state (drives aria-expanded).
    const stopPanelSync = effect(() => {
      const panel = panelSignal()
      state._internals.panel = panel
      if (!panel) return
      const onToggle = (e: Event) => {
        const newState = (e as ToggleEvent).newState
        const open = newState === 'open'
        if (panelOpenSignal() !== open) panelOpenSignal.set(open)
        if (open) anchorPanel()
      }
      panel.addEventListener('toggle', onToggle)
      return () => panel.removeEventListener('toggle', onToggle)
    })

    function anchorPanel(): void {
      const panel = panelSignal()
      const trigger = triggerSignal()
      if (!panel || !trigger) return
      const r = trigger.getBoundingClientRect()
      panel.style.top = `${r.bottom + 4}px`
      panel.style.left = `${Math.max(8, r.right - panel.offsetWidth)}px`
    }

    // Sync internal selection to the neutral `sui-theme-change` event so
    // sibling pickers (and any host code that emits on the same channel)
    // stay in sync. The picker itself emits `sui-theme-change` on
    // selection; the consumer is responsible for re-broadcasting external
    // theme applications (settings view, devtools, programmatic API) onto
    // this channel — neutral prop/event API, not a product SDK read.
    function onExternalChange(e: Event) {
      const id = (e as CustomEvent<{ id?: string }>).detail?.id
      if (!isValidTheme(id)) return
      if (activeThemeSignal() !== id) activeThemeSignal.set(id)
    }
    document.addEventListener('sui-theme-change', onExternalChange)

    // Arrow-key nav across cards while the panel is open.
    function onPanelKeydown(e: KeyboardEvent) {
      const panel = panelSignal()
      if (!panel) return
      const cards = Array.from(panel.querySelectorAll<HTMLButtonElement>('.card'))
      if (!cards.length) return
      const root = panel.getRootNode() as Document | ShadowRoot
      const idx = cards.findIndex((c) => c === root.activeElement)
      switch (e.key) {
        case 'ArrowDown':
          e.preventDefault()
          cards[(idx + 1 + cards.length) % cards.length]?.focus()
          break
        case 'ArrowUp':
          e.preventDefault()
          cards[(idx - 1 + cards.length) % cards.length]?.focus()
          break
        case 'Home':
          e.preventDefault()
          cards[0]?.focus()
          break
        case 'End':
          e.preventDefault()
          cards[cards.length - 1]?.focus()
          break
        case 'Escape':
          e.preventDefault()
          panel.hidePopover?.()
          break
      }
    }

    const stopKeyBind = effect(() => {
      const panel = panelSignal()
      if (!panel) return
      panel.addEventListener('keydown', onPanelKeydown)
      return () => panel.removeEventListener('keydown', onPanelKeydown)
    })

    // Focus the selected card when the panel opens — keyboard users
    // expect a starting position.
    const stopFocusOnOpen = effect(() => {
      if (!panelOpenSignal()) return
      requestAnimationFrame(() => {
        const panel = panelSignal()
        if (!panel) return
        const cards = Array.from(panel.querySelectorAll<HTMLButtonElement>('.card'))
        const current = cards.find((c) => c.getAttribute('aria-selected') === 'true')
        ;(current ?? cards[0])?.focus()
      })
    })

    return () => {
      document.removeEventListener('sui-theme-change', onExternalChange)
      stopPanelSync()
      stopKeyBind()
      stopFocusOnOpen()
    }
  },
})
