// Shared demo seeds for the /components surfaces.
//
// ONE source of truth consumed by BOTH:
//   • <sui-component-catalog>  — the /components gallery grid (one bare
//     specimen mounted per card).
//   • /components/[tag]         — the per-component detail page (one big
//     live preview stage).
//
// A bare `document.createElement(tag)` renders nothing meaningful for most
// primitives: they need default props, slotted children, OR — for the
// data-driven primitives — a JSON string on a scalar prop. This map supplies
// exactly that, per tag, so every catalog cell renders something
// representative of a real design system.
//
// Curated for the public surface: every tag visible in TAG_CATEGORY
// (minus HIDDEN_TAGS) has a seed entry here. HIDDEN_TAGS includes the
// composed slots (sui-accordion-item / sui-tab / sui-tab-panel / sui-
// tree-item / sui-timeline-item / sui-toast[-container] / sui-radio),
// the JS-prop-only viewer primitives (sui-render / sui-source-panel /
// sui-prop-knobs / sui-prop-matrix / sui-component-viewer), AND brand
//
// TWO delivery channels (mirroring how the host applies a seed):
//   • `props`     — set via `setAttribute` BEFORE the element connects, so a
//                   controller's SSR/declarative fallback (the `rows=` /
//                   `nodes=` / `events=` / `data=` / `value=` string props)
//                   sees them at `connectedCallback`.
//   • `innerHTML` — light-DOM children projected into the shadow slots,
//                   applied via the runtime's `bindHtml` (morph-backed — NOT
//                   a raw innerHTML write, which sui-controller-purity
//                   forbids).
//
// Data-driven note: the compiler's `<prop>` vocabulary is `string | number |
// bool | union` — `type="json"` does not compile — so structured data (table
// rows, tree nodes, calendar events, a spreadsheet workbook) arrives as a
// JSON STRING on a scalar prop and is `JSON.parse`d in the component's
// decode step. Every parser here is total (bad JSON → `[]`), so a malformed
// seed degrades to empty rather than throwing a console error.
//
// "Flattering default" rule: every seed ships the component in its
// representative / on-brand state, not its unstyled default. A bare button
// (default tone, foreground-only text on a low-contrast background) is the
// worst foot forward we can put in a gallery — `tone="primary"` + a real
// label lifts every cell. Same principle: badges get semantic tones
// (success / warning / destructive), inputs get filled labels with hint
// text, overlays get a substantive trigger, charts ship realistic data.

export interface PreviewSeed {
  props?: Record<string, string>
  innerHTML?: string
}

// Tiny sample-data helpers — keep the seeds readable. Strings only; the
// decoder in each primitive JSON.parses them.
const SMALL_TREE_NODES = JSON.stringify([
  { id: 'src', label: 'src', children: [
    { id: 'components', label: 'components', children: [
      { id: 'button', label: 'sui-button.sui' },
      { id: 'card',    label: 'sui-card.sui' },
    ] },
    { id: 'index', label: 'index.ts' },
  ] },
  { id: 'pkg', label: 'package.json' },
])

const PIPELINE_RUN_DATA = JSON.stringify([
  { id: '1', name: 'sui-button',  pack: 'primitives', status: 'stable' },
  { id: '2', name: 'sui-card',    pack: 'primitives', status: 'stable' },
  { id: '3', name: 'sui-calendar',pack: 'primitives', status: 'beta'   },
  { id: '4', name: 'sui-render',  pack: 'viewer',     status: 'beta'   },
])

const SPARKLINE_DATA = JSON.stringify([12, 18, 14, 22, 19, 30, 26, 33, 28, 36, 41, 38])

const BAR_CHART_DATA = JSON.stringify([
  { label: 'Mon', value: 12 }, { label: 'Tue', value: 18 },
  { label: 'Wed', value: 9  }, { label: 'Thu', value: 22 },
  { label: 'Fri', value: 16 },
])

const LINE_CHART_SERIES = JSON.stringify([
  { label: 'Shipped', values: [3, 5, 7, 6, 9, 12, 14] },
  { label: 'Blocked', values: [2, 1, 3, 2, 1, 0, 1] },
])
const LINE_CHART_LABELS = JSON.stringify(['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'])

const HEATMAP_DATA = JSON.stringify([
  [3, 5, 2, 7, 4],
  [6, 8, 4, 5, 9],
  [2, 3, 6, 4, 5],
  [4, 6, 5, 8, 7],
])

const PROJECT_LIST = JSON.stringify([
  { id: 'ship', name: 'Ship', path: '~/dev/ship' },
  { id: 'sui',  name: 'SUI Design System', path: '~/dev/sui'  },
  { id: 'deck', name: 'Deck', path: '~/dev/deck' },
])

const CALENDAR_EVENTS = JSON.stringify([
  { id: '1', title: 'Design review', allDay: true,  start: '2026-07-08' },
  { id: '2', title: 'Ship 0.3 launch', allDay: true,  start: '2026-07-15' },
])

const TOAST_ITEMS = JSON.stringify([
  { id: 'new', label: 'New deployment', icon: 'rocket',  group: 'Actions'   },
  { id: 'docs', label: 'Open docs',    icon: 'book',    group: 'Navigate'  },
])

export const PREVIEW_SEEDS: Record<string, PreviewSeed> = {
  // ── Actions & identity ──────────────────────────────────────────────
  'sui-button':          { props: { tone: 'primary', variant: 'solid' }, innerHTML: 'Save changes' },
  'sui-link':            { props: { href: '#', variant: 'underline' }, innerHTML: 'View documentation' },
  'sui-icon':            { props: { name: 'sparkles', variant: 'tile', tone: 'primary', size: 'lg' } },
  'sui-avatar':          { props: { name: 'Ada Lovelace', size: 'lg' } },
  // catalog (brand chrome, not a public component) and is composed
  'sui-chip':            { props: { removable: 'true', tone: 'primary' }, innerHTML: 'TypeScript' },
  'sui-status-dot':      { props: { tone: 'success', label: 'Operational', pulse: 'true' } },

  // ── Feedback & status ───────────────────────────────────────────────
  // sui-alert ships filled-success: the canonical "all systems go" cell.
  'sui-alert':           { props: { tone: 'success' }, innerHTML: '<sui-icon slot="icon" name="check-circle"></sui-icon><span slot="title">Deploy succeeded</span>All 24 checks passed on <code>job/site-demos</code>.' },
  // sui-toast-host: standalone mount shows an empty stack; the cell is
  // honestly content-light by design (the host routes `ship:notify`
  // events into a container program API; the gallery can't drive that
  // and the description on the card explains the contract).
  'sui-toast-host':      { props: { position: 'bottom-right' } },
  'sui-spinner':         { props: { label: 'Loading…' } },
  'sui-skeleton':        {},
  'sui-empty-state':     { innerHTML: '<sui-icon slot="icon" name="inbox"></sui-icon><span slot="title">No deployments yet</span>Ship your first build and it will appear here.<sui-button slot="action" tone="primary">New deployment</sui-button>' },
  'sui-error-state':     { props: { title: 'Service unavailable', body: 'The upstream service did not respond in time.', code: '502' } },
  'sui-error-boundary':  { innerHTML: '<div>Protected subtree — its children render here unless one throws.</div>' },

  // ── Overlays with an inline trigger ────────────────────────────────
  // Each overlay is seeded with a real-looking trigger; the cells render
  // the trigger only (the overlay opens on user interaction).
  'sui-tooltip':         { innerHTML: '<sui-button variant="outline">Hover for help</sui-button><span slot="content">Extra context appears in the bubble.</span>' },
  'sui-popover':         { innerHTML: '<sui-button slot="trigger" variant="outline">Open popover</sui-button><div>Popover body — rich content anchored to the trigger.</div>' },
  'sui-hover-card':      { innerHTML: '<sui-button slot="trigger" variant="ghost">@ada-lovelace</sui-button><div><strong>Ada Lovelace</strong><br/>First programmer · Analytical Engine</div>' },
  'sui-dropdown-menu':   { innerHTML: '<sui-button slot="trigger" variant="outline">Actions ▾</sui-button><button>Edit</button><button>Duplicate</button><button>Delete</button>' },
  'sui-context-menu':    { innerHTML: '<sui-button slot="trigger" variant="outline">Right-click me</sui-button><button>Cut</button><button>Copy</button><button>Paste</button>' },
  'sui-collapsible':     { innerHTML: '<sui-button slot="trigger" variant="ghost">Show advanced settings</sui-button><div>Region revealed when the trigger is toggled.</div>' },

  // ── Modal overlays (portal to the top layer on `open`; seeded for that state)
  'sui-dialog':          { innerHTML: '<h2 slot="header">Confirm deployment</h2><p>Deploy <code>job/site-demos</code> to production?</p><div slot="footer"><sui-button variant="ghost">Cancel</sui-button><sui-button tone="primary">Deploy</sui-button></div>' },
  'sui-alert-dialog':    { props: { tone: 'destructive', 'confirm-label': 'Delete', 'cancel-label': 'Keep' }, innerHTML: '<span slot="title">Delete project?</span>This permanently removes the project and every deployment under it.' },
  'sui-drawer':          { props: { side: 'right' }, innerHTML: '<span slot="title">Filters</span><span slot="description">Refine the component list</span><p>Drawer body content.</p>' },
  'sui-cmdk':            { props: { placeholder: 'Type a command…', items: JSON.stringify(TOAST_ITEMS) } },
  // sui-portal: teleports children; an empty stage at the host looks
  // empty. Honest emptiness here is the right cell — the description on
  // the card explains the contract.
  'sui-portal':          {},

  // ── Forms ───────────────────────────────────────────────────────────
  'sui-input':           { props: { value: 'Ada Lovelace', label: 'Full name', placeholder: 'Enter your name' } },
  'sui-textarea':        { props: { label: 'Release notes', rows: '3', value: 'Enriched the /components catalog with representative demo data for every primitive.' } },
  // Checked + a label makes the box visibly checked and labeled — not an
  // empty unchecked default.
  'sui-checkbox':        { props: { checked: 'true' }, innerHTML: 'Enable automatic deploys' },
  'sui-switch':          { props: { checked: 'true' }, innerHTML: 'Dark mode' },
  'sui-slider':          { props: { value: '64', min: '0', max: '100', label: 'Volume' } },
  'sui-radio-group':     { props: { value: 'card', label: 'Payment method' }, innerHTML: '<sui-radio value="card">Credit card</sui-radio><sui-radio value="paypal">PayPal</sui-radio><sui-radio value="wire">Wire transfer</sui-radio>' },
  'sui-toggle-group':    { props: { value: 'grid' }, innerHTML: '<sui-toggle value="list">List</sui-toggle><sui-toggle value="grid">Grid</sui-toggle><sui-toggle value="board">Board</sui-toggle>' },
  'sui-select':          { props: { value: 'prod', label: 'Environment' }, innerHTML: '<option value="dev">Development</option><option value="staging">Staging</option><option value="prod">Production</option>' },
  'sui-combobox':        { props: { placeholder: 'Choose a framework', options: '[{"value":"react","label":"React"},{"value":"vue","label":"Vue"},{"value":"svelte","label":"Svelte"},{"value":"solid","label":"SolidJS"},{"value":"qwik","label":"Qwik"}]' } },
  'sui-autocomplete':    { props: { placeholder: 'Search commands…', suggestions: '["deploy","deploy --prod","deploy --preview","rollback","rollback --to"]' } },
  'sui-field':           { innerHTML: '<sui-label slot="label">Email address</sui-label><sui-input type="email" value="ada@ship.dev"></sui-input><span slot="hint">We\'ll never share your email.</span>' },
  'sui-label':           { props: { required: 'true' }, innerHTML: 'Email address' },
  'sui-date-picker':     { props: { value: '2026-07-15', label: 'Ship date' } },
  'sui-color-picker':    { props: { value: '#4f46e5' } },
  'sui-file-upload':     { props: { accept: '.png,.jpg,.pdf', multiple: 'true' }, innerHTML: 'Drag files here or click to browse' },

  // ── Structure & layout ──────────────────────────────────────────────
  'sui-card':            { props: { 'accent-bar': 'static', tone: 'primary' }, innerHTML: '<div slot="header"><strong>Deployment #1284</strong></div><p>Finished in 42s across 24 checks.</p><div slot="footer"><span tone="success" sx="display:inline-flex min-h:control-h-2xs px:2 text:xs weight:semibold radius:sm bg:tone-bg border:tone-border fg:tone-fg">Live</span></div>' },
  'sui-accordion':       { innerHTML: '<sui-accordion-item open><span slot="trigger">What is Ship?</span>Ship compiles <code>.sui</code> templates to standard web components.</sui-accordion-item><sui-accordion-item><span slot="trigger">How do I theme it?</span>Every value is a design token — set a theme and every primitive follows.</sui-accordion-item><sui-accordion-item><span slot="trigger">Is it accessible?</span>ARIA roles and keyboard nav are built into every primitive.</sui-accordion-item>' },
  'sui-tabs':            { props: { active: 'overview' }, innerHTML: '<sui-tab name="overview">Overview</sui-tab><sui-tab name="activity" count="3">Activity</sui-tab><sui-tab name="settings">Settings</sui-tab><sui-tab-panel name="overview" active>A live overview of the deployment.</sui-tab-panel><sui-tab-panel name="activity">3 new events since your last visit.</sui-tab-panel><sui-tab-panel name="settings">Configure build &amp; deploy options.</sui-tab-panel>' },
  'sui-timeline':        { innerHTML: '<sui-timeline-item at="09:30" tone="success"><span slot="title">Build passed</span>24 checks green in 42s.</sui-timeline-item><sui-timeline-item at="09:45" tone="info"><span slot="title">Deployed to staging</span>job/site-demos → staging.</sui-timeline-item><sui-timeline-item at="10:02"><span slot="title">Promoted to production</span>Live for all users.</sui-timeline-item>' },
  'sui-scroll-area':     { props: { size: 'sm' }, innerHTML: '<div>sui-button</div><div>sui-card</div><div>sui-input</div><div>sui-select</div><div>sui-table</div><div>sui-tree</div><div>sui-calendar</div><div>sui-dialog</div><div>sui-tabs</div><div>sui-timeline</div><div>sui-accordion</div><div>sui-tooltip</div>' },
  'sui-center':          { innerHTML: '<span tone="primary" sx="display:inline-flex min-h:control-h-2xs px:2 text:xs weight:semibold radius:sm bg:tone-bg border:tone-border fg:tone-fg">Centered</span>' },
  'sui-grid':            { innerHTML: '<sui-card>One</sui-card><sui-card>Two</sui-card><sui-card>Three</sui-card>' },
  'sui-separator':       {},

  // ── Navigation ──────────────────────────────────────────────────────
  'sui-navigation-menu': { props: { value: 'components' }, innerHTML: '<sui-navigation-item value="docs">Docs</sui-navigation-item><sui-navigation-item value="components">Components</sui-navigation-item><sui-navigation-item value="guides">Guides</sui-navigation-item>' },
  'sui-pagination':      { props: { page: '3', total: '248', 'page-size': '20' } },

  // ── Data display (JSON-string scalar props — the SSR/declarative path)
  // Table: realistic project-status rows, not 1 column of empty cells.
  'sui-table':           { props: { variant: 'data', label: 'Components', columns: '[{"field":"name","label":"Component"},{"field":"pack","label":"Pack"},{"field":"status","label":"Status"}]', rows: PIPELINE_RUN_DATA } },
  'sui-tree':            { props: { label: 'Files', nodes: SMALL_TREE_NODES } },
  'sui-tree-grid':       { props: { label: 'Files', columns: '[{"field":"label","label":"Name"},{"field":"kind","label":"Kind"},{"field":"size","label":"Size"}]', nodes: SMALL_TREE_NODES } },
  'sui-tree-view':       { innerHTML: '<sui-tree-item label="src" expanded><sui-tree-item label="components" expanded><sui-tree-item label="sui-button.sui" leaf></sui-tree-item><sui-tree-item label="sui-card.sui" leaf></sui-tree-item></sui-tree-item><sui-tree-item label="index.ts" leaf></sui-tree-item></sui-tree-item>' },
  'sui-sheet':           { props: { label: 'Q3 Budget', workbook: '{"sheets":[{"id":"q3","name":"Q3 Budget","rowCount":5,"colCount":4,"cells":{"A1":{"value":"Item"},"B1":{"value":"Q1"},"C1":{"value":"Q2"},"D1":{"value":"Q3"},"A2":{"value":"Revenue"},"B2":{"value":120},"C2":{"value":145},"D2":{"value":168},"A3":{"value":"Costs"},"B3":{"value":80},"C3":{"value":92},"D3":{"value":101},"A4":{"value":"Profit"},"B4":{"value":40,"formula":"=B2-B3"},"C4":{"value":53,"formula":"=C2-C3"},"D4":{"value":67,"formula":"=D2-D3"}}}]}' } },

  // ── Code & content ──────────────────────────────────────────────────
  'sui-code':            { props: { block: 'true', lang: 'js' }, innerHTML: 'function sum(a, b) {\n  return a + b\n}' },
  'sui-code-editor':     { props: { language: 'html', value: '<prop name="tone" values="default|primary" default="default" />\n\n<button class="btn btn-{tone()}">\n  <slot></slot>\n</button>' } },
  'sui-markdown':        { props: { value: '# Ship SUI\n\n**Batteries-included** components for the web.\n\n- Compiles to standard web components\n- Themed with design tokens\n- `sui-compiler build` in the loop' } },
  'sui-diff':            { props: { from: "const tone = 'primary'\nconst size = 'md'", to: "const tone = 'success'\nconst size = 'lg'\nconst variant = 'solid'" } },

  // ── Theme picker ────────────────────────────────────────────────────
  // Scope=preview isolates the document-level applyTheme write — same
  // harness as before, but now the cell renders the picker itself.
  'sui-theme-picker':    { props: { scope: 'preview' } },

  // ── Charts — all chart seeds ship with a small dataset ──────────────
  'sui-area-chart':      { props: { width: '240', height: '120', series: '[{"label":"Shipped","values":[3,5,7,6,9,12,14]}]', labels: LINE_CHART_LABELS, fill: 'true' } },
  'sui-bar-chart':       { props: { width: '240', height: '120', data: BAR_CHART_DATA, tone: 'primary' } },
  'sui-donut':           { props: { size: '120', thickness: '16', data: '[{"label":"Shipped","value":42},{"label":"In review","value":18},{"label":"Blocked","value":7}]' } },
  'sui-gauge':           { props: { value: '68', tone: 'success', size: '120', thickness: '12' } },
  'sui-heatmap':         { props: { 'cell-size': '18', gap: '3', rows: '["Mon","Tue","Wed","Thu","Fri"]', cols: '["9","10","11","12","13"]', data: HEATMAP_DATA, tone: 'primary' } },
  'sui-line-chart':      { props: { width: '240', height: '120', series: LINE_CHART_SERIES, labels: LINE_CHART_LABELS, points: 'true' } },
  'sui-sparkline':       { props: { width: '120', height: '32', data: SPARKLINE_DATA, tone: 'primary', fill: 'true' } },

  // ── Canvas (board / canvas / draw / graph-canvas / annotations) ─────
  // Canvas family — a base sheet with brand grid + one shape; the bare
  // mount shows the editor chrome on top of a tokenized backdrop.
  'sui-board':           { props: { theme: 'light', grid: 'dots', value: '{"nodes":[{"id":"a","x":60,"y":80},{"id":"b","x":160,"y":80},{"id":"c","x":110,"y":150}],"edges":[{"from":"a","to":"b"},{"from":"a","to":"c"}]}' } },
  'sui-canvas':          { props: { theme: 'light', grid: 'dots' } },
  'sui-draw':            { props: { theme: 'light', grid: 'dots', tool: 'pen' } },
  'sui-graph-canvas':    { props: { width: '240', height: '160', layout: 'positioned', nodes: '[{"id":"a","x":40,"y":40},{"id":"b","x":120,"y":40},{"id":"c","x":80,"y":110}]', edges: '[{"from":"a","to":"b"},{"from":"a","to":"c"}]' } },
  // sui-annotation-layer: enable point mode so the crosshair overlay is
  // visible (mode=off is the no-UI default and renders invisibly).
  'sui-annotation-layer': { innerHTML: '<div style="padding:24px;border:1px dashed var(--border);border-radius:var(--radius-md);color:var(--muted-foreground);font:inherit;font-size:var(--text-sm);">Annotate this region</div>' },
  // sui-annotation-list: default state is "No staged annotations" — a
  // useful demo of the empty-state rather than a blank cell.
  'sui-annotation-list': {},
  // annotation-marker is a single-position pin; one inside-anchor demo.
  'sui-annotation-marker': { props: { kind: 'point', x: '40', y: '40', active: 'true' } },

  // ── Chat elements ───────────────────────────────────────────────────
  // Author each chat primitive in its representative conversational state.
  'sui-citation':        { props: { source: 'docs/components.md', index: '1' }, innerHTML: '<sui-icon slot="icon" name="bookmark"></sui-icon> From the SUI design system reference.' },
  'sui-composer':        { props: { placeholder: 'Message the agent…' } },
  'sui-message':         { props: { role: 'assistant', timestamp: '09:42', status: 'sent' }, innerHTML: '<p>Reviewed the <code>job/site-demos</code> deploy. 24 of 24 checks passed in 42s — ready to promote.</p>' },
  'sui-message-list':    { innerHTML: '<sui-message role="user" timestamp="09:41" status="sent"><p>Review the production build?</p></sui-message><sui-message role="assistant" timestamp="09:42" status="sent"><p>Reviewed — green across the board.</p></sui-message>' },
  'sui-thinking-dots':   { props: { label: 'Reviewing the build…', tone: 'primary' } },
  'sui-tool-call':       { props: { name: 'createDeployment', status: 'ok' }, innerHTML: '<code>site-demos</code> · <span>commit a7f31c · main · 42s</span>' },

  // ── Media ───────────────────────────────────────────────────────────
  // sui-media-av and sui-media-image both need `prop:src` (a JS-prop
  // seam the canvas-mount path can't supply) to render any frames. The
  // cells are empty by design; the card description explains the contract.
  // Only sui-media-csv produces an inline-rendered preview (CSV text on
  // its `text` attribute).
  'sui-media-av':        { props: { kind: 'video', controls: 'true' } },
  'sui-media-csv':       { props: { header: 'true', text: 'name,pack,status\nsui-button,primitives,stable\nsui-card,primitives,stable\nsui-render,viewer,beta' } },
  'sui-media-image':     { props: { alt: 'A tiny gradient swatch', fit: 'cover', zoomable: 'true' } },
}
