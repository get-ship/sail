// Snippets for /sail/language/templates. Every SAIL sample is compiled by the
// build (site_snippets_compile) — nothing here is pseudo-code.
export const code = {
  read: `<p>Signed in as {user().name}.</p>
<button disabled={saving()}>Save</button>`,

  ifBlock: `{#if user()}
  <p>Welcome back.</p>
{:else if pending()}
  <spinner></spinner>
{:else}
  <a href="/login">Sign in</a>
{/if}`,

  matchBlock: `{#match status()}
  {:case 'draft'} <badge tone="muted">Draft</badge>
  {:case 'live'}  <badge tone="success">Live</badge>
  {:default}      <badge>Unknown</badge>
{/match}`,

  forBlock: `{#for row of rows() key={row.id}}
  <tr><td>{row.name}</td></tr>
{/for}`,

  forKeyError: `error: [sail-for-key] {#for} block has no \`key=\` — the reconciler
  will fall back to positional matching and DOM identity (focus, selection,
  in-place edits) will be lost on reorder.
  suggest: Add \`key={EXPR}\` to the for-block header, where EXPR returns a
  stable unique value per item — e.g. \`{#for item of items key={item.id}}\`.`,

  events: `<button on:click={() => count.set((n) => n + 1)}>
  clicked {count()} times
</button>`,

  modifiers: `<form on:submit|prevent={save}>...</form>
<input on:input|debounce:200={search} />
<div on:keydown|key:Escape={close}>...</div>`,

  twoWay: `<input bind:value={draft} />
<input type="checkbox" bind:checked={agreed} />`,

  slotDef: `<div class="label"><slot name="label"></slot></div>
<div class="control"><slot></slot></div>
<div class="hint msg"><slot name="hint"></slot></div>`,

  slotUse: `<field>
  <span slot="label">Email</span>
  <input type="email" bind:value={email} />
  <span slot="hint">We never share it.</span>
</field>`,
}
