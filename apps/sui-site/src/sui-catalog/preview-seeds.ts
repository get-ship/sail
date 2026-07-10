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

/** How to drive a component into its "interesting" visible state after mount.
 *  Overlays are closed at rest and disclosure needs a toggle, so the DETAIL
 *  ("viewer") surface — which must prove the REAL component renders — applies
 *  this post-mount. The stage establishes a containing block (transform/
 *  contain), so an opened fixed/portaled overlay is trapped inside the stage
 *  instead of covering the page. */
export interface DriveDirective {
  /** Boolean/string attributes to set after mount (e.g. `{ open: 'true' }`). */
  attrs?: Record<string, string>
  /** Zero-arg host methods to call after mount (e.g. `['show']`). */
  methods?: string[]
  /** Click the first element matching this selector (searched light + shadow).
   *  Dispatches a full pointer sequence and retries across a few frames, since
   *  some overlays wire their trigger listener only after connect. */
  click?: string
  /** Hover the first element matching this selector (pointerenter/over) — for
   *  tooltip / hover-card that open on hover, not click. */
  hover?: string
  /** Dispatch an ErrorEvent on the host after mount. Used by `sui-error-boundary`
   *  to surface the FALLBACK branch (its defining behavior) in the gallery card
   *  without needing user interaction. The boundary catches the error and swaps
   *  in the fallback content. */
  error?: { message?: string; type?: string }
  /** Inject a behavioral trigger button next to the mounted specimen. Used for
   *  per-type demos (e.g. error-boundary needs a "Throw" button) where the
   *  closed-at-rest state hides the component's defining behavior. The control
   *  lives next to the stage, not inside it (so the demo trigger isn't clipped
   *  by overflow:hidden on the stage). Removed on unmount. */
  triggerButton?: { label: string; action: 'throw' | 'notify' }
}

export interface PreviewSeed {
  props?: Record<string, string>
  innerHTML?: string
  /** Detail/viewer surface: how to open/expand the REAL component so its
   *  interesting state is visible. The viewer mounts real and applies this —
   *  never a fake. */
  drive?: DriveDirective
  /** Gallery-card POSTER ONLY. A frozen thumbnail (native HTML + token vars)
   *  for true modal overlays that physically cannot render inside a 200px
   *  card even when contained (dialog / alert-dialog / cmdk / drawer / sheet).
   *  The gallery grid renders this as a thumbnail; the DETAIL page ignores it
   *  entirely and mounts the real, driven component. */
  staticHTML?: string
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

// sui-board wants the full persisted node shape: rect {x,y,w,h} + ports +
// kind (NOT flat {x,y}). The engine does not normalize, so a flat seed crashes
// on `n.rect.x`. Three nodes, two bezier edges — the compile pipeline as a graph.
const BOARD_DATA = JSON.stringify({
  nodes: [
    { id: 'a', rect: { x: 40, y: 80, w: 132, h: 52 }, ports: [{ id: 'out', side: 'right', offset: 0.5 }], kind: 'default', data: { label: 'Author .sui' } },
    { id: 'b', rect: { x: 280, y: 30, w: 132, h: 52 }, ports: [{ id: 'in', side: 'left', offset: 0.5 }], kind: 'default', data: { label: 'Compile' } },
    { id: 'c', rect: { x: 280, y: 140, w: 132, h: 52 }, ports: [{ id: 'in', side: 'left', offset: 0.5 }], kind: 'default', data: { label: 'Web component' } },
  ],
  edges: [
    { id: 'e1', source: { node: 'a', port: 'out' }, target: { node: 'b', port: 'in' }, kind: 'bezier' },
    { id: 'e2', source: { node: 'a', port: 'out' }, target: { node: 'c', port: 'in' }, kind: 'bezier' },
  ],
})

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
  'sui-icon':            { props: { name: 'sparkles', variant: 'tile', tone: 'primary', size: 'lg' } },
  // catalog (brand chrome, not a public component) and is composed
  'sui-chip':            { props: { removable: 'true', tone: 'primary' }, innerHTML: 'TypeScript' },
  // sui-error-boundary: the gallery card shows a staticHTML poster of the
  // FALLBACK branch — the boundary's defining behavior. The live
  // `<sui-error-boundary>` host renders an empty default slot when healthy
  // (its fallback is only swapped in after `runProtected` catches a real
  // throw from a child). A poster of the fallback shows what the boundary
  // produces in its interesting state without needing to fabricate a child
  // throw at mount. The detail page uses a live throw trigger (same
  // pattern as cmdk/dialog).
  'sui-error-boundary':  {
    staticHTML: '<div style="display:flex;flex-direction:column;gap:var(--space-2);padding:var(--space-3);border:1px solid var(--destructive);background:color-mix(in oklch, var(--destructive) 8%, var(--surface-1));border-radius:var(--radius-md);color:var(--destructive);font:inherit;font-size:var(--text-sm);max-width:18rem"><strong style="font-size:var(--text-md)">Fallback caught the error</strong><span style="color:var(--muted-foreground)">TypeError: child.render is not a function</span><button variant="ghost" size="sm" style="align-self:flex-start;color:var(--destructive)">Retry</button></div>',
  },
  'sui-context-menu':    { innerHTML: '<button slot="trigger" variant="outline">Right-click me</button><button>Cut</button><button>Copy</button><button>Paste</button>', drive: { attrs: { open: 'true' } } },
  // drive.click) opens the menu so the gallery card demos the menu surface
  // rather than just the closed trigger.
  'sui-context-menu':    { innerHTML: '<button slot="trigger" variant="outline">Right-click me</button><button>Cut</button><button>Copy</button><button>Paste</button>', drive: { click: '[slot="trigger"]' } },
  'sui-toast-host':      { staticHTML: '<div style="position:relative;display:flex;align-items:flex-end;justify-content:flex-end;width:100%;height:100%;min-height:7rem"><div style="background:var(--background);border:1px solid var(--border);border-left:3px solid var(--success);border-radius:var(--radius-md);box-shadow:var(--shadow-lg);padding:var(--space-3) var(--space-4);font:inherit;font-size:var(--text-sm);max-width:18rem"><strong style="display:block;margin-bottom:var(--space-1)">Deployed</strong><span style="color:var(--muted-foreground)">job/site-demos → production</span></div></div>' },

  // ── Modal overlays ──────────────────────────────────────────────────
  // DETAIL: real component, props open the panel, drive as backup.
  // GALLERY: staticHTML poster (a 200px card can't hold a real modal).
  'sui-dialog':          {
    props: { open: 'true', size: 'sm' },
    innerHTML: '<span slot="header">Confirm deployment</span><p>Deploy <code>job/site-demos</code> to production?</p><button slot="footer">Cancel</button><button slot="footer" tone="primary">Deploy</button>',
    drive: { attrs: { open: 'true' } },
    staticHTML: '<div style="width:100%;max-width:21rem;background:var(--background);border:1px solid var(--border);border-radius:var(--radius-lg);box-shadow:var(--shadow-lg);padding:1.25rem;display:flex;flex-direction:column;gap:0.6rem;font-size:var(--text-sm)"><strong style="font-size:var(--text-md)">Confirm deployment</strong><p style="margin:0;color:var(--muted-foreground)">Deploy <code>job/site-demos</code> to production?</p><div style="display:flex;gap:0.5rem;justify-content:flex-end;margin-top:0.4rem"><button>Cancel</button><button style="background:var(--primary);color:var(--primary-foreground);border-color:var(--primary)">Deploy</button></div></div>' },
  'sui-alert-dialog':    {
    props: { open: 'true', tone: 'destructive', 'confirm-label': 'Delete', 'cancel-label': 'Keep' },
    innerHTML: '<span slot="title">Delete project?</span>This permanently removes the project and every deployment under it.',
    drive: { attrs: { open: 'true' } },
    staticHTML: '<div style="width:100%;max-width:21rem;background:var(--background);border:1px solid var(--border);border-radius:var(--radius-lg);box-shadow:var(--shadow-lg);padding:1.25rem;display:flex;flex-direction:column;gap:0.6rem;font-size:var(--text-sm)"><strong style="font-size:var(--text-md);color:var(--destructive)">Delete project?</strong><p style="margin:0;color:var(--muted-foreground)">This permanently removes the project and every deployment under it.</p><div style="display:flex;gap:0.5rem;justify-content:flex-end;margin-top:0.4rem"><button>Keep</button><button style="background:var(--destructive);color:var(--destructive-foreground);border-color:var(--destructive)">Delete</button></div></div>' },
  // open lives in `drive` only (NOT props): a drawer portals to <body>, so an
  // open prop would take over the GALLERY grid, not just the detail stage. The
  // gallery renders the poster; the detail page drives the real drawer open.
  'sui-drawer':          {
    props: { side: 'right' },
    innerHTML: '<span slot="title">Filters</span><span slot="description">Refine the component list</span><p>Drawer body content.</p>',
    drive: { attrs: { open: 'true' } },
    staticHTML: '<div style="width:100%;max-width:16rem;margin-left:auto;height:100%;min-height:13rem;background:var(--background);border-left:1px solid var(--border);box-shadow:var(--shadow-lg);padding:1.25rem;display:flex;flex-direction:column;gap:0.5rem;font-size:var(--text-sm)"><strong style="font-size:var(--text-md)">Filters</strong><span style="color:var(--muted-foreground)">Refine the component list</span><p style="margin:0.5rem 0 0">Drawer body content.</p></div>' },
  'sui-cmdk':            {
    props: { open: 'true' },
    drive: { attrs: { open: 'true' } },
    staticHTML: '<div style="width:100%;max-width:22rem;background:var(--background);border:1px solid var(--border);border-radius:var(--radius-lg);box-shadow:var(--shadow-lg);overflow:hidden;font-size:var(--text-sm)"><div style="padding:0.7rem 1rem;border-bottom:1px solid var(--border);color:var(--muted-foreground)">⌕&nbsp; Type a command…</div><div style="padding:0.4rem"><div style="padding:0.5rem 0.75rem;border-radius:var(--radius-md);background:var(--surface-2)">New deployment</div><div style="padding:0.5rem 0.75rem;color:var(--muted-foreground)">Open docs</div></div></div>' },
  // sui-portal: teleports children; an empty stage at the host looks empty.
  // The cell shows a staticHTML poster of the portal target (a small "teleported
  // content" badge) so the cell reads as a meaningful primitive, not a hole.
  'sui-portal':          {
    staticHTML: '<div style="display:flex;align-items:center;gap:var(--space-2);padding:var(--space-3) var(--space-4);background:var(--surface-1);border:1px dashed var(--border);border-radius:var(--radius-md);color:var(--muted-foreground);font:inherit;font-size:var(--text-sm);max-width:18rem"><strong style="color:var(--foreground)">↗ teleported</strong><span>Children render in the body, not in place.</span></div>',
  },

  // ── Forms ───────────────────────────────────────────────────────────
  'sui-input':           { props: { value: 'Ada Lovelace', label: 'Full name', placeholder: 'Enter your name' } },
  'sui-slider':          {
    props: { value: '64', min: '0', max: '100', label: 'Volume' },
    // The live slider is height:6px which gets lost in the gallery card. A
    // hand-authored poster makes the slider readable in a thumbnail.
    staticHTML: '<div style="display:flex;flex-direction:column;gap:var(--space-2);width:100%;max-width:18rem;font:inherit;font-size:var(--text-sm)"><div style="display:flex;justify-content:space-between;color:var(--muted-foreground);font-family:var(--font-mono);font-size:var(--text-xs)"><span>0</span><span>Volume · 64</span><span>100</span></div><div style="position:relative;height:8px;background:var(--muted);border-radius:var(--radius-full);box-shadow:inset 0 0 0 1px var(--border)"><div style="position:absolute;top:0;left:0;width:64%;height:100%;background:var(--primary);border-radius:inherit"></div><div style="position:absolute;top:50%;left:64%;width:18px;height:18px;background:var(--surface-1);border:2px solid var(--primary);border-radius:50%;transform:translate(-50%,-50%);box-shadow:var(--shadow-sm)"></div></div></div>',
  },
  'sui-radio-group':     { props: { value: 'card', label: 'Payment method' }, innerHTML: '<sui-radio value="card">Credit card</sui-radio><sui-radio value="paypal">PayPal</sui-radio><sui-radio value="wire">Wire transfer</sui-radio>' },
  'sui-toggle-group':    { props: { value: 'grid' }, innerHTML: '<sui-toggle value="list">List</sui-toggle><sui-toggle value="grid">Grid</sui-toggle><sui-toggle value="board">Board</sui-toggle>' },
  'sui-select':          { props: { value: 'prod', label: 'Environment' }, innerHTML: '<option value="dev">Development</option><option value="staging">Staging</option><option value="prod">Production</option>' },
  'sui-combobox':        { props: { placeholder: 'Choose a framework', options: '[{"value":"react","label":"React"},{"value":"vue","label":"Vue"},{"value":"svelte","label":"Svelte"},{"value":"solid","label":"SolidJS"},{"value":"qwik","label":"Qwik"}]' } },
  'sui-autocomplete':    { props: { placeholder: 'Search commands…', suggestions: '["deploy","deploy --prod","deploy --preview","rollback","rollback --to"]' } },
  'sui-field':           { innerHTML: '<label slot="label">Email address</label><sui-input type="email" value="ada@ship.dev"></sui-input><span slot="hint">We\'ll never share your email.</span>' },
  'sui-date-picker':     { props: { value: '2026-07-15', label: 'Ship date' } },
  'sui-color-picker':    { props: { value: '#4f46e5' }, drive: { click: '[part="input"]' } },
  'sui-file-upload':     { props: { accept: '.png,.jpg,.pdf', multiple: 'true' }, innerHTML: 'Drag files here or click to browse' },

  // ── Structure & layout ──────────────────────────────────────────────
  'sui-tabs':            { props: { active: 'overview' }, innerHTML: '<sui-tab name="overview">Overview</sui-tab><sui-tab name="activity" count="3">Activity</sui-tab><sui-tab name="settings">Settings</sui-tab><sui-tab-panel name="overview" active>A live overview of the deployment.</sui-tab-panel><sui-tab-panel name="activity">3 new events since your last visit.</sui-tab-panel><sui-tab-panel name="settings">Configure build &amp; deploy options.</sui-tab-panel>' },
  'sui-timeline':        { innerHTML: '<sui-timeline-item at="09:30" tone="success"><span slot="title">Build passed</span>24 checks green in 42s.</sui-timeline-item><sui-timeline-item at="09:45" tone="info"><span slot="title">Deployed to staging</span>job/site-demos → staging.</sui-timeline-item><sui-timeline-item at="10:02"><span slot="title">Promoted to production</span>Live for all users.</sui-timeline-item>' },
  'sui-scroll-area':     { props: { size: 'sm' }, innerHTML: '<div>sui-button</div><div>sui-card</div><div>sui-input</div><div>sui-select</div><div>sui-table</div><div>sui-tree</div><div>sui-calendar</div><div>sui-dialog</div><div>sui-tabs</div><div>sui-timeline</div><div>sui-tooltip</div>' },

  // ── Navigation ──────────────────────────────────────────────────────
  'sui-nav-top':         { innerHTML: '<span slot="brand"><strong>SHIP</strong></span><a href="#">Docs</a><a href="#">Components</a><a href="#">Guides</a><button slot="actions" tone="primary">Sign in</button>' },
  'sui-navigation-menu': { props: { value: 'components' }, innerHTML: '<sui-navigation-item value="docs">Docs</sui-navigation-item><sui-navigation-item value="components">Components</sui-navigation-item><sui-navigation-item value="guides">Guides</sui-navigation-item>' },
  'sui-pagination':      { props: { page: '3', total: '248', 'page-size': '20' } },

  // ── Anchored overlays — closed at rest, a poster shows what they ARE.
  //     Each is a triggered overlay (popover/menu/tooltip/hover-card). A
  //     bare mount is closed, so the cell would be empty. The staticHTML
  //     poster renders a representative OPEN state at thumbnail scale. ──
  'sui-popover':          {
    staticHTML: '<div style="position:relative;display:flex;align-items:center;justify-content:center;min-height:7rem;width:100%"><button style="background:var(--surface-1);border:1px solid var(--border);border-radius:var(--radius-md);padding:var(--space-2) var(--space-3);font:inherit;font-size:var(--text-sm);color:var(--foreground)">Open</button><div style="position:absolute;top:calc(100% - 0.4rem);left:50%;transform:translateX(-50%);background:var(--background);border:1px solid var(--border);border-radius:var(--radius-md);box-shadow:var(--shadow-lg);padding:var(--space-3);font:inherit;font-size:var(--text-sm);min-width:14rem"><strong style="display:block;margin-bottom:var(--space-1)">Popover content</strong><span style="color:var(--muted-foreground)">Anchored to its trigger; closes on outside click.</span></div></div>',
  },
  'sui-dropdown-menu':    {
    staticHTML: '<div style="position:relative;display:flex;align-items:center;justify-content:center;min-height:7rem;width:100%"><button style="background:var(--surface-1);border:1px solid var(--border);border-radius:var(--radius-md);padding:var(--space-2) var(--space-3);font:inherit;font-size:var(--text-sm);color:var(--foreground)">Actions</button><div style="position:absolute;top:calc(100% - 0.4rem);left:50%;transform:translateX(-50%);background:var(--background);border:1px solid var(--border);border-radius:var(--radius-md);box-shadow:var(--shadow-lg);padding:var(--space-1);font:inherit;font-size:var(--text-sm);min-width:12rem;display:flex;flex-direction:column"><div style="padding:var(--space-2) var(--space-3);border-radius:var(--radius-sm);cursor:pointer">Cut</div><div style="padding:var(--space-2) var(--space-3);border-radius:var(--radius-sm);cursor:pointer">Copy</div><div style="padding:var(--space-2) var(--space-3);border-radius:var(--radius-sm);cursor:pointer">Paste</div></div></div>',
  },
  'sui-tooltip':          {
    staticHTML: '<div style="position:relative;display:flex;align-items:center;justify-content:center;min-height:5rem;width:100%"><button style="background:var(--surface-1);border:1px solid var(--border);border-radius:var(--radius-md);padding:var(--space-2) var(--space-3);font:inherit;font-size:var(--text-sm);color:var(--foreground)">Hover me</button><div style="position:absolute;bottom:calc(100% + 0.4rem);left:50%;transform:translateX(-50%);background:var(--foreground);color:var(--background);border-radius:var(--radius-sm);padding:var(--space-1) var(--space-2);font:inherit;font-size:var(--text-xs);white-space:nowrap">Helpful hint</div></div>',
  },
  'sui-hover-card':       {
    staticHTML: '<div style="position:relative;display:flex;align-items:center;justify-content:center;min-height:8rem;width:100%"><div style="padding:var(--space-2) var(--space-3);background:var(--surface-1);border:1px solid var(--border);border-radius:var(--radius-md);font:inherit;font-size:var(--text-sm);color:var(--foreground);text-decoration:underline;text-decoration-color:var(--primary)">@ada-lovelace</div><div style="position:absolute;top:calc(100% - 0.4rem);left:50%;transform:translateX(-50%);background:var(--background);border:1px solid var(--border);border-radius:var(--radius-md);box-shadow:var(--shadow-lg);padding:var(--space-3);font:inherit;font-size:var(--text-sm);min-width:18rem;display:flex;gap:var(--space-3);align-items:center"><div style="width:2.5rem;height:2.5rem;border-radius:50%;background:linear-gradient(135deg,var(--primary),var(--accent));flex-shrink:0"></div><div><strong>Ada Lovelace</strong><br><span style="color:var(--muted-foreground)">First programmer · London</span></div></div></div>',
  },

  // ── Data display (JSON-string scalar props — the SSR/declarative path)
  // Table: realistic project-status rows, not 1 column of empty cells.
  // native variant frames a real light-DOM <table> — renders reliably in the
  // gallery (the virtualized `data` variant needs a live JS rows reference the
  // attribute-only gallery can't supply, and windows to 0 rows without it).
  'sui-table':           { props: { variant: 'native', label: 'Components' }, innerHTML: '<table><thead><tr><th>Component</th><th>Pack</th><th>Status</th></tr></thead><tbody><tr><td>sui-button</td><td>primitives</td><td>stable</td></tr><tr><td>sui-card</td><td>primitives</td><td>stable</td></tr><tr><td>sui-calendar</td><td>primitives</td><td>beta</td></tr><tr><td>sui-render</td><td>viewer</td><td>beta</td></tr></tbody></table>' },
  'sui-tree':            { props: { label: 'Files', nodes: SMALL_TREE_NODES } },
  'sui-json-view':       { props: { data: '{"name":"@ship/sui","version":"0.3.0","stable":true,"packs":["primitives","charts","canvas"],"engine":{"compiler":"rust","runtime":"signals"}}', expanded: 'true' } },
  'sui-tree-grid':       { props: { label: 'Files', columns: '[{"field":"label","label":"Name"},{"field":"kind","label":"Kind"},{"field":"size","label":"Size"}]', nodes: SMALL_TREE_NODES } },
  'sui-tree-view':       { innerHTML: '<sui-tree-item label="src" expanded><sui-tree-item label="components" expanded><sui-tree-item label="sui-button.sui" leaf></sui-tree-item><sui-tree-item label="sui-card.sui" leaf></sui-tree-item></sui-tree-item><sui-tree-item label="index.ts" leaf></sui-tree-item></sui-tree-item>' },
  'sui-sheet':           { props: { label: 'Q3 Budget', workbook: '{"sheets":[{"id":"q3","name":"Q3 Budget","rowCount":5,"colCount":4,"cells":{"A1":{"value":"Item"},"B1":{"value":"Q1"},"C1":{"value":"Q2"},"D1":{"value":"Q3"},"A2":{"value":"Revenue"},"B2":{"value":120},"C2":{"value":145},"D2":{"value":168},"A3":{"value":"Costs"},"B3":{"value":80},"C3":{"value":92},"D3":{"value":101},"A4":{"value":"Profit"},"B4":{"value":40,"formula":"=B2-B3"},"C4":{"value":53,"formula":"=C2-C3"},"D4":{"value":67,"formula":"=D2-D3"}}}]}' } },

  // ── Code & content ──────────────────────────────────────────────────
  'sui-code-editor':     { props: { language: 'html', value: '<prop name="tone" values="default|primary" default="default" />\n\n<button class="btn btn-{tone()}">\n  <slot></slot>\n</button>' } },
  'sui-markdown':        { props: { value: '# Ship SUI\n\n**Batteries-included** components for the web.\n\n- Compiles to standard web components\n- Themed with design tokens\n- `sui-compiler build` in the loop' } },
  'sui-diff':            { props: { from: "const tone = 'primary'\nconst size = 'md'", to: "const tone = 'success'\nconst size = 'lg'\nconst variant = 'solid'" } },
  'sui-mermaid':         { props: { value: 'graph LR\n  A[Author .sui] --> B[Compile]\n  B --> C[Web Component]\n  C --> D[Themed by tokens]' } },

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
  'sui-board':           { staticHTML: '<svg viewBox="0 0 200 80" preserveAspectRatio="xMidYMid meet" style="width:100%;height:100%;display:block" aria-hidden="true"><line x1="50" y1="40" x2="150" y2="40" stroke="var(--muted-foreground)" stroke-width="1.5"/><circle cx="50" cy="40" r="22" fill="var(--background)" stroke="var(--primary)" stroke-width="2"/><text x="50" y="44" text-anchor="middle" font-family="ui-monospace,monospace" font-size="9" fill="var(--foreground)">A</text><circle cx="150" cy="40" r="22" fill="var(--background)" stroke="var(--accent)" stroke-width="2"/><text x="150" y="44" text-anchor="middle" font-family="ui-monospace,monospace" font-size="9" fill="var(--foreground)">B</text></svg>' },
  // Canvas family — the live primitive needs JS-prop seams (canvas/board
 // /graph-canvas/draw) to populate content; the attribute-only gallery
 // mount can only show the base sheet. Use a staticHTML poster that shows
  // the substrate + a representative "sketch" so the cell reads as a
  // canvas primitive, not a hole.
  'sui-canvas':          { staticHTML: '<svg viewBox="0 0 200 80" preserveAspectRatio="xMidYMid meet" style="width:100%;height:100%;display:block" aria-hidden="true"><circle cx="50" cy="40" r="22" fill="none" stroke="var(--primary)" stroke-width="2"/><rect x="128" y="20" width="44" height="40" fill="none" stroke="var(--accent)" stroke-width="2" rx="4"/><line x1="72" y1="40" x2="128" y2="40" stroke="var(--muted-foreground)" stroke-width="1.5" stroke-dasharray="3 3"/></svg>' },
  'sui-draw':            { staticHTML: '<svg viewBox="0 0 200 60" preserveAspectRatio="xMidYMid meet" style="width:100%;height:100%;display:block" aria-hidden="true"><path d="M 20 40 Q 50 10, 80 35 T 140 25 T 180 20" fill="none" stroke="var(--primary)" stroke-width="2" stroke-linecap="round"/><circle cx="180" cy="20" r="3" fill="var(--primary)"/></svg>' },
  'sui-graph-canvas':    { props: { width: '240', height: '160', layout: 'positioned', nodes: '[{"id":"a","title":"Author","x":40,"y":40,"kind":"skill"},{"id":"b","title":"Compile","x":120,"y":40,"kind":"skill"},{"id":"c","title":"Ship","x":75,"y":110,"kind":"skill"}]', edges: '[{"from":"a","to":"b"},{"from":"a","to":"c"}]' } },
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
  // sui-message-list: needs `prop:items` (JS-prop seam) to render a list.
  // A staticHTML poster shows what the conversation looks like; the live
  // cell without the JS prop is content-light.
  'sui-message-list':    { staticHTML: '<div style="display:flex;flex-direction:column;gap:var(--space-2);width:100%;max-width:18rem;font:inherit;font-size:var(--text-sm)"><div style="display:flex;gap:var(--space-2);align-items:flex-end"><div style="width:1.5rem;height:1.5rem;border-radius:50%;background:linear-gradient(135deg,var(--primary),var(--accent));flex-shrink:0"></div><div style="background:var(--surface-1);border:1px solid var(--border);border-radius:var(--radius-md);padding:var(--space-2) var(--space-3);max-width:80%">Review the production build?</div></div><div style="display:flex;gap:var(--space-2);align-items:flex-end;flex-direction:row-reverse"><div style="background:var(--primary);color:var(--primary-foreground);border-radius:var(--radius-md);padding:var(--space-2) var(--space-3);max-width:80%">Reviewed &mdash; green across the board.</div></div></div>' },
  'sui-thinking-dots':   { props: { label: 'Reviewing the build…', tone: 'primary' } },
  'sui-tool-call':       { props: { name: 'createDeployment', status: 'ok' }, innerHTML: '<code>site-demos</code> · <span>commit a7f31c · main · 42s</span>' },

  // ── Media ───────────────────────────────────────────────────────────
  // sui-media-av and sui-media-image both need `prop:src` (a JS-prop
  // seam the canvas-mount path can't supply) to render any frames. The
  // cells are empty by design; the card description explains the contract.
  // sui-media-av and sui-media-image both need `prop:src` (a JS-prop
  'sui-media-av':        { staticHTML: '<svg viewBox="0 0 80 50" preserveAspectRatio="xMidYMid meet" style="width:100%;height:100%;max-height:5rem;display:block" aria-hidden="true"><rect x="2" y="2" width="76" height="46" rx="4" fill="var(--background)" stroke="var(--border)" stroke-width="1.5"/><polygon points="32,16 32,34 50,25" fill="var(--muted-foreground)"/></svg>' },
  'sui-media-csv':       { props: { header: 'true', text: 'name,pack,status\nsui-button,primitives,stable\nsui-card,primitives,stable\nsui-render,viewer,beta' } },
  'sui-media-image':     { staticHTML: '<svg viewBox="0 0 80 50" preserveAspectRatio="xMidYMid meet" style="width:100%;height:100%;max-height:5rem;display:block" aria-hidden="true"><defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="1"><stop offset="0%" stop-color="var(--primary)"/><stop offset="100%" stop-color="var(--accent)"/></linearGradient></defs><rect x="2" y="2" width="76" height="46" rx="4" fill="url(#g)"/><circle cx="25" cy="20" r="4" fill="rgba(255,255,255,0.4)"/><path d="M 12 44 L 32 28 L 50 36 L 70 20" fill="none" stroke="rgba(255,255,255,0.5)" stroke-width="1.5"/></svg>' },
}
