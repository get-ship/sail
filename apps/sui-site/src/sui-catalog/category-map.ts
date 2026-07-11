// Pack taxonomy for the SUI component catalog (apps/sui-site /components).
//
// ONE source of truth for two questions:
//   1. Which PACK a component belongs to (TAG → PackId). The packs are the
//      eight npm workspace packages under packages/components/sui/ — the
//      shipping boundary, not an invented UI taxonomy. The gallery groups
//      by pack because the pack boundary is what the operator reasons about
//      ("I need a charts pack component", "I need a media pack component").
//   2. Which tags are NOT public catalog entries — the internal sub-primitives
//      that compose but never appear standalone on /components
//      (`HIDDEN_TAGS`).
//
// A primitive missing the pack map falls into PackId.Primitives so the
// catalog never silently drops an unknown tag (intentional under-claim,
// not over-claim of curation).
//
// Language elements (L1) are SEPARATE — they are bare tags the compiler
// lowers (no `sui-` prefix), not custom elements. They live in a dedicated
// `LANGUAGE_ELEMENTS` list rendered in its own section BEFORE the packs.
// See L1_LANGUAGE_ELEMENTS below.

// ── Packs ────────────────────────────────────────────────────────────────────

/** The eight shipping packs. Display order matches the gallery section order
 *  — primitives first (the foundation), then chart/canvas/chat/code/media/
 *  mermaid (sibling domain packs), then viewer (the harness). */
export type PackId =
  | 'primitives'
  | 'charts'
  | 'canvas'
  | 'chat-elements'
  | 'code-editor'
  | 'media'
  | 'mermaid'
  | 'viewer'

export const PACK_LABELS: Record<PackId, string> = {
  primitives: 'Primitives',
  charts: 'Charts',
  canvas: 'Canvas',
  'chat-elements': 'Chat elements',
  'code-editor': 'Code editor',
  media: 'Media',
  mermaid: 'Mermaid',
  viewer: 'Viewer',
}

export const PACK_BLURBS: Record<PackId, string> = {
  primitives: 'The foundation. Forms, layout, overlays, navigation, feedback, typography — every custom element ships here.',
  charts: 'Data viz — area, bar, donut, gauge, heatmap, line, sparkline.',
  canvas: 'Spatial surfaces — board, draw, graph-canvas, annotation layers.',
  'chat-elements': 'Conversational surfaces — composer, message, citation, tool-call, thinking-dots.',
  'code-editor': 'CodeMirror-backed editor with language-aware highlighting.',
  media: 'AV / image / CSV media primitives.',
  mermaid: 'Diagram-as-element with the mermaid grammar.',
  viewer: 'The harness — render, source-panel, prop-knobs, prop-matrix, component-viewer.',
}

export const PACK_ORDER: readonly PackId[] = [
  'primitives',
  'charts',
  'canvas',
  'chat-elements',
  'code-editor',
  'media',
  'mermaid',
  'viewer',
] as const

// ── Hidden sub-primitives ────────────────────────────────────────────────────

/** Tags that compose a parent primitive, are pure scaffolding, or need a
 *  non-attribute prop seam (the JS-prop path) to render meaningfully — none
 *  of them belong in a public gallery cell. The composed parent (e.g.
 *  `<sui-accordion-item>` rolls up into `<sui-accordion>`,
 *  `<sui-tab-panel>` into `<sui-tabs>`) is the public API. Review/edit
 *  this list in one place. */
export const HIDDEN_TAGS: Record<string, true> = {
  // tabs — tab + panel are slots of `<sui-tabs>`
  'sui-tab': true,
  'sui-tab-panel': true,
  // tree — item is a slot of `<sui-tree>`
  'sui-tree-item': true,
  // timeline — item is a slot of `<sui-timeline>`
  'sui-timeline-item': true,
  // toggle-group — toggle is a slot of `<sui-toggle-group>`
  'sui-toggle': true,
  // radio-group — radio is a slot of `<sui-radio-group>`
  'sui-radio': true,
  // navigation-menu — item is a slot of `<sui-navigation-menu>`
  'sui-navigation-item': true,
  // toast family — toast + toast-container are internals; the composed
  // host routes a `ship:notify` event into the container's programmatic
  // `show()` API. The cell preview cannot drive that event from a bare
  // mount, so the cell would always be empty. Keep the host entry visible
  // — its description explains the contract.
  'sui-toast': true,
  'sui-toast-container': true,

  // Viewer scaffolding — these primitives need the JS-prop path
  // (`prop:source` / `prop:compile` set as host JS properties, NOT a
  // `setAttribute` write) to render. A bare mount leaves them empty,
  // which is misleading in a public gallery. The full demo harness lives
  // on the /components/[tag] detail page.
  'sui-render': true,
  'sui-source-panel': true,
  'sui-prop-knobs': true,
  'sui-prop-matrix': true,
  'sui-component-viewer': true,
}

// ── Public tag → pack mapping ────────────────────────────────────────────────

/** Pack membership for every public primitive. The runtime only cares about
 *  the flat Record; the comments below mirror the workspace package layout
 *  for review. Tags not in the map fall to `PackId.Primitives`. */
export const TAG_PACK: Record<string, PackId> = {
  // ── @ship/sui-primitives ─────────────────────────────────────────────
  // (accordion deduped → native <details> L1, see L1_LANGUAGE_ELEMENTS)
  'sui-alert-dialog':       'primitives',
  'sui-autocomplete':       'primitives',
  'sui-cmdk':               'primitives',
  'sui-color-picker':       'primitives',
  'sui-combobox':           'primitives',
  'sui-context-menu':       'primitives',
  'sui-date-picker':        'primitives',
  'sui-dialog':             'primitives',
  'sui-dropdown-menu':      'primitives',
  'sui-error-boundary':     'primitives',
  'sui-file-upload':        'primitives',
  'sui-hover-card':         'primitives',
  'sui-icon':               'primitives',
  'sui-input':              'primitives',
  'sui-json-view':          'primitives',
  'sui-markdown':           'primitives',
  'sui-nav-top':            'primitives',
  'sui-navigation-menu':    'primitives',
  'sui-pagination':         'primitives',
  'sui-popover':            'primitives',
  'sui-portal':             'primitives',
  'sui-radio-group':        'primitives',
  'sui-scroll-area':        'primitives',
  'sui-select':             'primitives',
  'sui-sheet':              'primitives',
  'sui-table':              'primitives',
  'sui-tabs':               'primitives',
  'sui-theme-picker':       'primitives',
  'sui-timeline':           'primitives',
  'sui-toast-host':         'primitives',
  'sui-toggle-group':       'primitives',
  'sui-tree':               'primitives',
  'sui-tree-grid':          'primitives',
  'sui-tree-view':          'primitives',
  'sui-diff':               'primitives',
  'sui-drawer':             'primitives',

  // ── @ship/sui-canvas ────────────────────────────────────────────────
  'sui-annotation-layer':   'canvas',
  'sui-annotation-list':    'canvas',
  'sui-annotation-marker':  'canvas',
  'sui-board':              'canvas',
  'sui-canvas':             'canvas',
  'sui-draw':               'canvas',
  'sui-graph-canvas':       'canvas',

  // ── @ship/sui-charts ────────────────────────────────────────────────
  'sui-area-chart':         'charts',
  'sui-bar-chart':          'charts',
  'sui-donut':              'charts',
  'sui-gauge':              'charts',
  'sui-heatmap':            'charts',
  'sui-line-chart':         'charts',
  'sui-sparkline':          'charts',

  // ── @ship/sui-chat-elements ─────────────────────────────────────────
  'sui-citation':           'chat-elements',
  'sui-composer':           'chat-elements',
  'sui-message':            'chat-elements',
  'sui-message-list':       'chat-elements',
  'sui-thinking-dots':      'chat-elements',
  'sui-tool-call':          'chat-elements',

  // ── @ship/sui-code-editor ───────────────────────────────────────────
  'sui-code-editor':        'code-editor',

  // ── @ship/sui-media ─────────────────────────────────────────────────
  'sui-media-av':           'media',
  'sui-media-csv':          'media',
  'sui-media-image':        'media',

  // ── @ship/sui-mermaid ───────────────────────────────────────────────
  'sui-mermaid':            'mermaid',

  // ── @ship/sui-viewer ────────────────────────────────────────────────
  // All viewer entries are hidden (see HIDDEN_TAGS); this block documents
  // that the pack exists in the workspace but does NOT contribute public
  // gallery cells. The /components/[tag] route still resolves the viewer
  // primitives (the harness) for tooling use.
}

// ── L1 language elements ────────────────────────────────────────────────────

/** A L1 element the compiler lowers — bare `<tag>`, not a custom element.
 *  These are NOT in the CEM; they live in the framework's compiler. The
 *  gallery renders them in a separate "Language elements" section before
 *  the pack sections, so a layout keyword and a behavior component never
 *  look like peers. */
export interface LanguageElement {
  /** The bare tag name (no prefix). The gallery mounts a real instance of
   *  this tag in its preview stage. */
  tag: string
  /** Short label for the card. */
  label: string
  /** Category of the element for sub-grouping. */
  group: 'layout' | 'typography' | 'primitives' | 'forms'
  /** The HTML to project into the element — its representative state.
   *  Optional (default empty string): void elements like `<separator>`
   *  carry no children. */
  innerHTML?: string
  /** Optional attributes the preview mount sets before connect. */
  attrs?: Record<string, string>
}

export const LANGUAGE_GROUPS = [
  { id: 'layout',      label: 'Layout',     blurb: 'Compiler-lowered flex & grid keywords.' },
  { id: 'typography',  label: 'Typography', blurb: 'Text and inline marks, themed by tokens.' },
  { id: 'forms',       label: 'Controls',   blurb: 'Native form & disclosure elements, skinned in @layer sui — sizes/tones from attributes.' },
  { id: 'primitives',  label: 'Stdlib L1',  blurb: 'Foundational surfaces the framework ships.' },
] as const
export type LanguageGroupId = typeof LANGUAGE_GROUPS[number]['id']

/** The full L1 element set. Curated — every entry here corresponds to a
 *  keyword the SUI compiler recognizes. The list is hand-picked, not
 *  auto-generated, so the gallery's L1 section is a curated tour.
 *
 *  Each entry's `innerHTML` is the representative state — a real, recognisable
 *  example of the keyword lowers to. Layout keywords compose with `<card>`
 *  (also L1) for visible content; typography entries use the type scale
 *  their tag lowers to. The composed "showcase" demos above the chip row
 *  are authored in the catalog template. */
export const L1_LANGUAGE_ELEMENTS: LanguageElement[] = [
  // ── Layout — flex & grid keywords lowered at compile time ──
  { tag: 'row',       label: 'row',       group: 'layout',     innerHTML: '<card padding="sm">A</card><card padding="sm">B</card><card padding="sm">C</card>' },
  { tag: 'column',    label: 'column',    group: 'layout',     innerHTML: '<card padding="sm">A</card><card padding="sm">B</card><card padding="sm">C</card>' },
  { tag: 'grid',      label: 'grid',      group: 'layout',     innerHTML: '<card padding="sm">1</card><card padding="sm">2</card><card padding="sm">3</card><card padding="sm">4</card>', attrs: { cols: '2', gap: '2' } },
  { tag: 'center',    label: 'center',    group: 'layout',     innerHTML: '<card padding="sm">centered</card>' },
  { tag: 'stack',     label: 'stack',     group: 'layout',     innerHTML: '<p>line one</p><p>line two</p><p>line three</p>' },

  // ── Typography — text and inline marks, themed by tokens ──
  { tag: 'h1',     label: 'h1',     group: 'typography', innerHTML: 'Display' },
  { tag: 'h2',     label: 'h2',     group: 'typography', innerHTML: 'Heading' },
  { tag: 'h3',     label: 'h3',     group: 'typography', innerHTML: 'Subhead' },
  { tag: 'h4',     label: 'h4',     group: 'typography', innerHTML: 'Title' },
  { tag: 'h5',     label: 'h5',     group: 'typography', innerHTML: 'Eyebrow' },
  { tag: 'h6',     label: 'h6',     group: 'typography', innerHTML: 'Caption' },
  { tag: 'p',      label: 'p',      group: 'typography', innerHTML: 'Body text lowers to a typographic rhythm.' },
  { tag: 'small',  label: 'small',  group: 'typography', innerHTML: 'Hint / metadata' },
  { tag: 'a',      label: 'a',      group: 'typography', innerHTML: 'inline link', attrs: { href: '#' } },
  { tag: 'strong', label: 'strong', group: 'typography', innerHTML: 'strong emphasis' },
  { tag: 'em',     label: 'em',     group: 'typography', innerHTML: 'stress emphasis' },
  { tag: 'code',   label: 'code',   group: 'typography', innerHTML: 'const x = 1' },
  { tag: 'kbd',    label: 'kbd',    group: 'typography', innerHTML: '⌘K' },
  { tag: 'ul',     label: 'ul',     group: 'typography', innerHTML: '<li>one</li><li>two</li><li>three</li>' },
  { tag: 'ol',     label: 'ol',     group: 'typography', innerHTML: '<li>one</li><li>two</li><li>three</li>' },
  { tag: 'li',     label: 'li',     group: 'typography', innerHTML: 'list item' },
  { tag: 'blockquote', label: 'blockquote', group: 'typography', innerHTML: 'A short quote lowers to a typographic block.' },

  // ── Controls — native form & disclosure elements, skinned in @layer sui ──
  // One entry per bare tag (the catalog keys by tag). <input type=checkbox|
  // radio|range> variants need a separate key/id model — a follow-up.
  { tag: 'details',  label: 'accordion',  group: 'forms', innerHTML: '<summary>Accordion header</summary><p>Panel content revealed on open.</p>', attrs: { open: '' } },
  { tag: 'input',    label: 'input',      group: 'forms', attrs: { placeholder: 'you@example.com' } },
  { tag: 'select',   label: 'select',     group: 'forms', innerHTML: '<option>Choose an option</option><option>Second</option>' },
  { tag: 'textarea', label: 'textarea',   group: 'forms', innerHTML: 'Multiline text input.' },
  { tag: 'button',   label: 'button',     group: 'forms', innerHTML: 'Button', attrs: { variant: 'solid', tone: 'primary' } },
  { tag: 'progress', label: 'progress',   group: 'forms', attrs: { value: '0.6' } },
  { tag: 'meter',    label: 'meter',      group: 'forms', attrs: { value: '0.7' } },

  // ── Stdlib L1 — foundational surfaces the framework ships ──
  { tag: 'card',      label: 'card',      group: 'primitives', innerHTML: 'A content surface with tone, padding, and an optional accent bar.' },
  { tag: 'icon',      label: 'icon',      group: 'primitives', attrs: { name: 'sparkles' } },
  { tag: 'label',     label: 'label',     group: 'primitives', innerHTML: 'Field label' },
  { tag: 'separator', label: 'separator', group: 'primitives' },
  { tag: 'spinner',   label: 'spinner',   group: 'primitives' },
]
