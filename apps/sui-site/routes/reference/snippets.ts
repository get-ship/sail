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

  runtime: `import { signal, derived, effect, batch, untrack, onCleanup, selector, debounce } from '@ship/sui-runtime'

const count = signal(0)              // () to read, .set(v) / .set(fn) to write
const doubled = derived(() => count() * 2)   // auto-tracked, lazy, cached
const dispose = effect(() => { … return cleanup })  // runs on dep change

batch(() => { … })                   // coalesce writes into one update pass
untrack(() => sig())                 // read without subscribing
onCleanup(() => …)                   // teardown on unmount
const isActive = selector(activeId)  // isActive(id): O(1) membership, no over-notify
const q = debounce(query, 200)       // a derived signal that lags by N ms`,

  liveHelpers: `import { live, unwrap, unwrapOr } from '@ship/sui-runtime'

const users = live(() => fetch('/api/users').then((r) => r.json()))
// users() -> { kind: 'loading' | 'error' | 'ok', value?, error? }
unwrap(users)              // the value, or undefined until 'ok'
unwrapOr(users, [])        // the value, or a fallback`,

  hooks: `import { useForm, useFilteredList, useSelection } from '@ship/sui-runtime'

// controller hooks — composable state bundles, spread into scope()
useForm({ initial: { name: '', email: '' }, submit })  // + pending + lastError
useFilteredList(items, (item, q) => item.name.includes(q))  // + { query }
useSelection(items, (item) => item.id)                      // + { selectedId }`,

  context: `import { shared, provide, useShared } from '@ship/sui-runtime'

// share a signal down a subtree, no prop drilling
provide(host, 'theme', shared(themeSignal))   // in a parent's mount(state, host)
const theme = useShared(host, 'theme')        // in a descendant's mount`,

  advanced: `import { defineComponent, morph } from '@ship/sui-runtime'

defineComponent('sui-widget', schema, factory)  // register a custom element
//   the compiler emits this — you rarely call it by hand
morph(targetEl, htmlOrElement)   // patch the DOM in place to match a new tree`,

  declarations: `<!-- component props: <prop> directive, or <component>{} object form -->
<prop name="tone" values="default|primary|accent" default="default" reflect />
<prop name="count" type="number" default="0" />

<!-- styling: sx token utilities on elements; <host sx> for the host;
     <variants> for state (keys off the host's attributes) -->
<host sx="radius:md bg:surface-1 border:border" />
<variants class="label">
  [tone=primary]: fg:primary
</variants>

<!-- live server data (compiles to the runtime data() SSE handle) -->
data live users = ship.users.list()`,

  slots: `<!-- component: declare named + default slots -->
<slot name="title"></slot>
<slot></slot>

<!-- consumer: fill by name; loose children -> default -->
<h2 slot="title">…</h2>`,
}
