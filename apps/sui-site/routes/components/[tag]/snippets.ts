// /components/[tag] — the USAGE snippet each detail page shows, one per entry
// on the full L1 surface (plus the two fx actions). Every one is a complete
// single-file program (controller + markup where state is read) and compiles
// as written — checked by crates/sui/compiler/tests/site_snippets_compile.rs.
export const usage: Record<string, string> = {
  button: `<controller>
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => {
    const saved = signal(false)
    return { saved, save: () => saved.set(true) }
  },
})
</controller>

<button variant="solid" tone="primary"
        on:click={save} motion:press={{ scale: 'press' }}>
  {saved() ? 'Saved ✓' : 'Save changes'}
</button>`,

  input: `<controller>
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => ({ name: signal('') }),
})
</controller>

<input placeholder="Ada Lovelace" bind:value={name} />
<p>hello, {name() || 'stranger'}</p>`,

  textarea: `<controller>
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => ({ bio: signal('') }),
})
</controller>

<textarea placeholder="Tell us about yourself" bind:value={bio}></textarea>
<p>{bio().length} characters</p>`,

  select: `<controller>
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => ({ team: signal('Design') }),
})
</controller>

<select bind:value={team}>
  <option>Engineering</option>
  <option>Design</option>
  <option>Research</option>
</select>
<p>team: {team()}</p>`,

  combobox: `<controller>
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => {
    const fw = signal('')
    return { fw, pick: (e) => fw.set(e.detail.value) }
  },
})
</controller>

<combobox placeholder="Pick a framework"
          on:sui-combobox-select={(e) => pick(e)}>
  <option value="sui">SUI</option>
  <option value="react">React</option>
  <option value="svelte">Svelte</option>
  <option value="solid">Solid</option>
</combobox>

<p>picked: {fw() || 'nothing yet'}</p>`,

  checkbox: `<controller>
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => ({ agreed: signal(false) }),
})
</controller>

<label><input type="checkbox" bind:checked={agreed} /> I agree to the terms</label>
<p>{agreed() ? 'thank you' : 'the checkbox is the platform one, skinned'}</p>`,

  'radio-group': `<radio-group label="Billing period" motion:enter={{ opacity: 'fade' }}>
  <label><input type="radio" name="plan" checked /> Monthly</label>
  <label><input type="radio" name="plan" /> Yearly</label>
</radio-group>`,

  switch: `<controller>
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => ({ notify: signal(true) }),
})
</controller>

<row gap="3" align="center">
  <switch bind:checked={notify}></switch>
  <p>{notify() ? 'Notifications on' : 'Muted'}</p>
</row>`,

  slider: `<controller>
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => ({ load: signal(62) }),
})
</controller>

<slider bind:value={load} min="0" max="100" tone="primary"></slider>
<progress value={load()} max="100"></progress>
<p>Two components, one signal — drag it: {load()}%</p>`,

  'toggle-group': `<toggle-group variant="segmented" motion:enter={{ opacity: 'fade' }}>
  <toggle value="left">Left</toggle>
  <toggle value="center">Center</toggle>
  <toggle value="right">Right</toggle>
</toggle-group>`,

  field: `<field motion:enter={{ opacity: 'fade' }}>
  <label for="name">Name</label>
  <input id="name" placeholder="Ada Lovelace" />
</field>

<field invalid>
  <label for="email">Email</label>
  <input id="email" value="not-an-email" />
</field>`,

  label: `<column gap="1" motion:enter={{ opacity: 'fade' }}>
  <label for="email" required>Email</label>
  <input id="email" placeholder="ada@lovelace.dev" />
</column>`,

  fieldset: `<fieldset motion:enter={{ opacity: 'fade' }}>
  <legend>Account details</legend>
  <column gap="2">
    <input placeholder="Name" />
    <input placeholder="Email" />
  </column>
</fieldset>`,

  dropzone: `<card dropzone sx="p:6" motion:enter={{ opacity: 'fade' }}>
  Drop a file, or click to browse
</card>`,

  'theme-picker': `<!-- the whole switcher: trigger, panel, live previews -->
<theme-picker motion:enter={{ opacity: 'fade' }}></theme-picker>`,

  alert: `<controller>
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => {
    const visible = signal(true)
    return { visible, dismiss: () => visible.set(false) }
  },
})
</controller>

{#if visible()}
  <alert tone="warning" motion:exit={{ opacity: 'fade-out' }}>
    <strong>Careful.</strong> This cannot be undone.
  </alert>
{/if}
<button on:click={dismiss} variant="ghost">Dismiss</button>`,

  progress: `<controller>
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => ({ done: signal(64) }),
})
</controller>

<progress value={done()} max="100"></progress>
<p>{done()}% uploaded — a native element, themed</p>`,

  meter: `<column gap="2" motion:enter={{ opacity: 'fade' }}>
  <meter value="0.3"></meter>
  <meter value="0.7"></meter>
  <meter value="0.95"></meter>
</column>`,

  spinner: `<controller>
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => ({ loading: signal(true) }),
})
</controller>

{#if loading()}
  <row gap="2" align="center">
    <spinner tone="primary"></spinner>
    <p>fetching…</p>
  </row>
{/if}`,

  skeleton: `<row gap="3" align="center" motion:enter={{ opacity: 'fade' }}>
  <span skeleton shape="circle" sx="w:7 h:7"></span>
  <column gap="2" sx="grow:1">
    <span skeleton shape="text" size="sm"></span>
    <span skeleton shape="text" size="sm"></span>
  </column>
</row>`,

  'empty-state': `<empty-state motion:enter={{ opacity: 'fade' }}>
  <strong slot="title">No results</strong>
  Try a different search, or clear a filter.
</empty-state>`,

  'error-state': `<error-state code="ERR_TIMEOUT" motion:enter={{ opacity: 'fade' }}>
  The request timed out. Retry, or check the connection.
</error-state>`,

  'error-boundary': `<error-boundary message="This panel could not load.">
  <card sx="p:4" motion:enter={{ opacity: 'fade' }}>
    <p>A risky subtree. If a render throw happens inside,
       the fallback takes this card's place — Retry resets it.</p>
  </card>
</error-boundary>`,

  toast: `<toast-region position="bottom-end" motion:enter={{ opacity: 'fade' }}>
  <toast tone="success" duration="0">
    <strong>Saved.</strong> Your changes are live.
    <button data-toast-close variant="ghost" size="sm">×</button>
  </toast>
</toast-region>`,

  badge: `<row gap="2" align="center">
  <badge tone="success" motion:enter={{ opacity: 'fade' }}>Active</badge>
  <badge tone="warning">Pending</badge>
  <badge tone="destructive" variant="outline">Failed</badge>
  <badge tone="primary" variant="solid" pill>3</badge>
</row>`,

  chip: `<controller>
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => {
    const kept = signal(true)
    return { kept, drop: () => kept.set(false) }
  },
})
</controller>

{#if kept()}
  <chip removable on:sui-remove={drop}>Backspace or × removes me</chip>
{/if}
{#if !kept()}
  <button variant="ghost" on:click={() => kept.set(true)}>bring it back</button>
{/if}`,

  'status-dot': `<row gap="2" align="center" motion:enter={{ opacity: 'fade' }}>
  <status-dot tone="success"></status-dot>
  <p>all systems live</p>
</row>`,

  card: `<card sx="p:5" motion:hover={{ scale: 'spring:gentle' }}>
  <strong>Interactive</strong>
  <p sx="text:sm fg:muted-foreground">
    A surface. Tone, padding and motion are attributes.
  </p>
</card>`,

  avatar: `<row gap="2" align="center" motion:enter={{ opacity: 'fade' }}>
  <avatar name="Ada Lovelace"></avatar>
  <avatar initials="GH" tone="primary"></avatar>
  <avatar name="Alan Turing" size="lg" shape="square"></avatar>
</row>`,

  icon: `<row gap="3" align="center" motion:enter={{ opacity: 'fade' }}>
  <icon>
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
         stroke-width="2" stroke-linecap="round"><path d="M20 6 9 17l-5-5" /></svg>
  </icon>
  <icon spin>
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
         stroke-width="2" stroke-linecap="round"><path d="M21 12a9 9 0 1 1-6.2-8.56" /></svg>
  </icon>
</row>`,

  separator: `<column gap="2" motion:enter={{ opacity: 'fade' }}>
  <p>above</p>
  <separator></separator>
  <p>below — and vertical, between inline things:</p>
  <row gap="2" align="center">
    <span>A</span>
    <separator orientation="vertical"></separator>
    <span>B</span>
  </row>
</column>`,

  hr: `<column gap="3" motion:enter={{ opacity: 'fade' }}>
  <hr />
  <hr tone="primary" />
  <hr variant="dashed" />
</column>`,

  details: `<details motion:enter={{ opacity: 'fade' }}>
  <summary>Advanced options</summary>
  <p>Native disclosure. Zero JavaScript.</p>
</details>`,

  collapsible: `<collapsible motion:enter={{ opacity: 'fade' }}>
  <strong slot="trigger">Show the details</strong>
  <p>Animated open and close — and it remembers its state.</p>
</collapsible>`,

  prose: `<column gap="2" motion:enter={{ opacity: 'fade' }}>
  <h2>Plain tags, already styled</h2>
  <p>Body copy sets the baseline — <strong>strong</strong>,
     <em>emphasis</em>, <code>inline_code()</code> and
     <mark>a highlight</mark> need no classes.</p>
  <blockquote>A quotation carries its own rule.</blockquote>
</column>`,

  tooltip: `<tooltip content="Deletes the branch — cannot be undone"
         placement="top" motion:enter={{ opacity: 'fade' }}>
  <button tone="destructive" variant="outline">Delete branch</button>
</tooltip>`,

  'hover-card': `<button variant="ghost" popovertarget="profile">@ada</button>

<hover-card id="profile" motion:enter={{ opacity: 'fade' }}>
  <card sx="p:4">
    <strong>Ada Lovelace</strong>
    <p sx="text:sm fg:muted-foreground">wrote the first program.</p>
  </card>
</hover-card>`,

  popover: `<button popovertarget="pop">Open popover</button>

<popover id="pop" placement="bottom" motion:enter={{ opacity: 'fade' }}>
  <card sx="p:4">Rides the platform top layer — no z-index war.</card>
</popover>`,

  dropdown: `<button popovertarget="menu">Actions ▾</button>

<dropdown id="menu" motion:enter={{ opacity: 'fade' }}>
  <menu-item>Edit</menu-item>
  <menu-item>Duplicate</menu-item>
  <menu-item tone="destructive">Delete</menu-item>
</dropdown>`,

  'context-menu': `<card id="canvas" sx="p:6" motion:enter={{ opacity: 'fade' }}>
  right-click me
</card>

<context-menu for="canvas">
  <menu-item value="cut">Cut</menu-item>
  <menu-item value="copy">Copy</menu-item>
  <menu-item value="paste">Paste</menu-item>
</context-menu>`,

  dialog: `<controller>
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => {
    const gone = signal(false)
    return { gone, confirm: () => gone.set(true) }
  },
})
</controller>

<button command="show-modal" commandfor="confirm"
        motion:press={{ scale: 'press' }}>Delete project…</button>

<dialog id="confirm">
  <header>Delete project</header>
  <section>This cannot be undone.</section>
  <footer>
    <button command="close" commandfor="confirm" variant="ghost">Cancel</button>
    <button command="close" commandfor="confirm"
            tone="destructive" variant="solid" on:click={confirm}>Delete</button>
  </footer>
</dialog>`,

  tabs: `<tabs variant="pill" tone="primary">
  <tab-list>
    <tab>Overview</tab>
    <tab>Activity</tab>
    <tab>Settings</tab>
  </tab-list>
  <tab-panel motion:enter={{ opacity: 'fade' }}>
    <p>Keyboard-complete: arrows rove, Home/End jump.</p>
  </tab-panel>
  <tab-panel><p>Activity feed.</p></tab-panel>
  <tab-panel><p>Settings.</p></tab-panel>
</tabs>`,

  breadcrumb: `<breadcrumb motion:enter={{ opacity: 'fade' }}>
  <a href="/">Home</a>
  <a href="/components">Components</a>
  <small>breadcrumb</small>
</breadcrumb>`,

  pagination: `<controller>
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => {
    const page = signal('2')
    return { page, go: (e) => page.set(e.detail.value) }
  },
})
</controller>

<pagination value={page()} on:sui-page-change={(e) => go(e)}>
  <page value="1">1</page>
  <page value="2">2</page>
  <page value="3">3</page>
</pagination>
<p>on page {page()}</p>`,

  navbar: `<navbar bordered motion:enter={{ opacity: 'fade' }}>
  <strong slot="brand">SHIP</strong>
  <a href="/docs">Docs</a>
  <a href="/components">Components</a>
  <button slot="actions" size="sm" variant="outline">Sign in</button>
</navbar>`,

  table: `<controller>
import { controller, signal } from '@ship/sui-runtime'

export default controller({
  scope: () => ({
    people: signal([
      { id: 1, name: 'Ada Lovelace', team: 'Engines', status: 'active' },
      { id: 2, name: 'Grace Hopper', team: 'Compilers', status: 'away' },
    ]),
  }),
})
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

  layout: `<column gap="3" motion:enter={{ opacity: 'fade' }}>
  <row gap="2" align="center" justify="between">
    <strong>row — the x axis</strong>
    <badge tone="primary">end</badge>
  </row>
  <grid cols="3" gap="2">
    <card sx="p:3">1</card>
    <card sx="p:3">2</card>
    <card sx="p:3">3</card>
  </grid>
  <center sx="p:4 border:subtle radius:md">
    <p>center — both axes</p>
  </center>
</column>`,

  'scroll-area': `<scroll-area sx="max-h:panel-max-h-sm p:3 border:subtle radius:md"
             motion:enter={{ opacity: 'fade' }}>
  <p>Constrain it, and the overflow scrolls behind thin,
     themed scrollbars…</p>
  <p>…line after line…</p>
  <p>…after line.</p>
</scroll-area>`,

  spotlight: `<controller>
import { controller } from '@ship/sui-runtime'
import { spotlight } from '@ship/sui-fx'

export default controller({
  scope: () => ({ spotlight }),
})
</controller>

<div use:spotlight sx="p:8 radius:lg border:border">
  <p>move the pointer — a spotlight follows it</p>
</div>`,

}
