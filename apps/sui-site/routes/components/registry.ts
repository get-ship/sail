// /components — the catalog the browser filters over.
//
// The `lang` rows are the FULL L1 surface — the same 48 elements the
// conformance gate (packages/sui/l1-conformance/specs, minus the motion spec)
// renders on every commit. Each row names a tile that page.sui renders LIVE
// (the tile markup is the component, not a picture of it). The pack rail and
// the category chips both key off this list, so a new tile is one row here
// plus its markup in page.sui — and the counts stay honest by construction.

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
   *   (`crates/sui/compiler/src/action_manifest.rs`) and is not shown
   *   here — actions are not elements. */
  tag: string
  /** The scope key page.sui reads visibility through (`show.<key>()`). */
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
  { id: 'lang', label: 'In the language', sub: 'no import — it compiles in' },
  { id: 'charts', label: 'Charts', sub: '@ship/sui-charts' },
  { id: 'media', label: 'Media', sub: '@ship/sui-media' },
  // Canvas is deliberately absent from the rail: its pack has zero
  // cataloged components until their demo tiles land (see the CATALOG
  // note) — a rail entry whose filter shows an empty grid is a lie.
  { id: 'chat', label: 'Chat elements', sub: '@ship/sui-chat-elements' },
  { id: 'mermaid', label: 'Mermaid', sub: '@ship/sui-mermaid' },
  { id: 'code-editor', label: 'Code editor', sub: '@ship/sui-code-editor' },
  // The fx pack is gone: the actions it shipped (`spotlight`,
  // `targetCursor`) are stdlib and live in the action table; they
  // are not in this catalog because actions are not elements.
]

const L = (tag: string, key: string, cat: CatalogEntry['cat']): CatalogEntry => ({
  tag,
  key,
  slug: tag,
  pack: 'lang',
  cat,
})

/** Pack-component helper. `key` is the camelCase scope key page.sui reads
 *  through `show.<key>()`, `slug` is the /components/<slug> detail route.
 *  Every pack tag is the custom element the @ship/sui-* packs define; the
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

export const CATALOG: CatalogEntry[] = [
  // ── In the language: the 48-element L1 surface ─────────────────────
  L('button', 'button', 'inputs'),
  L('input', 'input', 'inputs'),
  L('textarea', 'textarea', 'inputs'),
  L('select', 'select', 'inputs'),
  L('combobox', 'combobox', 'inputs'),
  L('checkbox', 'checkbox', 'inputs'),
  L('radio-group', 'radioGroup', 'inputs'),
  L('switch', 'switchToggle', 'inputs'),
  L('slider', 'slider', 'inputs'),
  L('toggle-group', 'toggleGroup', 'inputs'),
  L('field', 'field', 'inputs'),
  L('label', 'label', 'inputs'),
  L('fieldset', 'fieldset', 'inputs'),
  L('dropzone', 'dropzone', 'inputs'),
  L('theme-picker', 'themePicker', 'inputs'),
  L('alert', 'alert', 'feedback'),
  L('progress', 'progressBar', 'feedback'),
  L('meter', 'meter', 'feedback'),
  L('spinner', 'spinner', 'feedback'),
  L('skeleton', 'skeleton', 'feedback'),
  L('empty-state', 'emptyState', 'feedback'),
  L('error-state', 'errorState', 'feedback'),
  L('error-boundary', 'errorBoundary', 'feedback'),
  L('toast', 'toast', 'feedback'),
  L('badge', 'badge', 'display'),
  L('chip', 'chip', 'display'),
  L('status-dot', 'statusDot', 'display'),
  L('card', 'card', 'display'),
  L('avatar', 'avatar', 'display'),
  L('icon', 'icon', 'display'),
  L('separator', 'separator', 'display'),
  L('hr', 'hrRule', 'display'),
  L('details', 'detailsEl', 'display'),
  L('collapsible', 'collapsible', 'display'),
  L('prose', 'prose', 'display'),
  L('tooltip', 'tooltip', 'overlay'),
  L('hover-card', 'hoverCard', 'overlay'),
  L('popover', 'popover', 'overlay'),
  L('dropdown', 'dropdown', 'overlay'),
  L('context-menu', 'contextMenu', 'overlay'),
  L('dialog', 'dialog', 'overlay'),
  L('tabs', 'tabs', 'navigation'),
  L('breadcrumb', 'breadcrumb', 'navigation'),
  L('pagination', 'pagination', 'navigation'),
  L('navbar', 'navbar', 'navigation'),
  L('table', 'table', 'data'),
  L('layout', 'layout', 'layout'),
  L('scroll-area', 'scrollArea', 'layout'),
  // Actions (`spotlight`, `targetCursor`, etc.) are stdlib and live in
  // the action table. They are not elements; this catalog is
  // elements only.

  // ── @ship/sui-charts — 7 SVG-rendered data viz primitives ───────────
  P('charts', 'sui-area-chart', 'suiAreaChart', 'data'),
  P('charts', 'sui-bar-chart', 'suiBarChart', 'data'),
  P('charts', 'sui-donut', 'suiDonut', 'data'),
  P('charts', 'sui-gauge', 'suiGauge', 'data'),
  P('charts', 'sui-heatmap', 'suiHeatmap', 'data'),
  P('charts', 'sui-line-chart', 'suiLineChart', 'data'),
  P('charts', 'sui-sparkline', 'suiSparkline', 'data'),

  // ── @ship/sui-media — content renderers (image / audio-video) ────────
  // sui-media-csv exists in the pack but has NO live tile in page.sui yet —
  // a catalog entry REQUIRES a live tile (`show.<key>()` gate), or the
  // rail's counts lie and its filter shows an empty grid. Add the entry
  // WITH its demo, never before.
  P('media', 'sui-media-image', 'suiMediaImage', 'data'),
  P('media', 'sui-media-av', 'suiMediaAv', 'data'),

  // ── @ship/sui-canvas — NOT CATALOGED YET ─────────────────────────────
  // The pack ships 7 drawing/annotation surfaces (sui-canvas, sui-board,
  // sui-draw, sui-graph-canvas, sui-annotation-*), floor-migrated and
  // CEM-green — but none has a live demo tile in page.sui, so listing
  // them made the Canvas filter an empty lie (owner-reported 2026-07-16).
  // They enter this catalog with the pack-consolidation wave (task #53),
  // each entry landing together with its tile.

  // ── @ship/sui-chat-elements — chat-message UI primitives ─────────────
  // sui-message-list: pack component without a live tile — same rule as
  // sui-media-csv above; enters with its demo.
  P('chat', 'sui-message', 'suiMessage', 'display'),
  P('chat', 'sui-citation', 'suiCitation', 'display'),
  P('chat', 'sui-composer', 'suiComposer', 'inputs'),
  P('chat', 'sui-thinking-dots', 'suiThinkingDots', 'feedback'),
  P('chat', 'sui-tool-call', 'suiToolCall', 'display'),

  // ── @ship/sui-mermaid — 1 diagram renderer (themed SVG) ──────────────
  P('mermaid', 'sui-mermaid', 'suiMermaid', 'data'),

  // ── @ship/sui-code-editor — 1 CodeMirror-wrapped editor ──────────────
  P('code-editor', 'sui-code-editor', 'suiCodeEditor', 'inputs'),
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
