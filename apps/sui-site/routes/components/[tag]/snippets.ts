// /components/[tag] — the USAGE snippet each detail page shows. Every one is
// a complete single-file program (controller + markup where state is read)
// and compiles as written — checked by
// crates/sui/compiler/tests/site_snippets_compile.rs.
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

  badge: `<row gap="2" align="center">
  <badge tone="success" motion:enter={{ opacity: 'fade' }}>Active</badge>
  <badge tone="warning">Pending</badge>
  <badge tone="destructive" variant="outline">Failed</badge>
  <badge tone="primary" variant="solid" pill>3</badge>
</row>`,

  card: `<card sx="p:5" motion:hover={{ scale: 'spring:gentle' }}>
  <strong>Interactive</strong>
  <p sx="text:sm fg:muted-foreground">
    A surface. Tone, padding and motion are attributes.
  </p>
</card>`,

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
}
