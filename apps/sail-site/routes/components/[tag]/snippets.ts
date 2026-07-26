// /components/[tag] — the USAGE snippet each detail page shows, one per entry
// on the full L1 surface. Every one is a complete
// single-file program (controller + markup where state is read) and compiles
// as written — checked by crates/sail/compiler/tests/site_snippets_compile.rs.
export const usage: Record<string, string> = {
  button: `<controller>
export default controller(() => {
  const saved = signal(false)
  return { saved, save: () => saved.set(true) }
})
</controller>

<button variant="solid" tone="primary"
        on:click={save} fx:press={{ scale: 'press' }}>
  {saved() ? 'Saved ✓' : 'Save changes'}
</button>`,

  input: `<controller>
export default controller(() => ({ name: signal('') }))
</controller>

<input placeholder="Ada Lovelace" bind:value={name} />
<p>hello, {name() || 'stranger'}</p>`,

  textarea: `<controller>
export default controller(() => ({ bio: signal('') }))
</controller>

<textarea placeholder="Tell us about yourself" bind:value={bio}></textarea>
<p>{bio().length} characters</p>`,

  select: `<controller>
export default controller(() => ({ team: signal('Design') }))
</controller>

<select bind:value={team}>
  <option>Engineering</option>
  <option>Design</option>
  <option>Research</option>
</select>
<p>team: {team()}</p>`,

  combobox: `<controller>
export default controller(() => {
  const fw = signal('')
  return { fw, pick: (e) => fw.set(e.detail.value) }
})
</controller>

<combobox placeholder="Pick a framework"
          on:sail-combobox-select={(e) => pick(e)}>
  <option value="sail">Sail</option>
  <option value="react">React</option>
  <option value="svelte">Svelte</option>
  <option value="solid">Solid</option>
</combobox>

<p>picked: {fw() || 'nothing yet'}</p>`,

  checkbox: `<controller>
export default controller(() => ({ agreed: signal(false) }))
</controller>

<label><input type="checkbox" bind:checked={agreed} /> I agree to the terms</label>
<p>{agreed() ? 'thank you' : 'the checkbox is the platform one, skinned'}</p>`,

  'radio-group': `<radio-group label="Billing period" fx:enter={{ opacity: 'fade' }}>
  <label><input type="radio" name="plan" checked /> Monthly</label>
  <label><input type="radio" name="plan" /> Yearly</label>
</radio-group>`,

  switch: `<controller>
export default controller(() => ({ notify: signal(true) }))
</controller>

<row gap="3" align="center">
  <switch bind:checked={notify}></switch>
  <p>{notify() ? 'Notifications on' : 'Muted'}</p>
</row>`,

  slider: `<controller>
export default controller(() => ({ load: signal(62) }))
</controller>

<slider bind:value={load} min="0" max="100" tone="primary"></slider>
<progress value={load()} max="100"></progress>
<p>Two components, one signal — drag it: {load()}%</p>`,

  'toggle-group': `<toggle-group variant="segmented" fx:enter={{ opacity: 'fade' }}>
  <toggle value="left">Left</toggle>
  <toggle value="center">Center</toggle>
  <toggle value="right">Right</toggle>
</toggle-group>`,

  field: `<field fx:enter={{ opacity: 'fade' }}>
  <label for="name">Name</label>
  <input id="name" placeholder="Ada Lovelace" />
</field>

<field invalid>
  <label for="email">Email</label>
  <input id="email" value="not-an-email" />
</field>`,

  label: `<column gap="1" fx:enter={{ opacity: 'fade' }}>
  <label for="email" required>Email</label>
  <input id="email" placeholder="ada@lovelace.dev" />
</column>`,

  fieldset: `<fieldset fx:enter={{ opacity: 'fade' }}>
  <legend>Account details</legend>
  <column gap="2">
    <input placeholder="Name" />
    <input placeholder="Email" />
  </column>
</fieldset>`,

  dropzone: `<card dropzone sx={style.pad(6)} fx:enter={{ opacity: 'fade' }}>
  Drop a file, or click to browse
</card>`,

  'theme-picker': `<!-- the whole switcher: trigger, panel, live previews -->
<theme-picker fx:enter={{ opacity: 'fade' }}></theme-picker>`,

  alert: `<controller>
export default controller(() => {
  const visible = signal(true)
  return { visible, dismiss: () => visible.set(false) }
})
</controller>

{#if visible()}
  <alert tone="warning" fx:exit={{ opacity: 'fade-out' }}>
    <strong>Careful.</strong> This cannot be undone.
  </alert>
{/if}
<button on:click={dismiss} variant="ghost">Dismiss</button>`,

  progress: `<controller>
export default controller(() => ({ done: signal(64) }))
</controller>

<progress value={done()} max="100"></progress>
<p>{done()}% uploaded — a native element, themed</p>`,

  meter: `<column gap="2" fx:enter={{ opacity: 'fade' }}>
  <meter value="0.3"></meter>
  <meter value="0.7"></meter>
  <meter value="0.95"></meter>
</column>`,

  spinner: `<controller>
export default controller(() => ({ loading: signal(true) }))
</controller>

{#if loading()}
  <row gap="2" align="center">
    <spinner tone="primary"></spinner>
    <p>fetching…</p>
  </row>
{/if}`,

  skeleton: `<row gap="3" align="center" fx:enter={{ opacity: 'fade' }}>
  <span skeleton shape="circle" sx={style.w(7).h(7)}></span>
  <column gap="2" sx={style.grow(1)}>
    <span skeleton shape="text" size="sm"></span>
    <span skeleton shape="text" size="sm"></span>
  </column>
</row>`,

  'empty-state': `<empty-state fx:enter={{ opacity: 'fade' }}>
  <strong slot="title">No results</strong>
  Try a different search, or clear a filter.
</empty-state>`,

  'error-state': `<error-state code="ERR_TIMEOUT" fx:enter={{ opacity: 'fade' }}>
  The request timed out. Retry, or check the connection.
</error-state>`,

  'error-boundary': `<error-boundary message="This panel could not load.">
  <card sx={style.pad(4)} fx:enter={{ opacity: 'fade' }}>
    <p>A risky subtree. If a render throw happens inside,
       the fallback takes this card's place — Retry resets it.</p>
  </card>
</error-boundary>`,

  toast: `<toast-region position="bottom-end" fx:enter={{ opacity: 'fade' }}>
  <toast tone="success" duration="0">
    <strong>Saved.</strong> Your changes are live.
    <button data-toast-close variant="ghost" size="sm">×</button>
  </toast>
</toast-region>`,

  badge: `<row gap="2" align="center">
  <badge tone="success" fx:enter={{ opacity: 'fade' }}>Active</badge>
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

{#if kept()}
  <chip removable on:sail-remove={drop}>Design</chip>
{/if}
{#if !kept()}
  <button variant="ghost" on:click={() => kept.set(true)}>bring it back</button>
{/if}`,

  'status-dot': `<row gap="2" align="center" fx:enter={{ opacity: 'fade' }}>
  <status-dot tone="success"></status-dot>
  <p>all systems live</p>
</row>`,

  card: `<card interactive sx={style.pad(5)}>
  <strong>A surface</strong>
  <p sx={style.fontSize(sm).fg(muted-foreground)}>
    Tone, padding and the hover lift are attributes.
  </p>
</card>`,

  avatar: `<row gap="2" align="center" fx:enter={{ opacity: 'fade' }}>
  <avatar name="Ada Lovelace"></avatar>
  <avatar initials="GH" tone="primary"></avatar>
  <avatar name="Alan Turing" size="lg" shape="square"></avatar>
</row>`,

  icon: `<row gap="3" align="center" fx:enter={{ opacity: 'fade' }}>
  <icon>
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
         stroke-width="2" stroke-linecap="round"><path d="M20 6 9 17l-5-5" /></svg>
  </icon>
  <icon spin>
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
         stroke-width="2" stroke-linecap="round"><path d="M21 12a9 9 0 1 1-6.2-8.56" /></svg>
  </icon>
</row>`,

  separator: `<column gap="2" fx:enter={{ opacity: 'fade' }}>
  <p>above</p>
  <separator></separator>
  <p>below — and vertical, between inline things:</p>
  <row gap="2" align="center">
    <span>A</span>
    <separator orientation="vertical"></separator>
    <span>B</span>
  </row>
</column>`,

  hr: `<column gap="3" fx:enter={{ opacity: 'fade' }}>
  <hr />
  <hr tone="primary" />
  <hr variant="dashed" />
</column>`,

  details: `<details fx:enter={{ opacity: 'fade' }}>
  <summary>Advanced options</summary>
  <p>Native disclosure. Zero JavaScript.</p>
</details>`,

  collapsible: `<collapsible fx:enter={{ opacity: 'fade' }}>
  <strong slot="trigger">Show the details</strong>
  <p>Animated open and close — and it remembers its state.</p>
</collapsible>`,

  prose: `<column gap="2" fx:enter={{ opacity: 'fade' }}>
  <h2>Plain tags, already styled</h2>
  <p>Body copy sets the baseline — <strong>strong</strong>,
     <em>emphasis</em>, <code>inline_code()</code> and
     <mark>a highlight</mark> need no classes.</p>
  <blockquote>A quotation carries its own rule.</blockquote>
</column>`,

  tooltip: `<tooltip content="Deletes the branch — cannot be undone"
         placement="top" fx:enter={{ opacity: 'fade' }}>
  <button tone="destructive" variant="outline">Delete branch</button>
</tooltip>`,

  'hover-card': `<button variant="ghost" popovertarget="profile">@ada</button>

<hover-card id="profile" fx:enter={{ opacity: 'fade' }}>
  <card sx={style.pad(4)}>
    <strong>Ada Lovelace</strong>
    <p sx={style.fontSize(sm).fg(muted-foreground)}>wrote the first program.</p>
  </card>
</hover-card>`,

  popover: `<button popovertarget="pop">Open popover</button>

<popover id="pop" placement="bottom" fx:enter={{ opacity: 'fade' }}>
  <card sx={style.pad(4)}>Rides the platform top layer — no z-index war.</card>
</popover>`,

  dropdown: `<button popovertarget="menu">Actions ▾</button>

<dropdown id="menu" fx:enter={{ opacity: 'fade' }}>
  <menu-item>Edit</menu-item>
  <menu-item>Duplicate</menu-item>
  <menu-item tone="destructive">Delete</menu-item>
</dropdown>`,

  'context-menu': `<card id="canvas" sx={style.pad(6)} fx:enter={{ opacity: 'fade' }}>
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

<button command="show-modal" commandfor="confirm"
        fx:press={{ scale: 'press' }}>Delete project…</button>

<dialog id="confirm">
  <header>Delete project</header>
  <section>This cannot be undone.</section>
  <footer>
    <button command="close" commandfor="confirm" variant="ghost">Cancel</button>
    <button command="close" commandfor="confirm"
            tone="destructive" variant="solid" on:click={confirm}>Delete</button>
  </footer>
</dialog>`,

  tabs: `<tabs tone="primary">
  <tab-list variant="pill">
    <tab>Overview</tab>
    <tab>Activity</tab>
    <tab>Settings</tab>
  </tab-list>
  <tab-panel fx:enter={{ opacity: 'fade' }}>
    <p>Keyboard-complete: arrows rove, Home/End jump.</p>
  </tab-panel>
  <tab-panel><p>Activity feed.</p></tab-panel>
  <tab-panel><p>Settings.</p></tab-panel>
</tabs>`,

  breadcrumb: `<breadcrumb fx:enter={{ opacity: 'fade' }}>
  <a href="/">Home</a>
  <a href="/components">Components</a>
  <small>breadcrumb</small>
</breadcrumb>`,

  pagination: `<controller>
export default controller(() => {
  const page = signal('2')
  return { page, go: (e) => page.set(e.detail.value) }
})
</controller>

<pagination value={page()} on:sail-page-change={(e) => go(e)}>
  <page value="1">1</page>
  <page value="2">2</page>
  <page value="3">3</page>
</pagination>
<p>on page {page()}</p>`,

  navbar: `<navbar bordered fx:enter={{ opacity: 'fade' }}>
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

<table>
  <thead>
    <tr><th>Name</th><th>Team</th><th>Status</th></tr>
  </thead>
  <tbody>
    {#for p of people() key={p.id}}
      <tr>
        <td>{p.name}</td>
        <td>{p.team}</td>
        <td><badge tone={p.status === 'active' ? 'success' : 'muted'}>{p.status}</badge></td>
      </tr>
    {/for}
  </tbody>
</table>`,

  layout: `<column gap="3" fx:enter={{ opacity: 'fade' }}>
  <row gap="2" align="center" justify="between">
    <strong>row — the x axis</strong>
    <badge tone="primary">end</badge>
  </row>
  <grid cols="3" gap="2">
    <card sx={style.pad(3)}>1</card>
    <card sx={style.pad(3)}>2</card>
    <card sx={style.pad(3)}>3</card>
  </grid>
  <center sx={style.pad(4).borderWidth(1).border(border-subtle).radius(md)}>
    <p>center — both axes</p>
  </center>
</column>`,

  'scroll-area': `<scroll-area sx={style.maxH(panel-max-h-sm).pad(3).borderWidth(1).border(border-subtle).radius(md)}
             fx:enter={{ opacity: 'fade' }}>
  <p>Constrain it, and the overflow scrolls behind thin,
     themed scrollbars…</p>
  <p>…line after line…</p>
  <p>…after line.</p>
</scroll-area>`,

  // Action snippets (spotlight / targetCursor) moved to the action
  // table's docs surface; not in this element-detail snippets file.
}
