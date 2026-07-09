// <sui-component-catalog> — `/components` "gallery" index. Reads the build-time
// `COMPONENTS_CATALOG` (emitted by `deck build`'s cem pass into
// `apps/sui-site/components.gen.ts`) and projects it into a searchable,
// surface-filtered card gallery. Framework-pure — no ship-platform coupling.
//
// Curated surfaces (this commit):
//   - Surface axis: TWO chips — Standard Library (default) vs Packs. There
//     is no chip for "all"; the Standard Library chip IS the gallery, and
//     Packs is a distinct secondary surface. There are no invented category
//     buckets (no Forms / Layout / Overlays / Navigation / Feedback / Data /
//     Typography / Utility).
//   - Standard Library: the `@ship/sui-primitives` pack, surfaced as ONE
//     clean alphabetical grid (no sub-category headers).
//   - Packs: every Sail-authored pack grouped under a pack-name header, in
//     canonical order (Charts, Canvas, Chat Elements, Media, Code Editor,
//     Mermaid). Standard-library entries never appear here.
//   - Hidden surface: the internal sub-primitives that compose but never
//     appear standalone (accordion-item / tab / tab-panel / tree-item /
//     timeline-item / toast* / radio etc.) plus the entire viewer pack are
//     filtered out of the entry list in `buildCatalog()`, before either
//     chip is computed, so a chip never promises more cards than it shows.
//   - Search: case-insensitive substring on the tag, two-way bound through
//     the <sui-input>'s custom `sui-input` event (its native `input` does
//     not cross the shadow boundary).
//
// Interactivity is two signals: `query` (the search text) and `activeSurface`
// (the chip). A card's visibility derives from both — `visible(entry)` — so
// filtering is a reactive `attr:hidden` toggle, never a re-render. The one-
// shot preview mount therefore fires exactly once: every registered specimen
// mounts once in `mount()`, and filtering only shows/hides already-mounted
// cards.
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
  PACK_LABELS,
  PACK_ORDER,
  packOf,
  surfaceOf,
  type Surface,
} from './category-map'

// ── Types ────────────────────────────────────────────────────────────────────

export interface ComponentEntry {
  tag: string
  /** CEM `package` field — the npm pack name (e.g. `@ship/sui-charts`,
   *  `@ship/sui-primitives`). Preserved on every entry for the detail page. */
  pack: string
  /** Surface label: pack name for pack tags, `null` for standard-library tags. */
  packLabel: string | null
  description: string
  propCount: number
  slotCount: number
  eventCount: number
  /** Bento cell size. Data/complex components earn a 2×2 (`lg`); horizontal
   *  ones a 2×1 (`wide`); everything else is a 1×1 (`sm`, the default). */
  size: BentoSize
}

export type BentoSize = 'sm' | 'wide' | 'lg'

/** Cell size by component nature — the bento rhythm. Absent tags = `sm`. */
const BENTO_SIZE: Record<string, BentoSize> = {
  // 2×2 — data-dense / complex surfaces that need room to read.
  'sui-table': 'lg',
  'sui-tree-grid': 'lg',
  'sui-calendar': 'lg',
  'sui-code-editor': 'lg',
  'sui-diff': 'lg',
  'sui-markdown': 'lg',
  'sui-spreadsheet': 'lg',
  'sui-data-grid': 'lg',
  'sui-kanban': 'lg',
  'sui-command-palette': 'lg',
  'sui-json-view': 'lg',
  // 2×1 — horizontal / list-shaped components.
  'sui-alert': 'wide',
  'sui-banner': 'wide',
  'sui-timeline': 'wide',
  'sui-tabs': 'wide',
  'sui-nav-top': 'wide',
  'sui-navigation-menu': 'wide',
  'sui-breadcrumb': 'wide',
  'sui-pagination': 'wide',
  'sui-progress': 'wide',
  'sui-stepper': 'wide',
  'sui-accordion': 'wide',
  'sui-collapsible': 'wide',
  'sui-toast': 'wide',
  'sui-scroll-area': 'wide',
  'sui-theme-picker': 'wide',
}

/** A catalog entry decorated with its own reactive `hidden` flag. Template
 *  bindings may only call zero-arg signal reads, so per-card visibility can't
 *  be a `visible(entry)` call in the markup — instead each entry carries a
 *  `hidden` derived, read as the zero-arg `c.hidden()`. */
export interface CardEntry extends ComponentEntry {
  hidden: Derived<boolean>
}

/** A chip on the surface row. Exactly two chips: Standard Library (default)
 *  and Packs. */
export interface SurfaceChip {
  id: Surface
  label: string
  count: number
}

/** A pack group in the Packs view: a header label + the cards in that pack,
 *  alphabetical. Empty packs are omitted from the rendered surface. */
export interface PackGroup {
  pack: string
  label: string
  cards: CardEntry[]
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

// Per-tag preview seed lives in the shared `preview-seeds` module — imported
// above and consumed by BOTH this grid and the /components/[tag] detail page.
// Tags absent from the map fall through to a bare createElement with no slot
// content. `PreviewSeed` is re-exported for any downstream typing needs.
export type { PreviewSeed }

// ── Catalog ingestion ────────────────────────────────────────────────────────

/** Project the generated CEM into a flat, tag-sorted entry list plus the
 *  two-chip surface row + pack-grouped sections. Filters out hidden sub-
 *  primitives BEFORE projection so the chip-row counts reflect what the user
 *  actually sees. Pure + sync — bundled data computed once at module load. */
function buildCatalog(): {
  entries: ComponentEntry[]
  chips: SurfaceChip[]
  packOrder: string[]
} {
  const catalog = COMPONENTS_CATALOG as unknown as { modules?: CemModule[] }
  const modules = catalog.modules ?? []
  const entries: ComponentEntry[] = []
  for (const m of modules) {
    const pack = m.package ?? 'workspace'
    for (const d of m.declarations ?? []) {
      if (d.kind !== 'class' || !d.customElement || !d.tagName) continue
      const tag = d.tagName
      // Curated hidden surface: internal sub-primitives compose but never
      // appear as standalone catalog entries.
      if (HIDDEN_TAGS[tag]) continue
      entries.push({
        tag,
        pack,
        packLabel: packOf(tag),
        description: (d.description ?? '').split('\n')[0]?.trim() ?? '',
        propCount: (d.attributes ?? []).length,
        slotCount: (d.slots ?? []).length,
        eventCount: (d.events ?? []).length,
        size: BENTO_SIZE[tag] ?? 'sm',
      })
    }
  }
  entries.sort((a, b) => a.tag.localeCompare(b.tag))

  // Two-chip row in canonical order. Each chip's count is the number of
  // visible entries on that surface — never inflated by hidden tags.
  const stdlibCount = entries.filter((e) => surfaceOf(e.tag) === 'stdlib').length
  const packCount = entries.filter((e) => surfaceOf(e.tag) === 'pack').length
  const chips: SurfaceChip[] = [
    { id: 'stdlib', label: 'Standard Library', count: stdlibCount },
    { id: 'pack', label: 'Packs', count: packCount },
  ]

  // Pack display order — only packs that actually have entries, in canonical
  // order. The runtime re-decorates each group's cards with reactive `hidden`
  // deriveds in `scope()`; this pre-pass just establishes the section
  // headers so the Packs view renders them in canonical order.
  const packOrder = PACK_ORDER.filter((pack) =>
    entries.some((e) => e.packLabel === pack),
  )

  return { entries, chips, packOrder }
}

const { entries: ENTRIES, chips: CHIPS, packOrder: PACK_ORDER_VISIBLE } = buildCatalog()
// ── Controller ───────────────────────────────────────────────────────────────

export default controller<Scope>({
  scope: () => {
    const query = signal<string>('')
    const activeSurface = signal<Surface>('stdlib')

    // <sui-input> emits a custom `sui-input` event (its internal native input's
    // `input` doesn't cross the shadow boundary), so a native two-way
    // `bind:value` would be write-only. Drive the query signal from the custom
    // event's detail instead — one-way input → signal is all search needs.
    const onSearch = (e: Event): void => {
      const detail = (e as CustomEvent<{ value?: string }>).detail
      query.set(detail?.value ?? '')
    }
    const setSurface = (id: Surface): void => { activeSurface.set(id) }

    // A card is visible when it matches BOTH the active surface AND the
    // search substring (case-insensitive, on the tag). Reads both signals.
    const visible = (c: ComponentEntry): boolean => {
      const q = query().trim().toLowerCase()
      const surfOk = surfaceOf(c.tag) === activeSurface()
      const qOk = q === '' || c.tag.toLowerCase().includes(q)
      return surfOk && qOk
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

    // Pack groups carry their own per-card `hidden` flag, parallel to `cards`
    // above. The Packs view renders from `packGroups` (so it can interleave
    // pack-name headers between card groups); the Standard Library view
    // renders from `cards`. Section order is the canonical PACK_ORDER_VISIBLE
    // list computed once at module load — only packs with at least one entry
    // appear, in the canonical Charts → Canvas → Chat Elements → Media →
    // Code Editor → Mermaid order.
    const packGroups: PackGroup[] = PACK_ORDER_VISIBLE.map((pack) => {
      const label = PACK_LABELS[pack] ?? pack
      const cards = ENTRIES
        .filter((e) => e.packLabel === pack)
        .map((e) => ({ ...e, hidden: derived<boolean>(() => !visible(e)) }))
      return { pack, label, cards }
    })

    return {
      components: derived<CardEntry[]>(() => cards),
      packGroups: derived<PackGroup[]>(() => packGroups),
      chips: derived<SurfaceChip[]>(() => CHIPS),
      totalComponents: derived<number>(() => ENTRIES.length),
      visibleCount: derived<number>(() => ENTRIES.filter((c) => visible(c)).length),
      query,
      activeSurface,
      setSurface,
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
        if (!tag) return
        const seed = PREVIEW_SEEDS[tag]
        // Static representative surface — overlays (closed + portaled) and
        // disclosure can't show themselves in a frozen card, so render a
        // hand-authored snapshot of the OPEN surface (native HTML + token
        // vars) instead of the live tag.
        if (seed?.staticHTML) {
          const wrap = document.createElement('div')
          wrap.className = 'preview-static'
          const html = seed.staticHTML
          binds.push(bindHtml(wrap, () => html))
          stage.appendChild(wrap)
          stage.dataset.previewState = 'static'
          mounted.push(wrap)
          return
        }
        if (!customElements.get(tag)) {
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
      // FIT-TO-STAGE: measure each specimen and scale down anything larger
      // than its stage so nothing bleeds out of the card (calendar, table,
      // wide inputs, the dialog surfaces). Runs after layout settles.
      requestAnimationFrame(() => {
        root.querySelectorAll<HTMLElement>('[data-preview-for]').forEach((stage) => {
          if (stage.dataset.previewState === 'unavailable') return
          const el = stage.firstElementChild as HTMLElement | null
          if (!el) return
          const pad = 2 * (parseFloat(getComputedStyle(stage).paddingLeft) || 0)
          const cw = stage.clientWidth - pad, ch = stage.clientHeight - pad
          const sw = el.scrollWidth, sh = el.scrollHeight
          if (cw > 0 && ch > 0 && (sw > cw + 2 || sh > ch + 2)) {
            const s = Math.max(0.4, Math.min(cw / sw, ch / sh))
            el.style.transform = `scale(${s})`
            el.style.transformOrigin = 'center'
          }
        })
      })
    })
    return () => {
      cancelAnimationFrame(raf)
      for (const unbind of binds) unbind()
      for (const el of mounted) el.remove()
    }
  },
})