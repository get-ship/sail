// Snippets for /sail/language/templates. Every Sail sample is compiled by the
// build (site_snippets_compile) — nothing here is pseudo-code.
export const code = {
  read: `<p>Signed in as {user().name}.</p>
<button disabled={saving()}>Save</button>`,

  ifBlock: `<if {user()}>
  <p>Welcome back.</p>
{else-if {pending()} => <spinner></spinner>}{else => <a href="/login">Sign in</a>}</if>`,

  matchBlock: `<match {status()}>
  {'draft' => <badge tone="muted">Draft</badge>}{'live' => <badge tone="success">Live</badge>}{else => <badge>Unknown</badge>}</match>`,

  forBlock: `<for {rows() as row} key={row.id}>
  <tr><td>{row.name}</td></tr>
</for>`,

  events: `<button on:click={() => count.set((n) => n + 1)}>
  clicked {count()} times
</button>`,

  modifiers: `<form on:submit|prevent={save}>...</form>
<input on:input|debounce:200={search} />
<dialog on:keydown|key:Escape={close}>...</dialog>`,

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
