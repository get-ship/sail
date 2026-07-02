// <sui-component-catalog> — `/components` "gallery" index. Reads the build-time
// `COMPONENTS_CATALOG` (emitted by `deck build`'s cem pass into
// `apps/sui-site/components.gen.ts`) and projects it into a flat, searchable
// card gallery. Framework-pure — no ship-platform coupling.
//
// Interactivity is two signals: `query` (search text, two-way bound to the
// <sui-input> via bind:value) and `activePack` (the pack chip filter). A card's
// visibility derives from both — `visible(entry)` — so filtering is a reactive
// attribute toggle (`attr:hidden`), never a re-render. That keeps the live
// preview mount a single one-shot pass: every registered specimen mounts once
// in `mount()`, and filtering only shows/hides already-mounted cards.
//
// The preview mount is the SAME harness the catalog has always used: for each
// `[data-preview-for]` stage the template renders, mount one bare instance of
// the registered custom element, seeded from `PREVIEW_SEEDS` (default props +
// default-slot innerHTML via the runtime's `bindHtml` — NOT a raw
// `el.innerHTML = …` write, which the sui-controller-purity lint forbids).

import { controller, signal, derived, bindHtml, type Derived } from '@ship/sui-runtime'
import type { Scope } from './sui-component-catalog.sui'
import { COMPONENTS_CATALOG } from '../../components.gen'
import { PREVIEW_SEEDS, type PreviewSeed } from './preview-seeds'

// ── Types ────────────────────────────────────────────────────────────────────

export interface ComponentEntry {
  tag: string
  pack: string
  description: string
  propCount: number
  slotCount: number
  eventCount: number
}

/** A catalog entry decorated with its own reactive `hidden` flag. Template
 *  bindings may only call zero-arg signal reads, so per-card visibility can't
 *  be a `visible(entry)` call in the markup — instead each entry carries a
 *  `hidden` derived, read as the zero-arg `c.hidden()`. */
export interface CardEntry extends ComponentEntry {
  hidden: Derived<boolean>
}

export interface PackChip {
  id: string
  label: string
  count: number
}

interface CemAttr { name?: string; type?: { text?: string }; default?: unknown }
interface CemDecl {
  kind?: string
  customElement?: boolean
  tagName?: string
  description?: string
  attributes?: CemAttr[]
  slots?: Array<{ name?: string }>
  events?: Array<{ name?: string }>
}
interface CemModule { package?: string; declarations?: CemDecl[] }

/** Framework `@ship/sui-*` packs sort before app/domain packs; ties break
 *  alphabetically. A stable, readable pack order for the chip row. */
function packRank(pkg: string): number {
  return pkg.startsWith('@ship/sui-') ? 0 : 1
}

/** Short chip label for a pack id — strip the `@ship/sui-` (or `@ship/`)
 *  namespace so the chip reads `primitives` / `viewer` / `code-editor`. */
function packLabel(pkg: string): string {
  return pkg.replace(/^@ship\/sui-/, '').replace(/^@ship\//, '')
}

// Per-tag preview seed lives in the shared `preview-seeds` module — imported
// above and consumed by BOTH this grid and the /components/[tag] detail page.
// Tags absent from the map fall through to a bare createElement with no slot
// content. `PreviewSeed` is re-exported for any downstream typing needs.
export type { PreviewSeed }

// ── Catalog ingestion ────────────────────────────────────────────────────────

/** Project the generated CEM into a flat, tag-sorted entry list plus the pack
 *  chip row. Pure + synchronous — bundled data computed once at module load. */
function buildCatalog(): { entries: ComponentEntry[]; packs: PackChip[] } {
  const modules = (COMPONENTS_CATALOG as { modules?: CemModule[] }).modules ?? []
  const entries: ComponentEntry[] = []
  const packCounts = new Map<string, number>()
  for (const m of modules) {
    const pack = m.package ?? 'workspace'
    for (const d of m.declarations ?? []) {
      if (d.kind !== 'class' || !d.customElement || !d.tagName) continue
      entries.push({
        tag: d.tagName,
        pack,
        description: (d.description ?? '').split('\n')[0]?.trim() ?? '',
        propCount: (d.attributes ?? []).length,
        slotCount: (d.slots ?? []).length,
        eventCount: (d.events ?? []).length,
      })
      packCounts.set(pack, (packCounts.get(pack) ?? 0) + 1)
    }
  }
  entries.sort((a, b) => a.tag.localeCompare(b.tag))

  const packChips: PackChip[] = [...packCounts.entries()]
    .map(([id, count]): PackChip => ({ id, label: packLabel(id), count }))
    .sort((a, b) => packRank(a.id) - packRank(b.id) || a.id.localeCompare(b.id))
  const packs: PackChip[] = [{ id: 'all', label: 'All', count: entries.length }, ...packChips]

  return { entries, packs }
}

const { entries: ENTRIES, packs: PACKS } = buildCatalog()

// ── Controller ───────────────────────────────────────────────────────────────

export default controller<Scope>({
  scope: () => {
    const query = signal<string>('')
    const activePack = signal<string>('all')

    // <sui-input> emits a custom `sui-input` event (its internal native input's
    // `input` doesn't cross the shadow boundary), so a native two-way
    // `bind:value` would be write-only. Drive the query signal from the custom
    // event's detail instead — one-way input → signal is all search needs.
    const onSearch = (e: Event): void => {
      const detail = (e as CustomEvent<{ value?: string }>).detail
      query.set(detail?.value ?? '')
    }
    const setPack = (id: string): void => { activePack.set(id) }

    // A card is visible when it matches BOTH the active pack filter and the
    // search substring (case-insensitive, on the tag). Reads both signals.
    const visible = (c: ComponentEntry): boolean => {
      const q = query().trim().toLowerCase()
      const packOk = activePack() === 'all' || c.pack === activePack()
      const qOk = q === '' || c.tag.toLowerCase().includes(q)
      return packOk && qOk
    }

    // Decorate each entry with a reactive `hidden` derived so the template can
    // toggle card visibility via the zero-arg `c.hidden()` read (a `visible(c)`
    // call with an argument is rejected in renderable expressions). The list
    // itself is stable — only each card's `hidden` flips — so the one-shot
    // preview mount is never disturbed by filtering.
    const cards: CardEntry[] = ENTRIES.map((e) => ({
      ...e,
      hidden: derived<boolean>(() => !visible(e)),
    }))

    return {
      components: derived<CardEntry[]>(() => cards),
      packs: derived<PackChip[]>(() => PACKS),
      totalComponents: derived<number>(() => ENTRIES.length),
      totalPacks: derived<number>(() => PACKS.length - 1),
      visibleCount: derived<number>(() => ENTRIES.filter((c) => visible(c)).length),
      query,
      activePack,
      setPack,
      onSearch,
    }
  },

  mount(_state, host) {
    // One-shot preview mount: for each rendered [data-preview-for] stage, mount
    // a single bare instance of the registered custom element. Seeded from
    // PREVIEW_SEEDS (props via setAttribute, default-slot innerHTML via the
    // runtime's `bindHtml` — the morph-backed binding, NOT a raw innerHTML
    // write). Filtering later toggles card `hidden`; mounted previews persist.
    //
    // `<sui-theme-picker>` is the one primitive whose mount has document-level
    // side effects (`applyTheme()` writes data-theme on documentElement); it
    // exposes a `scope` prop to isolate those writes — set `scope="preview"` so
    // the catalog never flips the document theme on load.
    const root = (host.shadowRoot ?? host) as ParentNode
    const mounted: Element[] = []
    const binds: Array<() => void> = []
    const raf = requestAnimationFrame(() => {
      const stages = root.querySelectorAll<HTMLElement>('[data-preview-for]')
      stages.forEach((stage) => {
        const tag = stage.getAttribute('data-preview-for')
        if (!tag || !customElements.get(tag)) {
          const note = document.createElement('span')
          note.className = 'preview-empty'
          note.textContent = 'not in bundle'
          stage.appendChild(note)
          stage.dataset.previewState = 'unavailable'
          mounted.push(note)
          return
        }
        try {
          const el = document.createElement(tag)
          const seed = PREVIEW_SEEDS[tag]
          if (seed?.props) {
            for (const [k, v] of Object.entries(seed.props)) el.setAttribute(k, v)
          }
          if (tag === 'sui-theme-picker') el.setAttribute('scope', 'preview')
          stage.appendChild(el)
          if (seed?.innerHTML) {
            const seedHtml = seed.innerHTML
            binds.push(bindHtml(el, () => seedHtml))
          }
          stage.dataset.previewState = 'live'
          mounted.push(el)
        } catch {
          const note = document.createElement('span')
          note.className = 'preview-empty'
          note.textContent = 'not in bundle'
          stage.appendChild(note)
          stage.dataset.previewState = 'unavailable'
          mounted.push(note)
        }
      })
    })
    return () => {
      cancelAnimationFrame(raf)
      for (const unbind of binds) unbind()
      for (const el of mounted) el.remove()
    }
  },
})
