// /components — the catalog the browser filters over.
//
// The `lang` rows are the FULL L1 component-primitives surface — the same 48
// elements the conformance gate (packages/sail/l1-conformance/specs, minus
// the motion spec) renders on every commit. Featured tiles on page.sail
// render LIVE; catalog tiles mix static (takumi/l1) previews with a detail
// link. The pack rail and category chips both key off this list.

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
  { id: 'charts', label: 'Charts', sub: '@ship/sail-charts' },
  { id: 'media', label: 'Media', sub: '@ship/sail-media' },
  // Canvas is deliberately absent from the rail: its pack has zero
  // cataloged components until their demo tiles land (see the CATALOG
  // note) — a rail entry whose filter shows an empty grid is a lie.
  { id: 'chat', label: 'Chat elements', sub: '@ship/sail-chat-elements' },
  { id: 'mermaid', label: 'Mermaid', sub: '@ship/sail-mermaid' },
  { id: 'code-editor', label: 'Code editor', sub: '@ship/sail-code-editor' },
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

export const CATALOG: CatalogEntry[] = [
  // ── Component primitives: the 48-element L1 surface ────────────────
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
  // separator IS the themed <hr> (stdlib lowers to native hr). Do not catalog bare `hr`.
  L('separator', 'separator', 'display'),
  L('kbd', 'kbd', 'display'),
  L('accordion', 'accordion', 'display'),
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
  L('ticker', 'ticker', 'effects'),
  // Actions (`spotlight`, `targetCursor`, etc.) are stdlib and live in
  // the action table. They are not elements; this catalog is
  // elements only.

  // ── @ship/sail-charts — 7 SVG-rendered data viz primitives ───────────
  P('charts', 'sail-area-chart', 'sailAreaChart', 'data'),
  P('charts', 'sail-bar-chart', 'sailBarChart', 'data'),
  P('charts', 'sail-donut', 'sailDonut', 'data'),
  P('charts', 'sail-gauge', 'sailGauge', 'data'),
  P('charts', 'sail-heatmap', 'sailHeatmap', 'data'),
  P('charts', 'sail-line-chart', 'sailLineChart', 'data'),
  P('charts', 'sail-sparkline', 'sailSparkline', 'data'),

  // ── @ship/sail-media — content renderers (image / audio-video) ────────
  // sail-media-csv exists in the pack but has NO live tile in page.sail yet —
  // a catalog entry REQUIRES a live tile (`show.<key>()` gate), or the
  // rail's counts lie and its filter shows an empty grid. Add the entry
  // WITH its demo, never before.
  P('media', 'sail-media-image', 'sailMediaImage', 'data'),
  P('media', 'sail-media-av', 'sailMediaAv', 'data'),

  // ── @ship/sail-canvas — NOT CATALOGED YET ─────────────────────────────
  // The pack ships drawing/annotation surfaces (sail-canvas, sail-board,
  // sail-draw, sail-graph-canvas, sail-annotation-*), floor-migrated and
  // CEM-green — but none has a live demo tile in page.sail, so listing them
  // would make the Canvas filter empty. Enter the catalog only with a tile.

  // ── @ship/sail-chat-elements — chat-message UI primitives ─────────────
  // sail-message-list: pack component without a live tile — same rule as
  // sail-media-csv above; enters with its demo.
  P('chat', 'sail-message', 'sailMessage', 'display'),
  P('chat', 'sail-citation', 'sailCitation', 'display'),
  P('chat', 'sail-composer', 'sailComposer', 'inputs'),
  P('chat', 'sail-thinking-dots', 'sailThinkingDots', 'feedback'),
  P('chat', 'sail-tool-call', 'sailToolCall', 'display'),

  // ── @ship/sail-mermaid — 1 diagram renderer (themed SVG) ──────────────
  P('mermaid', 'sail-mermaid', 'sailMermaid', 'data'),

  // ── @ship/sail-code-editor — 1 CodeMirror-wrapped editor ──────────────
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
