// Code snippets for the Data binding page. Kept OUT of the controller so the
// compiler's textual scope scan isn't fooled by the `controller({ ... })` /
// `on:` / `scope:` patterns inside these sample strings. Each .sui sample is
// compile-checked against the in-repo sui-compiler.
export const code = {
  twoway: `<!-- bind:value keeps a signal and an input in sync, both ways. -->
<input type="text" bind:value={name} />
<label>
  <input type="checkbox" bind:checked={agreed} /> I agree
</label>
<p>Hello {name() || "stranger"} — agreed: {agreed() ? "yes" : "no"}</p>`,

  events: `<!-- on:<event> wires a handler; write signals from inside it. -->
<button on:click={() => count.set((n) => n + 1)}>
  clicked {count()} times
</button>`,

  // An ACTION is the element handle. It receives the node, does its work, and
  // returns a `destroy` — so the element never becomes a value you have to store
  // in scope, thread through a template, and remember to release.
  actions: `<!-- use: hands the element to a function. Nothing to store. -->
<textarea use:autosize></textarea>
<div use:clickOutside={close}>…</div>`,

  // Authoring one is the whole API: a function over an element.
  authoring: `// autofocus.ts — an action is a plain function over an element
import type { Action } from '@ship/sui-runtime'

export const autofocus: Action<void> = (node) => {
  const el = node as HTMLElement
  const id = requestAnimationFrame(() => el.focus())
  // Return the cleanup; the runtime calls it when the element goes away.
  return { destroy: () => cancelAnimationFrame(id) }
}`,

  // The escape hatch, for when you genuinely need the node as a VALUE.
  refs: `<!-- bind:this fills a ref() from the controller; read it like a signal -->
<input bind:this={field} />
<button on:click={() => field().focus()}>Focus the input</button>`,
}
