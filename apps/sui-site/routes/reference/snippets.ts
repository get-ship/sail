// Reference cheat-sheets. Every construct here is verified against the in-repo
// sui-compiler (the directives/bindings compile; the runtime signatures match
// @ship/sui-runtime). Rendered as codeblocks so the braces read as text.
export const code = {
  text: `{expr}                        read a signal or any expression
{name() || 'fallback'}        expressions are plain JS`,

  events: `on:click={handler}            attach a DOM event
on:input={(e) => …}           the event is the argument
on:submit|prevent={save}      modifier: preventDefault first
on:click|stop={fn}            modifier: stopPropagation
on:click|self={fn}            fire only when target === currentTarget
on:input|debounce:200={fn}    modifier: debounce by N ms`,

  bind: `bind:value={sig}              two-way: text input <-> signal
bind:checked={sig}            two-way: checkbox <-> signal
bind:this={ref}               element handle (ref() in the controller)`,

  attrs: `class:active={cond}           toggle a class when cond is truthy
style:--gap={value}           set a CSS custom property
prop:name={value}             pass a prop down to a child component`,

  controlFlow: `{#if cond} … {:else if cond} … {:else} … {/if}

{#for item of list() key={item.id}} … {/for}

{#match expr()}
  {:case 'a'} …
  {:case 'b'} …
  {:default} …
{/match}`,

  asyncData: `<!-- render the three states of a live() / data-live handle -->
{#while-loading of users} … {/while-loading}
{#while-error err of users} {err.message} {/while-error}
{#if users().kind === 'ok'} … users().value … {/if}`,

  runtime: `import { signal, derived, effect, batch, untrack, onCleanup } from '@ship/sui-runtime'

const count = signal(0)              // () to read, .set(v) / .set(fn) to write
const doubled = derived(() => count() * 2)   // auto-tracked, lazy, cached
const dispose = effect(() => { … return cleanup })  // runs on dep change

batch(() => { … })                   // coalesce writes into one update
untrack(() => sig())                 // read without subscribing
onCleanup(() => …)                   // teardown on unmount`,

  declarations: `<!-- component props: name, type, default, enum values -->
<prop name="tone" values="default|primary|accent" default="default" />
<prop name="count" type="number" default="0" />

<!-- scoped styles: :host + shadow tree, reads @ship/tokens -->
<style>
  :host { display: block; color: var(--foreground); }
  :host([tone="primary"]) { border-color: var(--primary); }
</style>

<!-- live server data (compiles to the runtime data() SSE handle) -->
data live users = ship.users.list()`,

  slots: `<!-- component: declare named + default slots -->
<slot name="title"></slot>
<slot></slot>

<!-- consumer: fill by name; loose children -> default -->
<h2 slot="title">…</h2>`,
}
