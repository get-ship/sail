// <sui-component-catalog> — `/components` "gallery" index. Reads the build-time
// `COMPONENTS_CATALOG` (emitted by `deck build`'s cem pass into
// `apps/sui-site/components.gen.ts`) and projects it into a searchable,
// pack-grouped card gallery. Framework-pure — no ship-platform coupling.
//
// Curated surfaces (this commit):
//   - Grouping: by PACK, the eight npm workspace packages under
//     packages/components/sui/. The pack boundary is the shipping boundary —
//     "I need a charts pack component" — not an invented UI taxonomy. The
//     pack axis is rendered as section headers in the bento; the chip row
//     is a flat filter across all packs.
//   - Language elements: bare L1 keywords (row/column/grid/h1-h6/p/small/a/
//     strong/code/card/icon/label/separator/spinner) live in their OWN top
//     section, rendered as bare tags (no `sui-` prefix). A layout keyword
//     and an error-boundary never look like peers.
//   - Hidden surface: the internal sub-primitives that compose but never
//     appear standalone (accordion-item / tab / tab-panel / tree-item /
//     timeline-item / toggle / radio / navigation-item / toast* / viewer
//     internals) are filtered out of the entry list in `buildCatalog()`,
//     before the chip row is built — so a chip never promises more cards
//     than it actually shows.
//   - Search: case-insensitive substring on the tag, two-way bound through
//     the <sui-input>'s custom `sui-input` event (its native `input` does
//     not cross the shadow boundary).
//
// Interactivity is two signals: `query` (the search text) and
// `activePack` (the pack chip). A card's visibility derives from both —
// `visible(entry)` — so filtering is a reactive `attr:hidden` toggle, never
// a re-render. The one-shot preview mount therefore fires exactly once:
// every registered specimen mounts once in `mount()`, and filtering only
// shows/hides already-mounted cards.
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
  TAG_PACK,
  PACK_LABELS,
  PACK_BLURBS,
  PACK_ORDER,
  L1_LANGUAGE_ELEMENTS,
  type PackId,
  type LanguageElement,
  type LanguageGroupId,
} from './category-map'

// ── Types ────────────────────────────────────────────────────────────────────

export interface ComponentEntry {
  tag: string
  pack: PackId
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
  'sui-sheet': 'lg',
  'sui-heatmap': 'lg',
  'sui-board': 'lg',
  'sui-canvas': 'lg',
  'sui-draw': 'lg',
  'sui-graph-canvas': 'lg',
  // 2×1 — horizontal / list-shaped components.
  'sui-alert': 'wide',
  'sui-timeline': 'wide',
  'sui-tabs': 'wide',
  'sui-nav-top': 'wide',
  'sui-navigation-menu': 'wide',
  'sui-pagination': 'wide',
  'sui-accordion': 'wide',
  'sui-collapsible': 'wide',
  'sui-toast-host': 'wide',
  'sui-scroll-area': 'wide',
  'sui-message-list': 'wide',
  'sui-theme-picker': 'wide',
}

/** A catalog entry decorated with its own reactive `hidden` flag. Template
 *  bindings may only call zero-arg signal reads, so per-card visibility can't
 *  be a `visible(entry)` call in the markup — instead each entry carries a
 *  `hidden` derived, read as the zero-arg `c.hidden()`. */
export interface CardEntry extends ComponentEntry {
  hidden: Derived<boolean>
}

export interface PackChip {
  id: PackId | 'all'
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

/** Resolve the public pack for a tag. Falls through to `PackId.Primitives`
 *  when the tag is missing from the static map, OR when the CEM's `package`
 *  field is absent or doesn't name a known pack — under-claim, never
 *  over-claim. */
const packFor = (tag: string, pkg: string | undefined): PackId => {
  if (pkg) {
    const tail = pkg.replace(/^@ship\/sui-/, '')
    if ((PACK_ORDER as readonly string[]).includes(tail)) return tail as PackId
  }
  return TAG_PACK[tag] ?? 'primitives'
}

// Per-tag preview seed lives in the shared `preview-seeds` module — imported
// above and consumed by BOTH this grid and the /components/[tag] detail page.
// Tags absent from the map fall through to a bare createElement with no slot
// content. `PreviewSeed` is re-exported for any downstream typing needs.
export type { PreviewSeed, LanguageElement }

// ── Catalog ingestion ────────────────────────────────────────────────────────

/** A pack section as the template renders it: the pack id, label, blurb,
 *  and the entries that belong to it (already filtered + sorted). Each
 *  entry carries a reactive `hidden` derived so the template can toggle
 *  card visibility via the zero-arg `c.hidden()` read. */
export interface PackSection {
  id: PackId
  label: string
  blurb: string
  entries: CardEntry[]
}

/** Project the generated CEM into a flat, tag-sorted entry list plus the
 *  pack chip row and pack sections. Filters out hidden sub-primitives BEFORE
 *  projection so the chip-row counts reflect what the user actually sees.
 *  Pure + sync — bundled data computed once at module load. */
function buildCatalog(): { entries: ComponentEntry[]; chips: PackChip[]; sections: Omit<PackSection, 'entries'>[] } {
  const catalog = COMPONENTS_CATALOG as unknown as { modules?: CemModule[] }
  const modules = catalog.modules ?? []
  const entries: ComponentEntry[] = []
  const packCounts = new Map<PackId, number>()
  for (const m of modules) {
    for (const d of m.declarations ?? []) {
      if (d.kind !== 'class' || !d.customElement || !d.tagName) continue
      const tag = d.tagName
      // Curated hidden surface: internal sub-primitives compose but never
      // appear as standalone catalog entries.
      if (HIDDEN_TAGS[tag]) continue
      // The CEM's `package` field is the source of truth for the pack; the
      // static map is the defensive fallback (a stray module without a
      // package still lands in the right pack).
      const pack = packFor(tag, m.package)
      entries.push({
        tag,
        pack,
        description: (d.description ?? '').split('\n')[0]?.trim() ?? '',
        propCount: (d.attributes ?? []).length,
        slotCount: (d.slots ?? []).length,
        eventCount: (d.events ?? []).length,
        size: BENTO_SIZE[tag] ?? 'sm',
      })
      packCounts.set(pack, (packCounts.get(pack) ?? 0) + 1)
    }
  }
  entries.sort((a, b) => a.tag.localeCompare(b.tag))

  // Build the pack chip row in the canonical display order from
  // `PACK_ORDER`. Any zero-count pack still appears as a chip so the row
  // is stable across builds (no flicker on a new primitive landing under
  // an empty pack).
  const chips: PackChip[] = [{ id: 'all', label: 'All', count: entries.length }]
  for (const id of PACK_ORDER) {
    chips.push({ id, label: PACK_LABELS[id], count: packCounts.get(id) ?? 0 })
  }

  // Build pack section shells in display order. Zero-count packs are
  // dropped (a zero-card section is worse than no section). The entries
  // are attached per section BELOW in the scope factory, where the
  // reactive `hidden` derived can be wired to the search/pack signals.
  const byPack = new Map<PackId, ComponentEntry[]>()
  for (const e of entries) {
    const arr = byPack.get(e.pack) ?? []
    arr.push(e)
    byPack.set(e.pack, arr)
  }
  const sections: Omit<PackSection, 'entries'>[] = PACK_ORDER
    .filter((id) => (packCounts.get(id) ?? 0) > 0)
    .map((id) => ({ id, label: PACK_LABELS[id], blurb: PACK_BLURBS[id] }))

  return { entries, chips, sections }
}

const { entries: ENTRIES, chips: CHIPS, sections: SECTION_SHELLS } = buildCatalog()

/** L1 language elements grouped by sub-section for the top section. */
const L1_GROUPS = ((): readonly { id: LanguageGroupId; label: string; blurb: string; elements: LanguageElement[] }[] => {
  const byGroup: Record<LanguageGroupId, LanguageElement[]> = { layout: [], typography: [], forms: [], primitives: [] }
  for (const e of L1_LANGUAGE_ELEMENTS) byGroup[e.group].push(e)
  return [
    { id: 'layout' as const,     label: 'Layout',     blurb: 'Compiler-lowered flex & grid keywords.',     elements: byGroup.layout },
    { id: 'typography' as const, label: 'Typography', blurb: 'Text and inline marks, themed by tokens.',    elements: byGroup.typography },
    { id: 'forms' as const,      label: 'Controls',   blurb: 'Native form & disclosure elements, skinned in @layer sui — sizes/tones from attributes.', elements: byGroup.forms },
    { id: 'primitives' as const, label: 'Stdlib L1',  blurb: 'Foundational surfaces the framework ships.', elements: byGroup.primitives },
  ].filter((g) => g.elements.length > 0)
})()

// ── Controller ───────────────────────────────────────────────────────────────

export default controller<Scope>({
  scope: () => {
    const query = signal<string>('')
    const activePack = signal<PackId | 'all'>('all')

    // <sui-input> emits a custom `sui-input` event (its internal native input's
    // `input` doesn't cross the shadow boundary), so a native two-way
    // `bind:value` would be write-only. Drive the query signal from the custom
    // event's detail instead — one-way input → signal is all search needs.
    const onSearch = (e: Event): void => {
      const detail = (e as CustomEvent<{ value?: string }>).detail
      query.set(detail?.value ?? '')
    }
    const setPack = (id: PackId | 'all'): void => { activePack.set(id) }

    // A card is visible when it matches BOTH the active pack AND the
    // search substring (case-insensitive, on the tag). Reads both signals.
    const visible = (c: ComponentEntry): boolean => {
      const q = query().trim().toLowerCase()
      const packOk = activePack() === 'all' || c.pack === activePack()
      const qOk = q === '' || c.tag.toLowerCase().includes(q)
      return packOk && qOk
    }

    // An L1 element is visible when it matches the search substring on its
    // tag. The L1 section is always rendered (no pack chip) — the search
    // box is the only filter, mirroring how the language surface is
    // conceptually independent of the pack taxonomy.
    const visibleL1 = (e: LanguageElement): boolean => {
      const q = query().trim().toLowerCase()
      return q === '' || e.tag.toLowerCase().includes(q) || e.label.toLowerCase().includes(q)
    }

    // Decorate each entry with a reactive `hidden` derived so the template can
    // toggle card visibility via the zero-arg `c.hidden()` read (a `visible(c)`
    // call with an argument is rejected in renderable expressions). The list
    // itself is stable — only each card's `hidden` flips — so the one-shot
    // preview mount is never disturbed by filtering.
    const cardsByTag = new Map<string, CardEntry>()
    for (const e of ENTRIES) {
      cardsByTag.set(e.tag, { ...e, hidden: derived<boolean>(() => !visible(e)) })
    }
    // Section entries use the SAME CardEntry instances (so each card's
    // `hidden` derived is shared between the flat list and its section) —
    // the bento iterates `s.entries`, and `attr:hidden` reads the SAME
    // derived either way.
    const sections: PackSection[] = SECTION_SHELLS.map((s) => ({
      ...s,
      entries: ENTRIES.filter((e) => e.pack === s.id).map((e) => cardsByTag.get(e.tag)!),
    }))

    return {
      components: derived<CardEntry[]>(() => Array.from(cardsByTag.values())),
      chips: derived<PackChip[]>(() => CHIPS),
      sections: derived<PackSection[]>(() => sections),
      l1Groups: derived(() => L1_GROUPS),
      totalComponents: derived<number>(() => ENTRIES.length),
      totalL1: derived<number>(() => L1_LANGUAGE_ELEMENTS.length),
      totalPacks: derived<number>(() => SECTION_SHELLS.length),
      visibleCount: derived<number>(() => ENTRIES.filter((c) => visible(c)).length),
      visibleL1Count: derived<number>(() => L1_LANGUAGE_ELEMENTS.filter((e) => visibleL1(e)).length),
      query,
      activePack,
      setPack,
      onSearch,
    }
  },

  mount(_state: unknown, host: HTMLElement) {
    // One-shot preview mount: for each rendered [data-preview-for] stage, mount
    // a single bare instance of the registered custom element (or L1 bare tag).
    // Seeded from PREVIEW_SEEDS (props via setAttribute, default-slot innerHTML
    // via the runtime's `bindHtml` — the morph-backed binding, NOT a raw
    // innerHTML write). Filtering later toggles card `hidden`; mounted
    // previews persist.
    //
    // `<sui-theme-picker>` is the one primitive whose mount has document-level
    // side effects (`applyTheme()` writes data-theme on documentElement); it
    // exposes a `scope` prop to isolate those writes — set `scope="preview"` so
    // the catalog never flips the document theme on load.
    // Apply a single per-card drive directive. The gallery only runs the
    // triggers that change the resting state into the "interesting" state
    // (attrs, error event, optional trigger button) — clicks, hovers, and
    // methods are reserved for the detail page (where the user is expected
    // to interact). The "interesting" state is what the cell needs to be
    // visually informative, applied once.
    const applyDrive = (stage: HTMLElement, tag: string, host: HTMLElement): void => {
      const seed = PREVIEW_SEEDS[tag]
      const drive = seed?.drive
      if (!drive) return
      const el = host.shadowRoot
        ? (host.shadowRoot.querySelector(`[data-preview-for="${tag}"] > *`) as HTMLElement | null)
        : (stage.firstElementChild as HTMLElement | null)
      if (!el) return
      if (drive.attrs) for (const [k, v] of Object.entries(drive.attrs)) el.setAttribute(k, v)
      if (drive.error) {
        // Dispatch a platform ErrorEvent on the boundary host. The boundary
        // catches errors thrown from its children by listening for the
        // standard `error` event on the host; the fallback then swaps in.
        const errType = drive.error.type ?? 'Error'
        const Ctor = (globalThis as unknown as Record<string, unknown>)[errType] as
          | (new (m: string) => Error)
          | undefined
        const err = Ctor ? new Ctor(drive.error.message ?? 'Demo error') : new Error(drive.error.message ?? 'Demo error')
        el.dispatchEvent(new ErrorEvent('error', { error: err, bubbles: true, composed: true }))
      }
      if (drive.triggerButton) {
        // Inject a behavioral trigger next to the stage (NOT inside — the
        // stage's overflow:hidden would clip the button). The button lives
        // next to the card-link and is removed on unmount.
        const btn = document.createElement('button')
        btn.className = 'preview-trigger-btn'
        btn.type = 'button'
        btn.textContent = drive.triggerButton.label
        btn.setAttribute('data-test', `preview-trigger-${tag}`)
        btn.addEventListener('click', () => {
          if (drive.triggerButton!.action === 'throw') {
            const errType = drive.error?.type ?? 'Error'
            const Ctor = (globalThis as unknown as Record<string, unknown>)[errType] as
              | (new (m: string) => Error)
              | undefined
            const err = Ctor ? new Ctor(drive.error?.message ?? 'Demo error') : new Error(drive.error?.message ?? 'Demo error')
            el.dispatchEvent(new ErrorEvent('error', { error: err, bubbles: true, composed: true }))
          }
        })
        stage.parentElement?.appendChild(btn)
        mounted.push(btn)
      }
    }
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
        // L1 language element — the tag is a bare keyword, NOT a custom
        // element. Mount a real <tag> with the seed's innerHTML, no shadow
        // boundary to cross. Falls through to the custom-element path on a
        // miss.
        const l1 = L1_LANGUAGE_ELEMENTS.find((e) => e.tag === tag)
        if (l1) {
          try {
            const el = document.createElement(l1.tag)
            if (l1.attrs) {
              for (const [k, v] of Object.entries(l1.attrs)) el.setAttribute(k, v)
            }
            stage.appendChild(el)
            if (l1.innerHTML) binds.push(bindHtml(el, () => l1.innerHTML))
            stage.dataset.previewState = 'live'
            mounted.push(el)
          } catch {
            const note = document.createElement('span')
            note.className = 'preview-empty'
            note.textContent = 'unavailable'
            stage.appendChild(note)
            stage.dataset.previewState = 'unavailable'
            mounted.push(note)
          }
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
          // Apply the resting-state drive (attrs, error event) — done at
          // mount, NOT a click-driven interaction. The cell needs to look
          // "interesting" immediately, not after a user action.
          applyDrive(stage, tag, host)
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
