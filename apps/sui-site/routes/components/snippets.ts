// Usage snippets shown beside each live component. Kept out of the controller
// so the scope scan isn't fooled by patterns inside the strings. Verified
// against each primitive's real <prop> declarations + child structure.

// JSON for the <sui-json-view> demo — passed via a BINDING (data={jsonData}),
// not an inline `data='{…}'` attribute: a `{` in an attribute value is parsed
// as a binding delimiter by the compiler, which mangles inline JSON.
export const jsonData = '{"agent":"gatekeeper","files":14,"passing":true,"findings":null}'

export const code: Record<string, string> = {
  // ── Actions ──
  button: `<sui-button tone="primary" variant="solid">Save changes</sui-button>
<sui-button variant="outline">Cancel</sui-button>
<sui-button variant="ghost">Ghost</sui-button>
<sui-button tone="destructive" variant="soft" size="sm">Delete</sui-button>`,

  // ── Forms ──
  input: `<sui-input type="email" placeholder="you@example.com"></sui-input>`,
  field: `<sui-field>
  <sui-label slot="label" required>Email</sui-label>
  <sui-input type="email" placeholder="you@example.com"></sui-input>
  <span slot="hint">We'll never share it.</span>
</sui-field>`,
  label: `<sui-label required>Email address</sui-label>`,
  checkbox: `<sui-checkbox checked tone="primary">Subscribe to updates</sui-checkbox>`,
  switch: `<sui-switch checked label="Notifications"></sui-switch>
<sui-switch tone="success" checked label="Auto-deploy"></sui-switch>
<sui-switch disabled label="Locked"></sui-switch>`,
  radioGroup: `<sui-radio-group value="comfortable" label="Density" tone="primary">
  <sui-radio value="compact">Compact</sui-radio>
  <sui-radio value="comfortable">Comfortable</sui-radio>
  <sui-radio value="spacious">Spacious</sui-radio>
</sui-radio-group>`,
  toggleGroup: `<sui-toggle-group type="single" value="b" variant="outline" name="align">
  <sui-toggle value="a">Left</sui-toggle>
  <sui-toggle value="b">Center</sui-toggle>
  <sui-toggle value="c">Right</sui-toggle>
</sui-toggle-group>`,
  select: `<sui-select value="opus" label="Model">
  <option value="opus">Claude Opus</option>
  <option value="sonnet">Claude Sonnet</option>
  <option value="haiku">Claude Haiku</option>
</sui-select>`,
  combobox: `<sui-combobox placeholder="Pick a framework">
  <option value="sui">SUI</option>
  <option value="lit">Lit</option>
  <option value="svelte">Svelte</option>
</sui-combobox>`,
  autocomplete: `<sui-autocomplete placeholder="Search fruit">
  <option value="apple">Apple</option>
  <option value="apricot">Apricot</option>
  <option value="banana">Banana</option>
</sui-autocomplete>`,
  textarea: `<sui-textarea placeholder="Add a description" rows="3"></sui-textarea>`,
  slider: `<sui-slider value="40" label="Volume" tone="primary"></sui-slider>`,
  colorPicker: `<sui-color-picker value="#6d28d9"></sui-color-picker>`,
  fileUpload: `<sui-file-upload accept="image/*" multiple></sui-file-upload>`,

  // ── Feedback ──
  alert: `<sui-alert tone="success" dismissible>
  <span slot="title">Saved</span>
  Your changes were saved.
</sui-alert>`,
  spinner: `<sui-spinner></sui-spinner>
<sui-spinner tone="primary" size="lg"></sui-spinner>`,
  skeleton: `<sui-skeleton shape="circle"></sui-skeleton>
<sui-skeleton shape="text"></sui-skeleton>
<sui-skeleton shape="block"></sui-skeleton>`,
  emptyState: `<sui-empty-state>
  <sui-icon slot="icon" name="inbox"></sui-icon>
  <span slot="title">No messages</span>
  Your inbox is all caught up.
  <sui-button slot="action" size="sm" variant="outline">Refresh</sui-button>
</sui-empty-state>`,
  errorState: `<sui-error-state title="Failed to load" body="The agent stream disconnected." code="ERR_STREAM"></sui-error-state>`,
  statusDot: `<sui-status-dot tone="success" label="Online"></sui-status-dot>
<sui-status-dot tone="warning" pulse label="Pending"></sui-status-dot>
<sui-status-dot tone="destructive" label="Error"></sui-status-dot>`,

  // ── Display ──
  avatar: `<sui-avatar initials="MC"></sui-avatar>
<sui-avatar name="Ada Lovelace" tone="primary"></sui-avatar>
<sui-avatar name="Grace Hopper" shape="square" size="lg"></sui-avatar>`,
  badge: `<sui-badge>Default</sui-badge>
<sui-badge tone="success">Live</sui-badge>
<sui-badge tone="primary" variant="solid">New</sui-badge>
<sui-badge tone="destructive" variant="outline">Error</sui-badge>`,
  chip: `<sui-chip>Design</sui-chip>
<sui-chip tone="primary" selected>Selected</sui-chip>
<sui-chip removable>Removable</sui-chip>`,
  card: `<sui-card>
  <span slot="header">ship-it</span>
  A card groups related content on a raised surface.
  <div slot="footer"><sui-button tone="primary" size="sm">Dispatch</sui-button></div>
</sui-card>`,
  icon: `<sui-icon name="rocket" size="lg"></sui-icon>
<sui-icon name="sparkles" size="lg" tone="primary"></sui-icon>
<sui-icon name="git-branch" size="lg" tone="success"></sui-icon>`,
  brandMark: `<sui-brand-mark size="sm"></sui-brand-mark>
<sui-brand-mark size="lg" label="Ship"></sui-brand-mark>`,
  code: `<sui-code>const sum = (a, b) => a + b</sui-code>`,
  markdown: `<sui-markdown value="Renders **bold**, _italic_, and [links](#)."></sui-markdown>`,
  jsonView: `<sui-json-view data='{"agent":"gatekeeper","files":14,"passing":true}'></sui-json-view>`,
  diff: `<sui-diff from="role: viewer" to="role: admin"></sui-diff>`,
  calendar: `<sui-calendar view="month"></sui-calendar>`,

  // ── Layout ──
  cluster: `<sui-cluster gap="2">
  <sui-badge>One</sui-badge>
  <sui-badge tone="primary">Two</sui-badge>
  <sui-badge tone="success">Three</sui-badge>
</sui-cluster>`,
  stack: `<sui-stack gap="2">
  <sui-badge tone="info">Item 1</sui-badge>
  <sui-badge tone="warning">Item 2</sui-badge>
  <sui-badge tone="success">Item 3</sui-badge>
</sui-stack>`,
  separator: `<sui-separator></sui-separator>
<sui-separator variant="dashed" tone="primary"></sui-separator>`,
  scrollArea: `<sui-scroll-area orientation="vertical" style="height:120px;width:300px">
  <div>...overflowing content...</div>
</sui-scroll-area>`,

  // ── Navigation ──
  link: `<sui-link href="#">Documentation</sui-link>
<sui-link href="#" tone="primary" variant="underline">Get started</sui-link>
<sui-link href="#" external>External site</sui-link>`,
  pagination: `<sui-pagination page="2" total="97" page-size="10"></sui-pagination>`,
  navMenu: `<sui-navigation-menu value="home">
  <sui-navigation-item value="home" active>Home</sui-navigation-item>
  <sui-navigation-item value="docs">Docs</sui-navigation-item>
  <sui-navigation-item value="api">API</sui-navigation-item>
</sui-navigation-menu>`,
  tabs: `<sui-tabs active="tab1" variant="underline">
  <sui-tab name="tab1" label="Overview"></sui-tab>
  <sui-tab name="tab2" label="Settings"></sui-tab>
  <sui-tab name="tab3" label="Logs" count="12"></sui-tab>
</sui-tabs>`,
  tabPanel: `<sui-tabs active="tp1" variant="underline">
  <sui-tab name="tp1" label="Overview"></sui-tab>
  <sui-tab name="tp2" label="Details"></sui-tab>
  <sui-tab-panel name="tp1">Overview panel content.</sui-tab-panel>
  <sui-tab-panel name="tp2">Details panel content.</sui-tab-panel>
</sui-tabs>`,
  treeView: `<sui-tree-view>
  <sui-tree-item label="src" expanded>
    <sui-tree-item label="components" leaf></sui-tree-item>
    <sui-tree-item label="views" expanded>
      <sui-tree-item label="page.sui" leaf></sui-tree-item>
    </sui-tree-item>
  </sui-tree-item>
</sui-tree-view>`,

  // ── Disclosure ──
  accordion: `<sui-accordion value="compile">
  <sui-accordion-item value="what">
    <span slot="trigger">What is SUI?</span>
    <p>A web-component UI kit compiled from .sui source.</p>
  </sui-accordion-item>
  <sui-accordion-item value="compile">
    <span slot="trigger">How does it compile?</span>
    <p>Each .sui file becomes an ESM custom element.</p>
  </sui-accordion-item>
</sui-accordion>`,
  collapsible: `<sui-collapsible open>
  <sui-button slot="trigger" variant="outline" size="sm">Details</sui-button>
  <p>Content revealed when the trigger is toggled.</p>
</sui-collapsible>`,

  // ── Data ──
  table: `<sui-table variant="slot" label="Team" hoverable>
  <tr slot="head"><th>Name</th><th>Role</th><th>Status</th></tr>
  <tr><td>Alice</td><td>Engineer</td><td><sui-badge tone="success" size="sm">Active</sui-badge></td></tr>
  <tr><td>Bob</td><td>Designer</td><td><sui-badge tone="warning" size="sm">Away</sui-badge></td></tr>
</sui-table>`,
  timeline: `<sui-timeline>
  <sui-timeline-item tone="success" at="2026-06-01">
    <span slot="title">Deployed</span>
    Release shipped to production.
  </sui-timeline-item>
  <sui-timeline-item tone="info" at="2026-05-30">
    <span slot="title">Review passed</span>
    All checks green.
  </sui-timeline-item>
</sui-timeline>`,
}
