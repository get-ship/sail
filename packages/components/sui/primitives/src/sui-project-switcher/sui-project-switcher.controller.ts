// <sui-project-switcher> controller — generic, SDK-free project (space) picker.
//
// The original @ship/chrome version called `ship.space.list/activate` on a
// Connect client, `navigate('/')` for the post-activate reload, and a pack-local
// `openSpace()` transport. Here NONE of that exists: the project list arrives via
// the `projects` IDENTITY property (or a `projects` JSON attribute) + the
// `active-id` prop, and selections leave as `sui-project-*` events. The consuming
// app owns the SDK, the reload, and the open-window transport. RUNTIME-FREE.

import { controller, signal, derived, effect, ref } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-project-switcher.sui'

/** A switchable project/space. Only `id` + `name` are required. */
export interface SwitcherProject {
  id: string
  name: string
  path?: string
  /** A design hue token name; derived from `id` when omitted. */
  hue?: string
}

interface ProjectRow {
  id: string
  name: string
  path: string
  hue: string
}

type PopoverEl = HTMLElement & { showPopover?: () => void; hidePopover?: () => void }

// Deterministic per-project identity hue when the caller supplies none — stable
// across reloads. Pure (no SDK): a hash of the id picks from the hue palette.
const HUES = ['orange', 'amber', 'red', 'violet', 'blue', 'teal', 'green', 'pink', 'slate'] as const
function pickHue(id: string): string {
  if (!id) return 'slate'
  let h = 0
  for (let i = 0; i < id.length; i++) h = (h * 31 + id.charCodeAt(i)) | 0
  return HUES[Math.abs(h) % HUES.length]!
}

/** Parse the `projects='[…]'` JSON attribute (SSR fallback). Never throws. */
function parseJsonProjects(raw: string): SwitcherProject[] {
  if (!raw) return []
  let parsed: unknown
  try {
    parsed = JSON.parse(raw)
  } catch {
    return []
  }
  if (!Array.isArray(parsed)) return []
  const out: SwitcherProject[] = []
  for (const entry of parsed) {
    if (entry == null || typeof entry !== 'object') continue
    const o = entry as Record<string, unknown>
    if (o.id == null) continue
    const p: SwitcherProject = { id: String(o.id), name: o.name != null ? String(o.name) : String(o.id) }
    if (o.path != null) p.path = String(o.path)
    if (o.hue != null) p.hue = String(o.hue)
    out.push(p)
  }
  return out
}

export default controller({
  scope: ({ activeId }: ViewScope<Scope>) => {
    const projectsSig = signal<SwitcherProject[]>([])
    const menuOpen = signal(false)

    const rows = derived<ProjectRow[]>(() =>
      projectsSig().map((p) => ({
        id: p.id,
        name: p.name,
        path: p.path != null ? p.path : '',
        hue: p.hue != null && p.hue !== '' ? p.hue : pickHue(p.id),
      })),
    )
    const activeName = derived(
      () => projectsSig().find((p) => p.id === activeId())?.name ?? 'Project',
    )
    const activeHue = derived(() => {
      const p = projectsSig().find((x) => x.id === activeId())
      return p && p.hue != null && p.hue !== '' ? p.hue : pickHue(activeId())
    })
    const triggerAria = derived(() => `Active project: ${activeName()}. Switch project.`)

    const triggerBtn = ref<HTMLButtonElement>(null)
    const menuEl = ref<HTMLDivElement>(null)
    const internals: { menu: PopoverEl | null; host: HTMLElement | null } = {
      menu: null,
      host: null,
    }

    function toggleMenu(): void {
      const menu = internals.menu
      if (!menu) return
      if (menuOpen()) menu.hidePopover?.()
      else menu.showPopover?.()
    }
    function emit(eventName: string, id: string): void {
      internals.menu?.hidePopover?.()
      internals.host?.dispatchEvent(
        new CustomEvent(eventName, { detail: { id }, bubbles: true, composed: true }),
      )
    }

    return {
      projectsSig,
      menuOpen,
      rows,
      activeName,
      activeHue,
      triggerAria,
      triggerBtn,
      menuEl,
      toggleMenu,
      // Selecting the already-active project is a no-op (no reload churn).
      pick: (id: string) => {
        if (id === activeId()) {
          internals.menu?.hidePopover?.()
          return
        }
        emit('sui-project-select', id)
      },
      openAsWindow: (id: string) => emit('sui-project-open', id),
      _internals: internals,
    }
  },

  mount(state, host) {
    state._internals.host = host

    // `projects` IDENTITY property + JSON-attribute SSR fallback (the sui-tree
    // pattern). Not a declared <prop>, so the instance accessor clashes with
    // nothing.
    Object.defineProperty(host, 'projects', {
      configurable: true,
      get: () => state.projectsSig,
      set: (v: SwitcherProject[] | null) => {
        state.projectsSig = Array.isArray(v) ? v : []
      },
    })
    if (state.projectsSig.length === 0) {
      const parsed = parseJsonProjects(host.getAttribute('projects') ?? '')
      if (parsed.length > 0) state.projectsSig = parsed
    }

    function anchor(): void {
      const menu = state.menuEl
      const trigger = state.triggerBtn
      if (!menu || !trigger) return
      const r = trigger.getBoundingClientRect()
      menu.style.top = `${r.bottom + 4}px`
      menu.style.left = `${Math.max(8, r.left)}px`
    }

    const stopSync = effect(() => {
      const menu = state.menuEl as PopoverEl | null
      state._internals.menu = menu
      if (!menu) return
      const onToggle = (e: Event) => {
        const open = (e as ToggleEvent).newState === 'open'
        if (state.menuOpen !== open) state.menuOpen = open
        if (open) anchor()
      }
      menu.addEventListener('toggle', onToggle)
      return () => menu.removeEventListener('toggle', onToggle)
    })

    host.emit?.('sui-project-switcher-ready', {})

    return () => {
      stopSync()
    }
  },
})
