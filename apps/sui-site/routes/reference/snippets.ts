// Reference snippets. ONE construct per snippet — a code block that contains six
// unrelated things, each "named" by a comment inside it, is not a reference. Every
// sample is compiled by the build (crates/sui/compiler/tests/site_snippets_compile.rs).
export const code = {
  // ── template directives ────────────────────────────────────────────────
  text: `<p>Signed in as {user().name}.</p>
<p>{count()} item{count() === 1 ? '' : 's'}</p>`,

  attr: `<img src={avatarUrl()} alt={name()} />
<button disabled={saving()}>Save</button>`,

  on: `<button on:click={save}>Save</button>
<form on:submit|prevent={save}>…</form>
<input on:input|debounce:200={search} />
<div on:keydown|key:Escape={close}>…</div>`,

  bindValue: `<input bind:value={draft} />
<textarea bind:value={body}></textarea>
<select bind:value={team}>…</select>`,

  bindChecked: `<input type="checkbox" bind:checked={agreed} />`,

  bindThis: `<canvas bind:this={canvasEl}></canvas>`,

  classDir: `<a class:active={path() === '/'}>Home</a>`,

  styleDir: `<column style:--gap={density()}>…</column>`,

  propDir: `<risk-meter prop:reading={sensor()}></risk-meter>`,

  useDir: `<textarea use:autosize></textarea>
<div use:clickOutside={close}>…</div>`,

  motionDir: `<card motion:hover={{ scale: 'spring:gentle' }}>…</card>
<card motion:enter={{ opacity: 'fade' }} motion:exit={{ opacity: 'fade-out' }}>…</card>`,

  // ── control flow ───────────────────────────────────────────────────────
  ifBlock: `{#if ok()}
  <p>Ready.</p>
{:else if pending()}
  <spinner></spinner>
{:else}
  <error-state></error-state>
{/if}`,

  forBlock: `{#for item of items() key={item.id}}
  <li>{item.label}</li>
{/for}`,

  matchBlock: `{#match status()}
  {:case 'draft'}   <badge tone="muted">Draft</badge>
  {:case 'live'}    <badge tone="success">Live</badge>
  {:default}        <badge>Unknown</badge>
{/match}`,

  asyncBlock: `{#async users as list}
  {:loading}
    <spinner></spinner>
  {:error err}
    <alert tone="destructive">{err.message}</alert>
  {:ok}
    <p>{list.length} users</p>
{/async}`,

  whileBlocks: `<!-- what {#async} desugars into. Reach for these when you want
     only one of the three branches. -->
{#while-loading of users}
  <spinner></spinner>
{/while-loading}

{#while-error err of users}
  <alert tone="destructive">{err.message}</alert>
{/while-error}`,

  // ── declarations ───────────────────────────────────────────────────────
  componentBlock: `<component>{
  props: {
    tone: 'default' | 'primary' | 'destructive',
    count: 0,
    label: '',
  },
}</component>`,

  controllerBlock: `<controller>
import { controller, signal, derived } from '@ship/sui-runtime'

export default controller({
  scope: () => {
    const count = signal(0)
    return { count, doubled: derived(() => count() * 2) }
  },
  mount(state, host) {
    // runs once, after the element is in the DOM
  },
})
</controller>`,

  slots: `<!-- the element -->
<card>
  <slot name="header"></slot>
  <slot></slot>
</card>

<!-- the caller -->
<card>
  <h2 slot="header">Title</h2>
  <p>Body.</p>
</card>`,

  // ── runtime ────────────────────────────────────────────────────────────
  signals: `import { signal, derived, effect, batch, untrack } from '@ship/sui-runtime'

const count = signal(0)
count()                    // read
count.set(5)               // write
count.set((n) => n + 1)    // functional write

const doubled = derived(() => count() * 2)   // lazy, cached
effect(() => console.log(count()))           // re-runs on change; return a cleanup`,

  liveData: `import { data } from '@ship/sui-runtime'

const users = data(() => ship.users.list())   // { kind: 'loading' | 'error' | 'ok' }`,

  compositions: `// A composition is a plain function over the primitives. No type, no
// registration, no rules — if it returns state, it is a composition.
import { selection } from '@ship/sui-runtime'

const { selected, isSelected, toggle } = selection(() => rows())`,

  context: `import { provide, useShared } from '@ship/sui-runtime'

provide('theme', theme)          // in the parent
const theme = useShared('theme') // anywhere below`,

  action: `import type { Action } from '@ship/sui-runtime'

export const autofocus: Action<void> = (node) => {
  const id = requestAnimationFrame(() => (node as HTMLElement).focus())
  return { destroy: () => cancelAnimationFrame(id) }
}`,
}
