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
// exactly that, per tag, so every specimen that CAN render inline renders
// something representative of a real design system.
//
// TWO delivery channels (mirroring how the host applies a seed):
//   • `props`     — set via `setAttribute` BEFORE the element connects, so a
//                   controller's SSR/declarative fallback (the `rows=` /
//                   `nodes=` / `events=` / `data=` / `value=` string props)
//                   sees them at `connectedCallback`.
//   • `innerHTML` — light-DOM children projected into the shadow slots,
//                   applied via the runtime's `bindHtml` (morph-backed — NOT a
//                   raw innerHTML write, which sui-controller-purity forbids).
//
// Data-driven note: the compiler's `<prop>` vocabulary is `string | number |
// bool | union` — `type="json"` does not compile — so structured data (table
// rows, tree nodes, calendar events, a spreadsheet workbook) arrives as a JSON
// STRING on a scalar prop and is `JSON.parse`d in the component's decode step.
// Every parser here is total (bad JSON → `[]`), so a malformed seed degrades to
// empty rather than throwing a console error.

export interface PreviewSeed {
  props?: Record<string, string>
  innerHTML?: string
}

export const PREVIEW_SEEDS: Record<string, PreviewSeed> = {
  // ── Actions & identity ─────────────────────────────────────────────────────
  'sui-button':       { props: { tone: 'primary', variant: 'solid' }, innerHTML: 'Save changes' },
  'sui-link':         { props: { href: '#', variant: 'underline' }, innerHTML: 'View documentation' },
  'sui-icon':         { props: { name: 'sparkles', variant: 'tile', tone: 'primary', size: 'lg' } },
  'sui-avatar':       { props: { name: 'Ada Lovelace', size: 'lg' } },
  'sui-account-chip': { props: { name: 'Ada Lovelace' } },
  'sui-account-menu': { props: { status: 'signed-in', name: 'Ada Lovelace', email: 'ada@ship.dev', plan: 'Pro', provider: 'GitHub' } },
  'sui-brand-mark':   { props: { label: 'Ship', size: 'lg' } },
  'sui-badge':        { props: { tone: 'success', variant: 'soft' }, innerHTML: 'Passing' },
  'sui-chip':         { props: { removable: 'true', tone: 'primary' }, innerHTML: 'TypeScript' },
  'sui-status-dot':   { props: { tone: 'success', label: 'Operational', pulse: 'true' } },

  // ── Feedback & status ──────────────────────────────────────────────────────
  'sui-alert':        { props: { tone: 'success' }, innerHTML: '<sui-icon slot="icon" name="check-circle"></sui-icon><span slot="title">Deploy succeeded</span>All 24 checks passed on <code>job/site-demos</code>.' },
  'sui-toast':        { props: { tone: 'success' }, innerHTML: '<span slot="title">Changes saved</span>Your deployment is now live.' },
  'sui-spinner':      { props: { label: 'Loading…' } },
  'sui-skeleton':     {},
  'sui-separator':    {},
  'sui-empty-state':  { innerHTML: '<sui-icon slot="icon" name="inbox"></sui-icon><span slot="title">No deployments yet</span>Ship your first build and it will appear here.<sui-button slot="action" tone="primary">New deployment</sui-button>' },
  'sui-error-state':  { props: { title: 'Service unavailable', body: 'The upstream service did not respond in time.', code: '502' } },
  'sui-error-boundary': { innerHTML: '<div>Protected subtree — its children render here unless one throws.</div>' },

  // ── Overlays with an inline trigger (renders the trigger; opens on interaction) ─
  'sui-tooltip':      { innerHTML: '<sui-button variant="outline">Hover for help</sui-button><span slot="content">Extra context appears in the bubble.</span>' },
  'sui-popover':      { innerHTML: '<sui-button slot="trigger" variant="outline">Open popover</sui-button><div>Popover body — rich content anchored to the trigger.</div>' },
  'sui-hover-card':   { innerHTML: '<sui-button slot="trigger" variant="ghost">@ada-lovelace</sui-button><div><strong>Ada Lovelace</strong><br/>First programmer · Analytical Engine</div>' },
  'sui-dropdown-menu':{ innerHTML: '<sui-button slot="trigger" variant="outline">Actions ▾</sui-button><button>Edit</button><button>Duplicate</button><button>Delete</button>' },
  'sui-context-menu': { innerHTML: '<sui-button slot="trigger" variant="outline">Right-click me</sui-button><button>Cut</button><button>Copy</button><button>Paste</button>' },
  'sui-collapsible':  { innerHTML: '<sui-button slot="trigger" variant="ghost">Show advanced settings</sui-button><div>Region revealed when the trigger is toggled.</div>' },

  // ── Modal overlays (portal to the top layer on `open`; seeded for that state) ─
  'sui-dialog':       { innerHTML: '<h2 slot="header">Confirm deployment</h2><p>Deploy <code>job/site-demos</code> to production?</p><div slot="footer"><sui-button variant="ghost">Cancel</sui-button><sui-button tone="primary">Deploy</sui-button></div>' },
  'sui-alert-dialog': { props: { tone: 'destructive', 'confirm-label': 'Delete', 'cancel-label': 'Keep' }, innerHTML: '<span slot="title">Delete project?</span>This permanently removes the project and every deployment under it.' },
  'sui-drawer':       { props: { side: 'right' }, innerHTML: '<span slot="title">Filters</span><span slot="description">Refine the component list</span><p>Drawer body content.</p>' },
  'sui-cmdk':         { props: { placeholder: 'Type a command…', items: '[{"id":"new","label":"New deployment","icon":"rocket","group":"Actions"},{"id":"search","label":"Search components","icon":"search","group":"Actions"},{"id":"docs","label":"Open docs","icon":"book","group":"Navigate"},{"id":"settings","label":"Settings","icon":"settings","group":"Navigate"}]' } },

  // ── Forms ──────────────────────────────────────────────────────────────────
  'sui-input':        { props: { value: 'Ada Lovelace', label: 'Full name', placeholder: 'Enter your name' } },
  'sui-textarea':     { props: { label: 'Release notes', rows: '3', value: 'Enriched the /components catalog with representative demo data for every primitive.' } },
  'sui-checkbox':     { props: { checked: 'true' }, innerHTML: 'Enable automatic deploys' },
  'sui-switch':       { props: { checked: 'true' }, innerHTML: 'Dark mode' },
  'sui-slider':       { props: { value: '64', min: '0', max: '100', label: 'Volume' } },
  'sui-radio-group':  { props: { value: 'card', label: 'Payment method' }, innerHTML: '<sui-radio value="card">Credit card</sui-radio><sui-radio value="paypal">PayPal</sui-radio><sui-radio value="wire">Wire transfer</sui-radio>' },
  'sui-toggle-group': { props: { value: 'grid' }, innerHTML: '<sui-toggle value="list">List</sui-toggle><sui-toggle value="grid">Grid</sui-toggle><sui-toggle value="board">Board</sui-toggle>' },
  'sui-select':       { props: { value: 'prod', label: 'Environment' }, innerHTML: '<option value="dev">Development</option><option value="staging">Staging</option><option value="prod">Production</option>' },
  'sui-combobox':     { props: { placeholder: 'Choose a framework', options: '[{"value":"react","label":"React"},{"value":"vue","label":"Vue"},{"value":"svelte","label":"Svelte"},{"value":"solid","label":"SolidJS"},{"value":"qwik","label":"Qwik"}]' } },
  'sui-autocomplete': { props: { placeholder: 'Search commands…', suggestions: '["deploy","deploy --prod","deploy --preview","rollback","rollback --to"]' } },
  'sui-field':        { innerHTML: '<sui-label slot="label">Email address</sui-label><sui-input type="email" value="ada@ship.dev"></sui-input><span slot="hint">We\'ll never share your email.</span>' },
  'sui-label':        { props: { required: 'true' }, innerHTML: 'Email address' },
  'sui-date-picker':  { props: { value: '2026-07-15', label: 'Ship date' } },
  'sui-color-picker': { props: { value: '#4f46e5' } },
  'sui-file-upload':  { props: { accept: '.png,.jpg,.pdf', multiple: 'true' }, innerHTML: 'Drag files here or click to browse' },

  // ── Structure & layout ─────────────────────────────────────────────────────
  'sui-card':         { props: { 'accent-bar': 'static', tone: 'primary' }, innerHTML: '<div slot="header"><strong>Deployment #1284</strong></div><p>Finished in 42s across 24 checks.</p><div slot="footer"><sui-badge tone="success">Live</sui-badge></div>' },
  'sui-accordion':    { innerHTML: '<sui-accordion-item open><span slot="trigger">What is Ship?</span>Ship compiles <code>.sui</code> templates to standard web components.</sui-accordion-item><sui-accordion-item><span slot="trigger">How do I theme it?</span>Every value is a design token — set a theme and every primitive follows.</sui-accordion-item><sui-accordion-item><span slot="trigger">Is it accessible?</span>ARIA roles and keyboard nav are built into every primitive.</sui-accordion-item>' },
  'sui-tabs':         { props: { active: 'overview' }, innerHTML: '<sui-tab name="overview">Overview</sui-tab><sui-tab name="activity" count="3">Activity</sui-tab><sui-tab name="settings">Settings</sui-tab><sui-tab-panel name="overview" active>A live overview of the deployment.</sui-tab-panel><sui-tab-panel name="activity">3 new events since your last visit.</sui-tab-panel><sui-tab-panel name="settings">Configure build &amp; deploy options.</sui-tab-panel>' },
  'sui-tab-panel':    { props: { name: 'overview', active: 'true' }, innerHTML: 'Standalone tab-panel content.' },
  'sui-timeline':     { innerHTML: '<sui-timeline-item at="09:30" tone="success"><span slot="title">Build passed</span>24 checks green in 42s.</sui-timeline-item><sui-timeline-item at="09:45" tone="info"><span slot="title">Deployed to staging</span>job/site-demos → staging.</sui-timeline-item><sui-timeline-item at="10:02"><span slot="title">Promoted to production</span>Live for all users.</sui-timeline-item>' },
  'sui-scroll-area':  { props: { size: 'sm' }, innerHTML: '<div>sui-button</div><div>sui-card</div><div>sui-input</div><div>sui-select</div><div>sui-table</div><div>sui-tree</div><div>sui-calendar</div><div>sui-dialog</div><div>sui-tabs</div><div>sui-timeline</div><div>sui-accordion</div><div>sui-tooltip</div>' },
  'sui-stack':        { innerHTML: '<sui-badge tone="success">Live</sui-badge><sui-badge tone="info">Beta</sui-badge><sui-badge tone="warning">Deprecated</sui-badge>' },
  'sui-cluster':      { innerHTML: '<sui-badge tone="success">Live</sui-badge><sui-badge tone="info">Beta</sui-badge><sui-badge tone="warning">Deprecated</sui-badge>' },
  'sui-center':       { innerHTML: '<sui-badge tone="primary">Centered</sui-badge>' },
  'sui-grid':         { innerHTML: '<sui-card>One</sui-card><sui-card>Two</sui-card><sui-card>Three</sui-card>' },

  // ── Navigation ─────────────────────────────────────────────────────────────
  'sui-nav-top':          { innerHTML: '<sui-brand-mark slot="brand" label="Ship"></sui-brand-mark><a href="#">Docs</a><a href="#">Components</a><a href="#">Pricing</a><sui-button slot="actions" tone="primary" size="sm">Sign in</sui-button>' },
  'sui-navigation-menu':  { props: { value: 'components' }, innerHTML: '<sui-navigation-item value="docs">Docs</sui-navigation-item><sui-navigation-item value="components">Components</sui-navigation-item><sui-navigation-item value="guides">Guides</sui-navigation-item>' },
  'sui-pagination':       { props: { page: '3', total: '248', 'page-size': '20' } },
  'sui-project-switcher': { props: { 'active-id': 'sui', projects: '[{"id":"ship","name":"Ship","path":"~/dev/ship"},{"id":"sui","name":"SUI Design System","path":"~/dev/sui"},{"id":"deck","name":"Deck","path":"~/dev/deck"}]' } },

  // ── Data display (JSON-string scalar props — the SSR/declarative path) ──────
  'sui-table':      { props: { variant: 'data', label: 'Components', columns: '[{"field":"name","label":"Component"},{"field":"pack","label":"Pack"},{"field":"status","label":"Status"}]', rows: '[{"id":"1","name":"sui-button","pack":"primitives","status":"stable"},{"id":"2","name":"sui-table","pack":"primitives","status":"stable"},{"id":"3","name":"sui-calendar","pack":"primitives","status":"beta"},{"id":"4","name":"sui-render","pack":"viewer","status":"beta"}]' } },
  'sui-tree':       { props: { label: 'Files', nodes: '[{"id":"src","label":"src","children":[{"id":"components","label":"components","children":[{"id":"button","label":"sui-button.sui"},{"id":"card","label":"sui-card.sui"}]},{"id":"index","label":"index.ts"}]},{"id":"pkg","label":"package.json"}]' } },
  'sui-tree-grid':  { props: { label: 'Files', columns: '[{"field":"label","label":"Name"},{"field":"kind","label":"Kind"},{"field":"size","label":"Size"}]', nodes: '[{"id":"src","label":"src","kind":"dir","size":"—","children":[{"id":"btn","label":"sui-button.sui","kind":"file","size":"2.4 KB"},{"id":"ctl","label":"sui-button.controller.ts","kind":"file","size":"1.1 KB"}]},{"id":"pkg","label":"package.json","kind":"file","size":"0.6 KB"}]' } },
  'sui-tree-view':  { innerHTML: '<sui-tree-item label="src" expanded><sui-tree-item label="components" expanded><sui-tree-item label="sui-button.sui" leaf></sui-tree-item><sui-tree-item label="sui-card.sui" leaf></sui-tree-item></sui-tree-item><sui-tree-item label="index.ts" leaf></sui-tree-item></sui-tree-item>' },
  'sui-calendar':   { props: { view: 'month', 'focus-date': '2026-07-15', events: '[{"id":"1","title":"Design review","allDay":true,"start":"2026-07-08"},{"id":"2","title":"Ship 0.3 launch","allDay":true,"start":"2026-07-15"},{"id":"3","title":"Standup","start":"2026-07-15T09:30","end":"2026-07-15T10:00"},{"id":"4","title":"Retro","start":"2026-07-22T15:00","end":"2026-07-22T16:00"}]' } },
  'sui-json-view':  { props: { expanded: 'true', data: '{"name":"@ship/sui-primitives","version":"0.2.0","private":false,"exports":{"button":"./sui-button","card":"./sui-card"},"keywords":["ui","web-components"]}' } },
  'sui-sheet':      { props: { label: 'Q3 Budget', workbook: '{"sheets":[{"id":"q3","name":"Q3 Budget","rowCount":5,"colCount":4,"cells":{"A1":{"value":"Item"},"B1":{"value":"Q1"},"C1":{"value":"Q2"},"D1":{"value":"Q3"},"A2":{"value":"Revenue"},"B2":{"value":120},"C2":{"value":145},"D2":{"value":168},"A3":{"value":"Costs"},"B3":{"value":80},"C3":{"value":92},"D3":{"value":101},"A4":{"value":"Profit"},"B4":{"value":40,"formula":"=B2-B3"},"C4":{"value":53,"formula":"=C2-C3"},"D4":{"value":67,"formula":"=D2-D3"}}}]}' } },

  // ── Code & content (scalar `value`/`from`/`to` props — NOT slotted innerHTML) ─
  'sui-code':        { props: { block: 'true', lang: 'js' }, innerHTML: 'function sum(a, b) {\n  return a + b\n}' },
  'sui-code-editor': { props: { language: 'html', value: '<prop name="tone" values="default|primary" default="default" />\n\n<button class="btn btn-{tone()}">\n  <slot></slot>\n</button>' } },
  'sui-markdown':    { props: { value: '# Ship SUI\n\n**Batteries-included** components for the web.\n\n- Compiles to standard web components\n- Themed with design tokens\n- `sui-compiler build` in the loop' } },
  'sui-diff':        { props: { from: "const tone = 'primary'\nconst size = 'md'", to: "const tone = 'success'\nconst size = 'lg'\nconst variant = 'solid'" } },

  // ── Theme picker (scope=preview isolates its document-level applyTheme write) ─
  'sui-theme-picker': { props: { scope: 'preview' } },
}
