// /components — the catalog the browser filters over.
//
// Hand-curated on purpose: each row names a tile that page.sui renders LIVE
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
  pack: PackId
  cat: 'inputs' | 'display' | 'feedback' | 'overlay' | 'navigation' | 'data' | 'effects'
  /** Has a curated /components/[tag] detail page. */
  detail?: boolean
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

export const CATALOG: CatalogEntry[] = [
  // ── In the language ────────────────────────────────────────────────
  { tag: 'button', key: 'button', pack: 'lang', cat: 'inputs', detail: true },
  { tag: 'input', key: 'input', pack: 'lang', cat: 'inputs' },
  { tag: 'textarea', key: 'textarea', pack: 'lang', cat: 'inputs' },
  { tag: 'select', key: 'select', pack: 'lang', cat: 'inputs' },
  { tag: 'combobox', key: 'combobox', pack: 'lang', cat: 'inputs', detail: true },
  { tag: 'checkbox', key: 'checkbox', pack: 'lang', cat: 'inputs' },
  { tag: 'radio-group', key: 'radioGroup', pack: 'lang', cat: 'inputs' },
  { tag: 'switch', key: 'switchToggle', pack: 'lang', cat: 'inputs', detail: true },
  { tag: 'slider', key: 'slider', pack: 'lang', cat: 'inputs', detail: true },
  { tag: 'toggle-group', key: 'toggleGroup', pack: 'lang', cat: 'inputs' },
  { tag: 'field', key: 'field', pack: 'lang', cat: 'inputs' },
  { tag: 'dropzone', key: 'dropzone', pack: 'lang', cat: 'inputs' },
  { tag: 'alert', key: 'alert', pack: 'lang', cat: 'feedback', detail: true },
  { tag: 'progress', key: 'progressBar', pack: 'lang', cat: 'feedback' },
  { tag: 'meter', key: 'meter', pack: 'lang', cat: 'feedback' },
  { tag: 'spinner', key: 'spinner', pack: 'lang', cat: 'feedback' },
  { tag: 'skeleton', key: 'skeleton', pack: 'lang', cat: 'feedback' },
  { tag: 'empty-state', key: 'emptyState', pack: 'lang', cat: 'feedback' },
  { tag: 'error-state', key: 'errorState', pack: 'lang', cat: 'feedback' },
  { tag: 'badge', key: 'badge', pack: 'lang', cat: 'display', detail: true },
  { tag: 'chip', key: 'chip', pack: 'lang', cat: 'display' },
  { tag: 'status-dot', key: 'statusDot', pack: 'lang', cat: 'display' },
  { tag: 'card', key: 'card', pack: 'lang', cat: 'display', detail: true },
  { tag: 'avatar', key: 'avatar', pack: 'lang', cat: 'display' },
  { tag: 'icon', key: 'icon', pack: 'lang', cat: 'display' },
  { tag: 'separator', key: 'separator', pack: 'lang', cat: 'display' },
  { tag: 'details', key: 'detailsEl', pack: 'lang', cat: 'display' },
  { tag: 'collapsible', key: 'collapsible', pack: 'lang', cat: 'display' },
  { tag: 'tooltip', key: 'tooltip', pack: 'lang', cat: 'overlay' },
  { tag: 'hover-card', key: 'hoverCard', pack: 'lang', cat: 'overlay' },
  { tag: 'popover', key: 'popover', pack: 'lang', cat: 'overlay' },
  { tag: 'dropdown', key: 'dropdown', pack: 'lang', cat: 'overlay' },
  { tag: 'dialog', key: 'dialog', pack: 'lang', cat: 'overlay', detail: true },
  { tag: 'tabs', key: 'tabs', pack: 'lang', cat: 'navigation', detail: true },
  { tag: 'breadcrumb', key: 'breadcrumb', pack: 'lang', cat: 'navigation' },
  { tag: 'pagination', key: 'pagination', pack: 'lang', cat: 'navigation' },
  { tag: 'table', key: 'table', pack: 'lang', cat: 'data', detail: true },
  { tag: 'code-block', key: 'codeBlock', pack: 'lang', cat: 'data' },
  // ── @ship/sui-fx (actions, not elements — pure TS, so they clear the
  //    floor and run live; the .sui packs above are gated, see PACKS) ──
  { tag: 'use:spotlight', key: 'fxSpotlight', pack: 'fx', cat: 'effects' },
  { tag: 'use:dotGrid', key: 'fxDotGrid', pack: 'fx', cat: 'effects' },
]

export const CATEGORIES = [
  'all',
  'inputs',
  'display',
  'feedback',
  'overlay',
  'navigation',
  'data',
  'effects',
] as const

export function packCount(id: Pack['id']): number {
  return id === 'all' ? CATALOG.length : CATALOG.filter((c) => c.pack === id).length
}
