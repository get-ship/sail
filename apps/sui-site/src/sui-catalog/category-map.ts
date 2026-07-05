// Category taxonomy for the public SUI primitive catalog (apps/sui-site
// /components page).
//
// ONE source of truth for two questions:
//   1. Which category a public primitive belongs to (TAG → Category).
//   2. Which tags are NOT public catalog entries — the internal sub-primitives
//      that compose but never appear standalone on /components
//      (`HIDDEN_TAGS`).
//
// Why both lists live here:
//   - The mapping is deliberately a `Record<string, CategoryId>` — a static
//     lookup table keyed by tag (the compiler-built CEM produces the very
//     same set of string keys).
//   - The HIDDEN_TAGS set is kept right next to its parent entries'
//     assignments so a reviewer can flip an entry from `hidden` to a
//     category in one place — no second list to keep in sync across files.
//   - A primitive missing the map falls into CategoryId.Utility so the
//     catalog never silently drops an unknown tag (intentional under-claim,
//     not over-claim of curation).

// ── Categories ───────────────────────────────────────────────────────────────

/** The eight fold-categories for the public primitive surface. Display order
 *  matches the chip row; `Utility` is the fallback bucket. */
export type CategoryId =
  | 'forms'
  | 'layout'
  | 'overlays'
  | 'navigation'
  | 'feedback'
  | 'data'
  | 'typography'
  | 'utility'

export const CATEGORY_LABELS: Record<CategoryId, string> = {
  forms: 'Forms & Inputs',
  layout: 'Layout',
  overlays: 'Overlays',
  navigation: 'Navigation',
  feedback: 'Feedback',
  data: 'Data Display',
  typography: 'Typography',
  utility: 'Utility',
}

export const CATEGORY_ORDER: readonly CategoryId[] = [
  'forms',
  'layout',
  'overlays',
  'navigation',
  'feedback',
  'data',
  'typography',
  'utility',
] as const

// ── Hidden sub-primitives ────────────────────────────────────────────────────

/** Tags that compose a parent primitive but are NOT standalone catalog
 *  entries. The composed parent (e.g. `<sui-accordion-item>` rolls up into
 *  `<sui-accordion>`; `<sui-tab-panel>` into `<sui-tabs>`) is the public API.
 *  Review/edit this list in one place. */
export const HIDDEN_TAGS: Record<string, true> = {
  // accordion — the item is internal; the parent is the API
  'sui-accordion-item': true,
  // tabs — tab + panel are slots of `<sui-tabs>`
  'sui-tab': true,
  'sui-tab-panel': true,
  // tree — item is a slot of `<sui-tree>`
  'sui-tree-item': true,
  // timeline — item is a slot of `<sui-timeline>`
  'sui-timeline-item': true,
  // toast — the host is the public API; toast + container are internals
  'sui-toast': true,
  'sui-toast-container': true,
  // toast-host-as-one-entry — the host is the entry; drop the duplicate name
  // by also hiding any `sui-toast-host` collision with the public toast group.
  // (Kept named for parity — no collision exists today.)
  'sui-radio': true, // radio-group is the entry; radio is its slot
}

// ── Public tag → category mapping ───────────────────────────────────────────

/** Categorization for every public primitive. Organized by pack for review;
 *  the runtime only cares about the flat Record. Tags not in the map fall to
 *  `CategoryId.Utility`. */
export const TAG_CATEGORY: Record<string, CategoryId> = {
  // ── @ship/sui-primitives — Forms & Inputs ─────────────────────────────
  'sui-input':           'forms',
  'sui-textarea':        'forms',
  'sui-checkbox':        'forms',
  'sui-switch':          'forms',
  'sui-radio-group':     'forms',
  'sui-toggle-group':    'forms',
  'sui-select':          'forms',
  'sui-combobox':        'forms',
  'sui-autocomplete':    'forms',
  'sui-field':           'forms',
  'sui-label':           'forms',
  'sui-date-picker':     'forms',
  'sui-color-picker':    'forms',
  'sui-file-upload':     'forms',
  'sui-slider':          'forms',

  // ── @ship/sui-primitives — Layout ─────────────────────────────────────
  'sui-card':            'layout',
  'sui-accordion':       'layout',
  'sui-tabs':            'layout',
  'sui-timeline':        'layout',
  'sui-scroll-area':     'layout',
  'sui-center':          'layout',
  'sui-grid':            'layout',
  'sui-separator':       'layout',
  'sui-collapsible':     'layout',

  // ── @ship/sui-primitives — Overlays ───────────────────────────────────
  'sui-tooltip':         'overlays',
  'sui-popover':         'overlays',
  'sui-hover-card':      'overlays',
  'sui-dropdown-menu':   'overlays',
  'sui-context-menu':    'overlays',
  'sui-dialog':          'overlays',
  'sui-alert-dialog':    'overlays',
  'sui-drawer':          'overlays',
  'sui-cmdk':            'overlays',
  'sui-portal':          'overlays',

  // ── @ship/sui-primitives — Navigation ─────────────────────────────────
  'sui-nav-top':         'navigation',
  'sui-navigation-menu': 'navigation',
  'sui-pagination':      'navigation',
  'sui-project-switcher':'navigation',

  // ── @ship/sui-primitives — Feedback ───────────────────────────────────
  'sui-alert':           'feedback',
  'sui-toast-host':      'feedback',
  'sui-spinner':         'feedback',
  'sui-skeleton':        'feedback',
  'sui-empty-state':     'feedback',
  'sui-error-state':     'feedback',
  'sui-error-boundary':  'feedback',

  // ── @ship/sui-primitives — Data Display ───────────────────────────────
  'sui-table':           'data',
  'sui-tree':            'data',
  'sui-tree-grid':       'data',
  'sui-tree-view':       'data',
  'sui-calendar':        'data',
  'sui-json-view':       'data',
  'sui-sheet':           'data',

  // ── @ship/sui-primitives — Typography & identity ──────────────────────
  'sui-button':          'typography',
  'sui-link':            'typography',
  'sui-icon':            'typography',
  'sui-avatar':          'typography',
  'sui-account-chip':    'typography',
  'sui-account-menu':    'typography',
  'sui-brand-mark':      'typography',
  'sui-badge':           'typography',
  'sui-chip':            'typography',
  'sui-status-dot':      'typography',

  // ── @ship/sui-primitives — Utility ────────────────────────────────────
  'sui-theme-picker':    'utility',

  // ── @ship/sui-viewer — Data Display (the engine primitives) ──────────
  'sui-render':          'data',
  'sui-source-panel':    'data',
  'sui-prop-knobs':      'data',
  'sui-prop-matrix':     'data',
  'sui-component-viewer':'data',

  // ── @ship/sui-charts — Data Display ───────────────────────────────────
  'sui-area-chart':      'data',
  'sui-bar-chart':       'data',
  'sui-donut':           'data',
  'sui-gauge':           'data',
  'sui-heatmap':         'data',
  'sui-line-chart':      'data',
  'sui-sparkline':       'data',

  // ── @ship/sui-canvas ── Data Display ──────────────────────────────────
  'sui-annotation-layer':'data',
  'sui-annotation-list': 'data',
  'sui-annotation-marker':'data',
  'sui-board':           'data',
  'sui-canvas':          'data',
  'sui-draw':            'data',
  'sui-graph-canvas':    'data',

  // ── @ship/sui-code-editor — Typography (content authoring) ────────────
  'sui-code-editor':     'typography',

  // ── @ship/sui-chat-elements — Data Display ───────────────────────────
  'sui-citation':        'data',
  'sui-composer':        'data',
  'sui-message':         'data',
  'sui-message-list':    'data',
  'sui-thinking-dots':   'data',
  'sui-tool-call':       'data',

  // ── @ship/sui-media — Data Display ────────────────────────────────────
  'sui-media-av':        'data',
  'sui-media-csv':       'data',
  'sui-media-image':     'data',

  // ── @ship/sui-mermaid — Data Display ──────────────────────────────────
  'sui-mermaid':         'data',

  // ── @ship/sui-code — Typography ───────────────────────────────────────
  'sui-code':            'typography',
  'sui-diff':            'typography',
  'sui-markdown':        'typography',
}
