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
  | 'fx'

export interface Pack {
  id: PackId | 'all'
  label: string
  /** The install line — empty for the language itself. */
  sub: string
  /** Floor-gated on this branch: the pack's components still carry raw
   *  `<style>` blocks, which this compiler rejects — so they cannot mount
   *  live, and this page refuses to fake them with drawings. The rail still
   *  lists the pack; selecting it explains the gap. */
  gated?: boolean
  /** How many components are waiting behind the gate (for the copy). */
  waiting?: number
}

export interface CatalogEntry {
  /** The element tag (or action name, for @ship/sui-fx). */
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
  { id: 'charts', label: 'Charts', sub: '@ship/sui-charts', gated: true, waiting: 7 },
  { id: 'media', label: 'Media', sub: '@ship/sui-media', gated: true, waiting: 3 },
  { id: 'canvas', label: 'Canvas', sub: '@ship/sui-canvas', gated: true, waiting: 5 },
  { id: 'chat', label: 'Chat elements', sub: '@ship/sui-chat-elements', gated: true, waiting: 6 },
  { id: 'mermaid', label: 'Mermaid', sub: '@ship/sui-mermaid', gated: true, waiting: 1 },
  { id: 'code-editor', label: 'Code editor', sub: '@ship/sui-code-editor', gated: true, waiting: 1 },
  { id: 'fx', label: 'FX', sub: '@ship/sui-fx' },
]

const L = (tag: string, key: string, cat: CatalogEntry['cat']): CatalogEntry => ({
  tag,
  key,
  slug: tag,
  pack: 'lang',
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
  // ── @ship/sui-fx (actions, not elements — pure TS, so they clear the
  //    floor and run live; the .sui packs above are gated, see PACKS) ──
  { tag: 'use:spotlight', key: 'fxSpotlight', slug: 'spotlight', pack: 'fx', cat: 'effects' },
  { tag: 'use:dotGrid', key: 'fxDotGrid', slug: 'dot-grid', pack: 'fx', cat: 'effects' },
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
