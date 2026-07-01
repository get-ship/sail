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

  refs: `// controller.ts — ref() holds an element handle
import { controller, ref } from '@ship/sui-runtime'
export default controller({
  scope: () => ({ field: ref() }),
})

<!-- template — bind:this fills the ref; read it with field() -->
<input bind:this={field} />
<button on:click={() => field().focus()}>Focus the input</button>`,
}
