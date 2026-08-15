// /components — the catalog the browser filters over.
//
// L1 hub tiles are family hosts from @ship/sail-l1 (`hosts()` / FAMILIES).
// Parts (tab, tab-list, tab-panel, legend, …) are not tiles.
// `layout` is keywords, not an L1 family — kept as a sidecar tile.
// Pack rows stay hand-listed. Detail routes still read component-docs/*/meta.ts.

import { copyOf, hosts } from '@ship/sail-l1'
import { docMeta } from '../component-docs/registry'

export type PackId =
  | 'lang'
  | 'charts'
  | 'media'
  | 'canvas'
  | 'chat'
  | 'mermaid'
  | 'code-editor'

export interface Pack {
  id: PackId | 'all'
  label: string
  /** The install line — empty for the language itself. */
  sub: string
}

export interface CatalogEntry {
  /** The element tag. The action catalog lives in the action table
   *   (`crates/sail/compiler/src/action_manifest.rs`) and is not shown
   *   here — actions are not elements. */
  tag: string
  /** The scope key page.sail reads visibility through (`show.<key>()`). */
  key: string
  /** The /components/<slug> detail route. Every tile has one. */
  slug: string
  pack: PackId
  cat:
    | 'inputs'
    | 'display'
    | 'feedback'
    | 'overlay'
    | 'navigation'
    | 'data'
    | 'layout'
    | 'effects'
}

export const PACKS: Pack[] = [
  { id: 'all', label: 'All packs', sub: '' },
  // L1 surface — bare language tags that compile in (button, input, …).
  // Product name: "Component primitives" (was "In the language" / stdlib).
  { id: 'lang', label: 'Component primitives', sub: 'no import — it compiles in' },
  { id: 'charts', label: 'Charts', sub: '@ship/sail-components/charts' },
  { id: 'media', label: 'Media', sub: '@ship/sail-components/media' },
  // Canvas is deliberately absent from the rail: its pack has zero
  // cataloged components until their demo tiles land (see the CATALOG
  // note) — a rail entry whose filter shows an empty grid is a lie.
  { id: 'chat', label: 'Chat elements', sub: '@ship/sail-components/chat' },
  { id: 'mermaid', label: 'Mermaid', sub: '@ship/sail-components/mermaid' },
  { id: 'code-editor', label: 'Code editor', sub: '@ship/sail-components/code' },
  // The fx pack is gone: the actions it shipped (`spotlight`,
  // `targetCursor`) are component-primitive actions in the action table;
  // they are not in this catalog because actions are not elements.
]

const L = (tag: string, key: string, cat: CatalogEntry['cat']): CatalogEntry => ({
  tag,
  key,
  slug: tag,
  pack: 'lang',
  cat,
})

/** Pack-component helper. `key` is the camelCase scope key page.sail reads
 *  through `show.<key>()`, `slug` is the /components/<slug> detail route.
 *  Every pack tag is the custom element the @ship/sail-* packs define; the
 *  helper keeps pack rows consistent with the `L` ones. */
const P = (
  pack: Exclude<PackId, 'lang'>,
  tag: string,
  key: string,
  cat: CatalogEntry['cat'],
): CatalogEntry => ({
  tag,
  key,
  slug: tag,
  pack,
  cat,
})

function kebabToCamel(tag: string): string {
  if (tag === 'a') return 'a'
  return tag.replace(/-([a-z])/g, (_, c: string) => c.toUpperCase())
}

/** Shared menu cell — family host, not a hub tile (lives on dropdown / context-menu). */
const HUB_SKIP = new Set(['menu-item'])

/** Lang rows: one hub tile per L1 family host. Copy is optional. */
const LANG_CATALOG: CatalogEntry[] = hosts()
  .filter((family) => !HUB_SKIP.has(family.host))
  .map((family) => {
    const tag = family.host
    const meta = docMeta(tag)
    const copy = copyOf(tag)
    return L(tag, meta?.key ?? kebabToCamel(tag), copy?.category ?? meta?.category ?? 'display')
  })

/** Keywords tile — not an L1 family. */
const LAYOUT_TILE: CatalogEntry = L('layout', 'layout', 'layout')

export const CATALOG: CatalogEntry[] = [
  // ── Component primitives — SSOT: @ship/sail-l1 hosts() ───────────
  ...LANG_CATALOG,
  LAYOUT_TILE,
  // Actions (`spotlight`, `targetCursor`, etc.) are stdlib and live in
  // the action table. They are not elements; this catalog is
  // elements only. Legend is documented only inside the fieldset package.

  // ── @ship/sail-components/charts — 7 SVG-rendered data viz primitives ───────────
  P('charts', 'sail-area-chart', 'sailAreaChart', 'data'),
  P('charts', 'sail-bar-chart', 'sailBarChart', 'data'),
  P('charts', 'sail-donut', 'sailDonut', 'data'),
  P('charts', 'sail-gauge', 'sailGauge', 'data'),
  P('charts', 'sail-heatmap', 'sailHeatmap', 'data'),
  P('charts', 'sail-line-chart', 'sailLineChart', 'data'),
  P('charts', 'sail-sparkline', 'sailSparkline', 'data'),

  // ── @ship/sail-components/media — content renderers (image / audio-video) ────────
  // sail-media-csv exists in the pack but has NO live tile in page.sail yet —
  // a catalog entry REQUIRES a live tile (`show.<key>()` gate), or the
  // rail's counts lie and its filter shows an empty grid. Add the entry
  // WITH its demo, never before.
  P('media', 'sail-media-image', 'sailMediaImage', 'data'),
  P('media', 'sail-media-av', 'sailMediaAv', 'data'),

  // ── @ship/sail-components/canvas — NOT CATALOGED YET ─────────────────────────────
  // The pack ships drawing/annotation surfaces (sail-canvas, sail-board,
  // sail-draw, sail-graph-canvas, sail-annotation-*), floor-migrated and
  // CEM-green — but none has a live demo tile in page.sail, so listing them
  // would make the Canvas filter empty. Enter the catalog only with a tile.

  // ── @ship/sail-components/chat — chat-message UI primitives ─────────────
  // sail-message-list: pack component without a live tile — same rule as
  // sail-media-csv above; enters with its demo.
  P('chat', 'sail-message', 'sailMessage', 'display'),
  P('chat', 'sail-citation', 'sailCitation', 'display'),
  P('chat', 'sail-composer', 'sailComposer', 'inputs'),
  P('chat', 'sail-thinking-dots', 'sailThinkingDots', 'feedback'),
  P('chat', 'sail-tool-call', 'sailToolCall', 'display'),

  // ── @ship/sail-components/mermaid — 1 diagram renderer (themed SVG) ──────────────
  P('mermaid', 'sail-mermaid', 'sailMermaid', 'data'),

  // ── @ship/sail-components/code — 1 CodeMirror-wrapped editor ──────────────
  P('code-editor', 'sail-code-editor', 'sailCodeEditor', 'inputs'),
]

export const CATEGORIES = [
  'all',
  'inputs',
  'display',
  'feedback',
  'overlay',
  'navigation',
  'data',
  'layout',
  'effects',
] as const

export function packCount(id: Pack['id']): number {
  return id === 'all' ? CATALOG.length : CATALOG.filter((c) => c.pack === id).length
}
