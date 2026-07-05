// <sui-component-catalog> — `/components` "gallery" index. Reads the build-time
// `COMPONENTS_CATALOG` (emitted by `deck build`'s cem pass into
// `apps/sui-site/components.gen.ts`) and projects it into a searchable,
// category-filtered card gallery. Framework-pure — no ship-platform coupling.
//
// Curated surfaces (this commit):
//   - Grouping: by CATEGORY (Forms & Inputs / Layout / Overlays / etc.), not
//     by pack. The pack field is preserved on every entry (the /components/
//     [tag] detail page still reads it) but the chip row drops the pack
//     axis entirely.
//   - Hidden surface: the internal sub-primitives that compose but never
//     appear standalone (accordion-item / tab / tab-panel / tree-item /
//     timeline-item / toast* / radio etc.) are filtered out of the entry
//     list in `buildCatalog()`, before the card array is built, so neither
//     the chip row nor the empty-state ever see them.
//   - Search: case-insensitive substring on the tag, two-way bound through
//     the <sui-input>'s custom `sui-input` event (its native `input` does
//     not cross the shadow boundary).
//
// Interactivity is two signals: `query` (the search text) and
// `activeCategory` (the category chip). A card's visibility derives from
// both — `visible(entry)` — so filtering is a reactive `attr:hidden`
// toggle, never a re-render. The one-shot preview mount therefore fires
// exactly once: every registered specimen mounts once in `mount()`, and
// filtering only shows/hides already-mounted cards.
//
// The preview mount is the SAME harness the catalog has always used:
// for each `[data-preview-for]` stage the template renders, mount one
// bare instance of the registered custom element, seeded from
// `PREVIEW_SEEDS` (default props via setAttribute, default-slot innerHTML
// via the runtime's `bindHtml` — NOT a raw `el.innerHTML = …` write,
// which the sui-controller-purity lint forbids).

import { controller, signal, derived, bindHtml, type Derived } from '@ship/sui-runtime'
import type { Scope } from './sui-component-catalog.sui'
import { COMPONENTS_CATALOG } from '../../components.gen'
import { PREVIEW_SEEDS, type PreviewSeed } from './preview-seeds'
import {
  HIDDEN_TAGS,
  TAG_CATEGORY,
  CATEGORY_LABELS,
  CATEGORY_ORDER,
  type CategoryId,
} from './category-map'

// ── Types ────────────────────────────────────────────────────────────────────

export interface ComponentEntry {
  tag: string
  pack: string
  /** Public category the tag belongs to (see `category-map.ts`). */
  category: CategoryId
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

export interface CategoryChip {
  id: CategoryId | 'all'
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

/** Resolve the public category for a tag. Falls through to `CategoryId.Utility`
 *  when the tag is missing from the map — under-claim, never over-claim. */
function categoryFor(tag: string): CategoryId {
  const explicit = TAG_CATEGORY[tag]
  return explicit ?? 'utility'
}

// Per-tag preview seed lives in the shared `preview-seeds` module — imported
// above and consumed by BOTH this grid and the /components/[tag] detail page.
// Tags absent from the map fall through to a bare createElement with no slot
// content. `PreviewSeed` is re-exported for any downstream typing needs.
export type { PreviewSeed }

// ── Catalog ingestion ────────────────────────────────────────────────────────

/** Project the generated CEM into a flat, tag-sorted entry list plus the
 *  category chip row. Filters out hidden sub-primitives BEFORE projection so
 *  the chip-row counts reflect what the user actually sees. Pure + sync —
 *  bundled data computed once at module load. */
function buildCatalog(): { entries: ComponentEntry[]; chips: CategoryChip[] } {
  const catalog = COMPONENTS_CATALOG as unknown as { modules?: CemModule[] }
  const modules = catalog.modules ?? []
  const entries: ComponentEntry[] = []
  const catCounts = new Map<CategoryId, number>()
  for (const m of modules) {
    const pack = m.package ?? 'workspace'
    for (const d of m.declarations ?? []) {
      if (d.kind !== 'class' || !d.customElement || !d.tagName) continue
      const tag = d.tagName
      // Curated hidden surface: internal sub-primitives compose but never
      // appear as standalone catalog entries.
      if (HIDDEN_TAGS[tag]) continue
      const category = categoryFor(tag)
      entries.push({
        tag,
        pack,
        category,
        description: (d.description ?? '').split('\n')[0]?.trim() ?? '',
        propCount: (d.attributes ?? []).length,
        slotCount: (d.slots ?? []).length,
        eventCount: (d.events ?? []).length,
      })
      catCounts.set(category, (catCounts.get(category) ?? 0) + 1)
    }
  }
  entries.sort((a, b) => a.tag.localeCompare(b.tag))

  // Build the category chip row in the canonical display order from
  // `CATEGORY_ORDER`. Any zero-count category still appears as an empty chip
  // so the chip row is stable across builds (no flicker on a new
  // primitive landing under an empty category).
  const chips: CategoryChip[] = [{ id: 'all', label: 'All', count: entries.length }]
  for (const id of CATEGORY_ORDER) {
    chips.push({ id, label: CATEGORY_LABELS[id], count: catCounts.get(id) ?? 0 })
  }
  return { entries, chips }
}

const { entries: ENTRIES, chips: CHIPS } = buildCatalog()

// ── Controller ───────────────────────────────────────────────────────────────

export default controller<Scope>({
  scope: () => {
    const query = signal<string>('')
    const activeCategory = signal<CategoryId | 'all'>('all')

    // <sui-input> emits a custom `sui-input` event (its internal native input's
    // `input` doesn't cross the shadow boundary), so a native two-way
    // `bind:value` would be write-only. Drive the query signal from the custom
    // event's detail instead — one-way input → signal is all search needs.
    const onSearch = (e: Event): void => {
      const detail = (e as CustomEvent<{ value?: string }>).detail
      query.set(detail?.value ?? '')
    }
    const setCategory = (id: CategoryId | 'all'): void => { activeCategory.set(id) }

    // A card is visible when it matches BOTH the active category AND the
    // search substring (case-insensitive, on the tag). Reads both signals.
    const visible = (c: ComponentEntry): boolean => {
      const q = query().trim().toLowerCase()
      const catOk = activeCategory() === 'all' || c.category === activeCategory()
      const qOk = q === '' || c.tag.toLowerCase().includes(q)
      return catOk && qOk
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
      chips: derived<CategoryChip[]>(() => CHIPS),
      totalComponents: derived<number>(() => ENTRIES.length),
      totalCategories: derived<number>(() => CHIPS.length - 1),
      visibleCount: derived<number>(() => ENTRIES.filter((c) => visible(c)).length),
      query,
      activeCategory,
      setCategory,
      onSearch,
    }
  },

  mount(_state: unknown, host: HTMLElement) {
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
