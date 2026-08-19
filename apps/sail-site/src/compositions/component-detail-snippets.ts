// /components/[tag] — the USAGE snippet each detail page shows, one per entry
// on the full L1 surface. Every one is a complete
// single-file program (controller + markup where state is read) and compiles
// as written — checked by crates/sail/compiler/tests/site_snippets_compile.rs.
import { signal, derived } from '@ship/sail-runtime'
export const usage: Record<string, string> = {
  button: `<controller>
export default controller(() => {
  const saved = signal(false)
  return { saved, save: () => saved.set(true) }
})
</controller>

<button on:click={save}>
  {saved() ? 'Saved ✓' : 'Save changes'}
</button>`,

  a: `<p>
  Read the <a href="/docs">docs</a>, or open the
  <a href="/components" variant="solid" tone="primary">component catalog</a>.
</p>
<!-- bare <a> = text link · variant set = button-style link -->`,

  // Example programs for the detail walkthrough (not the primary usage block).
  'button.with-icon': `<button variant="solid" tone="primary">
  <icon name="check" size="sm"></icon>
  Save
</button>

<button variant="outline" tone="primary">
  Export
  <icon name="download" size="sm"></icon>
</button>`,

  'button.icon-only': `<button variant="solid" tone="primary" size="icon" label="Settings">
  <icon name="settings"></icon>
</button>
<button variant="ghost" size="iconSm" label="Search">
  <icon name="search" size="sm"></icon>
</button>`,

  'button.sizes': `<button variant="solid" tone="primary" size="xs">xs</button>
<button variant="solid" tone="primary" size="sm">sm</button>
<button variant="solid" tone="primary" size="md">md</button>
<button variant="solid" tone="primary" size="lg">lg</button>`,

  input: `<controller>
export default controller(() => ({
  email: signal(''),
  agreed: signal(false),
}))
</controller>

<input type="email" placeholder="you@ship.dev" bind:value={email} />
<input type="email" value="not-an-email" invalid />
<label>
  <checkbox bind:checked={agreed} />
  I agree
</label>`,

  textarea: `<controller>
export default controller(() => ({ bio: signal('') }))
</controller>

<textarea
  placeholder="Tell us about yourself"
  size="md"
  rows="3"
  bind:value={bio}
/>
<p>{bio().length} characters</p>`,

  select: `<controller>
export default controller(() => ({ team: signal('Design') }))
</controller>

<select bind:value={team} size="md">
  <option>Engineering</option>
  <option>Design</option>
  <option>Research</option>
</select>
<p>team: {team()}</p>
<!-- closed control only; styled filter listbox is <combobox> -->`,

  combobox: `<controller>
export default controller(() => {
  const fruit = signal('')
  return {
    fruit,
    onSelect: (e) => fruit.set(e?.label ?? e?.value ?? ''),
  }
})
</controller>

<combobox placeholder="Pick a fruit" on-select={onSelect}>
  <option value="apple">Apple</option>
  <option value="banana">Banana</option>
  <option value="cherry">Cherry</option>
  <option value="grape">Grape</option>
</combobox>
<p>Picked: {fruit() || 'nothing yet'}</p>`,

  checkbox: `<controller>
export default controller(() => ({ agreed: signal(false) }))
</controller>

<label><checkbox bind:checked={agreed} size="md" /> I agree to the terms</label>
<p>{agreed() ? 'thank you' : 'please accept'}</p>`,

  date: `<date type="date" />
<date type="time" step="900" />
<date type="datetime-local" />`,

  file: `<file accept="image/*" multiple />`,

  color: `<color value="#c8f542" />`,

  radio: `<radio value="monthly">Monthly</radio>
<radio value="yearly" disabled>Yearly (locked)</radio>`,

  'radio-group': `<radio-group label="Billing period" name="plan" value="monthly">
  <radio value="monthly">Monthly</radio>
  <radio value="yearly">Yearly</radio>
</radio-group>`,

  switch: `<controller>
export default controller(() => ({ notify: signal(true) }))
</controller>

<row gap="3" align="center">
  <switch bind:checked={notify} size="md" tone="primary"></switch>
  <p>{notify() ? 'Notifications on' : 'Muted'}</p>
</row>`,

  slider: `<controller>
export default controller(() => ({ volume: signal(48) }))
</controller>

<column gap="2">
  <row justify="between"><span>Volume</span><span>{volume()}%</span></row>
  <slider bind:value={volume} min="0" max="100" size="md"></slider>
</column>`,

  'toggle-group': `<toggle-group variant="segmented" style({ animate: { enter: play(fade) } })>
  <toggle value="left">Left</toggle>
  <toggle value="center">Center</toggle>
  <toggle value="right">Right</toggle>
</toggle-group>`,

  field: `<field style({ animate: { enter: play(fade) } })>
  <label for="name">Name</label>
  <input id="name" placeholder="Ada Lovelace" />
</field>

<field invalid>
  <label for="email">Email</label>
  <input id="email" value="not-an-email" />
</field>`,

  label: `<column gap="1">
  <label for="email" required>Email</label>
  <input id="email" placeholder="ada@lovelace.dev" />
</column>`,

  fieldset: `<fieldset>
  <legend>Account details</legend>
  <column gap="2">
    <input placeholder="Name" />
    <input placeholder="Email" />
  </column>
</fieldset>`,

  legend: `<fieldset>
  <legend>Account details</legend>
  <column gap="2">
    <input placeholder="Name" />
    <input placeholder="Email" />
  </column>
</fieldset>`,

  'theme-picker': `<theme-picker></theme-picker>`,

  ticker: `<ticker>
  <p>tokens · signals · a11y · hot reload · SSR</p>
</ticker>`,

  alert: `<controller>
export default controller(() => {
  const visible = signal(true)
  return { visible, dismiss: () => visible.set(false) }
})
</controller>

<if {visible()}>
  <alert tone="warning" style({ animate: { exit: play(fade) } })>
    <strong>Careful.</strong> This cannot be undone.
  </alert>
</if>
<button on:click={dismiss} variant="ghost">Dismiss</button>`,

  progress: `<controller>
export default controller(() => ({ done: signal(64) }))
</controller>

<progress value={done()} max="100" size="md"></progress>
<p>{done()}% uploaded — native value/max; track and fill from the theme</p>
<!-- omit value for indeterminate shimmer -->`,

  meter: `<column gap="2">
  <p>Disk free — 72%</p>
  <meter value="72" min="0" max="100" low="33" high="66" optimum="100"></meter>
</column>`,

  spinner: `<controller>
export default controller(() => ({ loading: signal(true) }))
</controller>

<if {loading()}>
  <row gap="2" align="center">
    <spinner tone="primary"></spinner>
    <p>fetching…</p>
  </row>
</if>`,

  skeleton: `<row gap="3" align="center">
  <skeleton shape="circle" size="lg"></skeleton>
  <column gap="2" style({ grow(1) })>
    <skeleton shape="text" size="md"></skeleton>
    <skeleton shape="text" size="sm"></skeleton>
  </column>
</row>`,

  'empty-state': `<empty-state>
  <icon slot="icon" name="inbox"></icon>
  <strong slot="title">No results</strong>
  Try a different search, or clear a filter.
  <button slot="action" variant="outline" size="sm">Clear filters</button>
</empty-state>`,

  'error-state': `<error-state code="ERR_TIMEOUT">
  <strong slot="title">Couldn't load your data</strong>
  The request timed out. Retry, or check the connection.
  <button slot="action" type="button" variant="solid" tone="destructive" size="sm">Retry</button>
</error-state>`,

  'error-boundary': `<error-boundary message="This panel could not load.">
  <card style({ pad(4), animate: { enter: play(fade) } })>
    <p>A risky subtree. If a render throw happens inside,
       the fallback takes this card's place — Retry resets it.</p>
  </card>
</error-boundary>`,

  toast: `<toast tone="success">
  <strong>Saved.</strong> Your changes are live.
</toast>`,

  badge: `<row gap="2" align="center">
  <badge tone="success" style({ animate: { enter: play(fade) } })>
    Active</badge>
  <badge tone="warning">Pending</badge>
  <badge tone="destructive" variant="outline">Failed</badge>
  <badge tone="primary" variant="solid" pill>3</badge>
</row>`,

  chip: `<controller>
export default controller(() => {
  const kept = signal(true)
  return { kept, drop: () => kept.set(false) }
})
</controller>

<if {kept()}>
  <chip removable on-remove={drop}>Design</chip>
</if>
<if {!kept()}>
  <button variant="ghost" on:click={() => kept.set(true)}>bring it back</button>
</if>`,

  card: `<card interactive style({ pad(5) })>
  <strong>A surface</strong>
  <p style({ fg(muted-foreground).type({ size: sm }) })>
    Tone, padding and the hover lift are attributes.
  </p>
</card>`,

  avatar: `<row gap="2" align="center" style({ animate: { enter: play(fade) } })>
  <avatar name="Ada Lovelace"></avatar>
  <avatar initials="GH" tone="primary"></avatar>
  <avatar name="Alan Turing" size="lg" shape="square"></avatar>
</row>`,

  icon: `<row gap="3" align="center">
  <icon name="check" size="sm"></icon>
  <icon name="search"></icon>
  <icon name="refresh" spin label="Refreshing"></icon>
</row>`,

  separator: `<column gap="3">
  <p>Ships to 14 Harbor Lane</p>
  <separator></separator>
  <p>Bills to the same address</p>
</column>`,

  accordion: `<accordion style({ animate: { enter: play(fade) } })>
  <strong slot="trigger">Show the details</strong>
  <p>Animated open and close — and it remembers its state.</p>
</accordion>`,

  kbd: `<p>Press <kbd>Ctrl</kbd> + <kbd>K</kbd> to open the command palette.</p>`,

  'code-block': `<code-block
  code={'const n = signal(0)\nconst double = derived(() => n() * 2)'}
  lang="ts"
></code-block>`,

  prose: `<prose>
  <h2>Plain tags, already styled</h2>
  <p>Body copy sets the baseline — <strong>strong</strong>,
     <em>emphasis</em>, <code>inline_code()</code> and
     <mark>a highlight</mark> need no classes.</p>
  <blockquote>A quotation carries its own rule.</blockquote>
</prose>`,

  tooltip: `<tooltip content="Deletes the branch — cannot be undone"
 placement="top" style({ animate: { enter: play(fade) } })>
  <button tone="destructive" variant="outline">Delete branch</button>
</tooltip>`,

  'hover-card': `<button variant="ghost" popovertarget="profile">@ada</button>

<hover-card id="profile" style({ animate: { enter: play(fade) } })>
  <card style({ pad(4) })>
    <strong>Ada Lovelace</strong>
    <p style({ fg(muted-foreground).type({ size: sm }) })>
      wrote the first program.</p>
  </card>
</hover-card>`,

  popover: `<button popovertarget="pop">Open popover</button>

<popover id="pop" placement="bottom" style({ animate: { enter: play(fade) } })>
  <card style({ pad(4) })>
    Rides the platform top layer — no z-index war.</card>
</popover>`,

  dropdown: `<button popovertarget="menu">Actions ▾</button>

<dropdown id="menu" style({ animate: { enter: play(fade) } })>
  <menu-item>Edit</menu-item>
  <menu-item>Duplicate</menu-item>
  <menu-item tone="destructive">Delete</menu-item>
</dropdown>`,

  'context-menu': `<card id="canvas" style({ pad(6), animate: { enter: play(fade) } })>
  right-click target
</card>

<context-menu for="canvas">
  <menu-item value="cut">Cut</menu-item>
  <menu-item value="copy">Copy</menu-item>
  <menu-item value="paste">Paste</menu-item>
</context-menu>`,

  dialog: `<controller>
export default controller(() => {
  const gone = signal(false)
  return { gone, confirm: () => gone.set(true) }
})
</controller>

<button command="show-modal" commandfor="confirm" style({ animate: { press: play(fade) } })>
  Delete project…</button>

<dialog id="confirm">
  <header>Delete project</header>
  <section>This cannot be undone.</section>
  <footer>
    <button command="close" commandfor="confirm" variant="ghost">Cancel</button>
    <button command="close" commandfor="confirm"
            tone="destructive" variant="solid" on:click={confirm}>Delete</button>
  </footer>
</dialog>`,

  drawer: `<button command="show" commandfor="filters">Filters</button>

<drawer id="filters" side="end" size="md">
  <header>Filters</header>
  <section>Narrow the catalog.</section>
  <footer>
    <button command="close" commandfor="filters" variant="ghost">Cancel</button>
    <button command="close" commandfor="filters" tone="primary">Apply</button>
  </footer>
</drawer>`,

  resizable: `<resizable>
  <div slot="start">Start pane</div>
  <div slot="end">End pane</div>
</resizable>`,

  tabs: `<tabs>
  <tab title="Overview">
    <p>Northwind is live on two regions. Last deploy 14 minutes ago.</p>
  </tab>
  <tab title="Activity">
    <p>Ada shipped the billing fix. Sam opened a PR on the ingest worker.</p>
  </tab>
  <tab title="Settings">
    <p>Retention is 30 days. Notifications go to #ship-ops.</p>
  </tab>
</tabs>`,

  breadcrumb: `<breadcrumb style({ animate: { enter: play(fade) } })>
  <a href="/">Home</a>
  <a href="/components">Components</a>
  <small>breadcrumb</small>
</breadcrumb>`,

  pagination: `<controller>
export default controller(() => {
  const page = signal('2')
  return { page, go: (e) => page.set(e.page ?? e.value) }
})
</controller>

<pagination value={page()} on-page-change={(e) => go(e)}>
  <page-item value="1">1</page-item>
  <page-item value="2">2</page-item>
  <page-item value="3">3</page-item>
</pagination>
<p>on page {page()}</p>`,

  navbar: `<navbar bordered style({ animate: { enter: play(fade) } })>
  <strong slot="brand">SHIP</strong>
  <a href="/docs">Docs</a>
  <a href="/components">Components</a>
  <button slot="actions" size="sm" variant="outline">Sign in</button>
</navbar>`,

  table: `<controller>
export default controller(() => ({
  people: signal([
    { id: 1, name: 'Ada Lovelace', team: 'Engines', status: 'active' },
    { id: 2, name: 'Grace Hopper', team: 'Compilers', status: 'away' },
  ]),
}))
</controller>

<table searchable>
  <thead>
    <tr><th sortable>Name</th><th sortable>Team</th><th sortable>Status</th></tr>
  </thead>
  <tbody>
    <for {people() as p} key={p.id}>
      <tr>
        <td>{p.name}</td>
        <td>{p.team}</td>
        <td><badge tone={p.status === 'active' ? 'success' : 'muted'}>{p.status}</badge></td>
      </tr>
    </for>
  </tbody>
</table>`,

  layout: `<column gap="3">
  <row gap="2" align="center" justify="between">
    <strong>row — the x axis</strong>
    <badge tone="primary">end</badge>
  </row>
  <grid cols="3" gap="2">
    <card style({ pad(3) })>
      1</card>
    <card style({ pad(3) })>
      2</card>
    <card style({ pad(3) })>
      3</card>
  </grid>
  <center style({ pad(4).border({ color: border-subtle, width: 1 }).radius(md) })>
    <p>center — both axes</p>
  </center>
</column>`,

  'scroll-area': `<scroll-area orientation="vertical"
  style({ size({ max-h: overlay-max-h-sm }).pad(3).border({ color: border-subtle, width: 1 }).radius(md) })>
  <p>Constrain height/width with Style — overflow scrolls behind thin themed thumbs.</p>
  <p>orientation picks the axis (vertical · horizontal · both).</p>
  <p>Thumb from the theme; hover uses the stronger border colour.</p>
</scroll-area>`,

  // Action snippets (spotlight / targetCursor) moved to the action
  // table's docs surface; not in this element-detail snippets file.
}
