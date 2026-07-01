// <sui-component-catalog> — `/components` index. Reads the build-time
// `COMPONENTS_CATALOG` (emitted by `deck build`'s cem pass into
// `apps/sui-site/components.gen.ts`) and groups every component by source
// pack. Framework-pure — no ship-platform coupling.
//
// After the template renders the per-component preview slots, the controller
// live-mounts a bare instance of every tag the bundle actually registered
// (`customElements.get`). A tag that isn't in the bundle still lists with
// its metadata — coverage over polish.

import { controller, derived } from '@ship/sui-runtime'
import type { Scope } from './sui-component-catalog.sui'
import { COMPONENTS_CATALOG } from '../../components.gen'

interface PropMeta { name: string; type: string; default?: unknown }
interface NamedMeta { name: string; description?: string }
export interface ComponentEntry {
  tag: string
  pkg: string
  description: string
  props: PropMeta[]
  slots: NamedMeta[]
  events: NamedMeta[]
}
export interface PackGroup {
  id: string
  pkg: string
  count: number
  components: ComponentEntry[]
}

interface CemAttr { name?: string; type?: { text?: string }; default?: unknown }
interface CemDecl {
  kind?: string
  customElement?: boolean
  tagName?: string
  description?: string
  attributes?: CemAttr[]
  slots?: Array<{ name?: string; description?: string }>
  events?: Array<{ name?: string; description?: string }>
  members?: unknown[]
}
interface CemModule { package?: string; declarations?: CemDecl[] }

/** Framework `@ship/sui-*` packs sort before app/domain `@ship/*` packs;
 *  ties break alphabetically. A stable, readable pack order for the browser. */
function packRank(pkg: string): number {
  return pkg.startsWith('@ship/sui-') ? 0 : 1
}

/** Project the generated CEM into pack-grouped component entries. Pure +
 *  synchronous — the catalog is bundled data, computed once at module load. */
function buildGroups(): PackGroup[] {
  const modules = (COMPONENTS_CATALOG as { modules?: CemModule[] }).modules ?? []
  const byPack = new Map<string, ComponentEntry[]>()
  for (const m of modules) {
    const pkg = m.package ?? 'workspace'
    for (const d of m.declarations ?? []) {
      if (d.kind !== 'class' || !d.customElement || !d.tagName) continue
      const entry: ComponentEntry = {
        tag: d.tagName,
        pkg,
        description: ((d.description ?? '').split('\n')[0]?.trim()) ?? '',
        props: (d.attributes ?? []).map((a): PropMeta => ({
          name: a.name ?? '',
          type: a.type?.text ?? 'string',
          ...(a.default === undefined ? {} : { default: a.default }),
        })),
        slots: (d.slots ?? []).map((s) => ({ name: s.name ?? '(default)', description: s.description })),
        events: (d.events ?? []).map((e) => ({ name: e.name ?? '', description: e.description })),
      }
      const list = byPack.get(pkg) ?? []
      list.push(entry)
      byPack.set(pkg, list)
    }
  }
  return [...byPack.entries()]
    .map(([pkg, list]): PackGroup => ({
      id: pkg.replace(/[^a-z0-9]+/gi, '-'),
      pkg,
      count: list.length,
      components: list.sort((a, b) => a.tag.localeCompare(b.tag)),
    }))
    .sort((a, b) => packRank(a.pkg) - packRank(b.pkg) || a.pkg.localeCompare(b.pkg))
}

const GROUPS: PackGroup[] = buildGroups()
const TOTAL_COMPONENTS = GROUPS.reduce((n, g) => n + g.count, 0)

export default controller<Scope>({
  scope: () => ({
    groups: derived<PackGroup[]>(() => GROUPS),
    totalComponents: derived<number>(() => TOTAL_COMPONENTS),
    totalPacks: derived<number>(() => GROUPS.length),
  }),

  mount(_state, host) {
    const root = (host.shadowRoot ?? host) as ParentNode
    const mounted: Element[] = []
    const raf = requestAnimationFrame(() => {
      const slots = root.querySelectorAll<HTMLElement>('[data-preview-for]')
      slots.forEach((slot) => {
        const tag = slot.getAttribute('data-preview-for')
        if (tag && customElements.get(tag)) {
          try {
            const el = document.createElement(tag)
            // A bare `createElement` + `appendChild` is enough for the
            // vast majority of previews. The single confirmed exception is
            // <sui-theme-picker>: its mount calls `applyTheme()` which
            // writes `data-theme` + `.dark` on documentElement. The picker
            // exposes a `scope` prop that, when non-empty, isolates those
            // writes to its host subtree. Setting `scope="preview"` here
            // keeps the catalog from flipping the document theme on every
            // page load.
            if (tag === 'sui-theme-picker') el.setAttribute('scope', 'preview')
            slot.appendChild(el)
            slot.dataset.previewState = 'live'
            mounted.push(el)
            return
          } catch { /* fall through */ }
        }
        const note = document.createElement('span')
        note.className = 'preview-empty'
        note.textContent = 'not in bundle'
        slot.appendChild(note)
        slot.dataset.previewState = 'unavailable'
        mounted.push(note)
      })
    })
    return () => {
      cancelAnimationFrame(raf)
      for (const el of mounted) el.remove()
    }
  },
})
