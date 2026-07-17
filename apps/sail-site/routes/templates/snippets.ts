// Snippets for the templates page. Every SUI sample is compiled by the build.
export const code = {
  anatomy: `<!-- one file, three parts. -->
<scope>
  props: {
    name = ''                 /* what comes IN (attributes/props)      */
  }
</scope>

<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller(() => {              /* what the template may BIND            */
  const count = signal(0)
  return { count }
})
</controller>

<button on:click={() => count.set(count() + 1)}>
  {name()} clicked {count()} times   <!-- the template: a pure read -->
</button>`,

  read: `<!-- a signal, read straight into the markup -->
<p>Signed in as {user().name}.</p>
<button disabled={saving()}>Save</button>`,

  write: `<!-- the template READS; handlers WRITE. That separation is the whole
     mental model, and it is why you never wonder where a value changed. -->
<button on:click={() => count.set((n) => n + 1)}>
  clicked {count()} times
</button>`,

  twoWay: `<!-- bind: is the ONE place a read and a write happen at once —
     and it is exactly sugar for the read plus an on:input write. -->
<input bind:value={draft} />
<input type="checkbox" bind:checked={agreed} />`,

  modifiers: `<!-- modifiers declare intent instead of boilerplate -->
<form on:submit|prevent={save}>…</form>
<input on:input|debounce:200={search} />
<div on:keydown|key:Escape={close}>…</div>`,

  ifBlock: `{#if user())}
  <p>Welcome back.</p>
{:else}
  <a href="/login">Sign in</a>
{/if}`,

  forBlock: `<!-- key is REQUIRED. Without it a reorder is a re-render, and the
     input you were typing in loses its cursor. -->
{#for row of rows() key={row.id}}
  <tr><td>{row.name}</td></tr>
{/for}`,

  matchBlock: `{#match status()}
  {:case 'draft'} <badge tone="muted">Draft</badge>
  {:case 'live'}  <badge tone="success">Live</badge>
  {:default}      <badge>Unknown</badge>
{/match}`,
}
