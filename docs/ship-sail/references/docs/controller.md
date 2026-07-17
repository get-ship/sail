---
group: Views
order: 3
title: The controller surface
description: The optional inline `<controller>` block. One function, ambient lifecycle, direct signal mutation.
---

# The controller surface

A controller is an **optional** `<controller>` block inline in the `.sui` file — never a separate file. There is no config, no import statement to wire it up, no directory-pairing convention: the block lives where the template lives.

The controller is the view's view-model: reactive state, derived values, event handlers, and lifecycle. Templates declare structure; the controller owns everything reactive.

## No controller at all

Pure declarative views need nothing. The template uses `<data>` for subscriptions and inline expressions for anything computed. **No reactive state, no `<controller>` block.**

```html
<!-- views/static/page.sui -->
<data name="overview" source="ship.git.overview()" live />

<p>Branch: {overview().value?.branch}</p>
```

Most one-shot read-only views land here.

## One function, one idiom

The moment a view needs local state, a derived value, or an event handler, add a `<controller>` block. It contains exactly one call: `controller(fn)`.

```html
<!-- views/agents/page.sui -->
<controller>
export default controller((ctx) => {
  const items = signal<Item[]>([])
  const query = signal('')

  const filtered = derived(() =>
    items().filter((i) => i.name.includes(query()))
  )

  mount((el) => {
    const id = setInterval(() => ctx.ship.agents.poll(), 5000)
    return () => clearInterval(id)
  })

  onCleanup(() => console.log('agents view disposed'))

  return {
    items,
    query,
    filtered,
    setQuery: (q: string) => query.set(q),
    refresh: () => ctx.ship.agents.refresh(),
  }
})
</controller>
```

`fn` is `(ctx, host?) => Record<string, unknown>`:

- **`ctx`** is the ambient view scope — `ship`, `clients`, `route`, `navigate`, plus every template-declared `<data>`/`<state>` signal, all directly accessible (`ctx.ship`, or destructure `{ ship, count }` in the parameter).
- **`host`** (optional second parameter) is the custom-element host, for reaching the shadow root or doing direct DOM work.
- Whatever `fn` **returns** is the entire contributed template scope: state signals (`signal(...)`), computed values (`derived(...)`), plain helper functions, and event handlers — all as ordinary object properties. There is no separate section for handlers; a handler is just a function returned alongside everything else.

No sections. No `setup:`/`derived:`/`on:`/`mount:`/`unmount:` object keys, no `use:` controller-hook mini-DSL, no paired `<stem>.controller.ts` sibling file — a sibling `.controller.ts` next to a `.sui` file is a compile error. `controller()` takes the function directly.

## Mutation is always `.set(...)`

State is a plain `signal()` closed over by the setup function. There is no writable-state proxy, no `state.count = 5` or `state.count++` assignment style — that idiom belonged to the retired `on:`/`mount:`/`unmount:` sections and no longer exists. Every mutation, from a handler or anywhere else, is a direct call on the signal:

```ts
count.set(count() + 1)          // replace
count.set((n) => n + 1)         // updater form
items.set([...items(), next])   // replace with a derived-from-previous value
```

The lint rule `controller_state_set_call` (sail-lint) enforces this everywhere a controller mutates state.

## Ambient lifecycle: `mount` and `onCleanup`

Lifecycle is not a section — it's two ambient functions a controller calls from anywhere in its setup call graph: the setup body itself, or a plain helper/composition function the setup body calls. Both are auto-injected by the compiler the same way `signal`/`derived`/`controller` are; an author never writes an import for them, just calls the bare identifier.

- **`mount(fn)`** — `fn: (el: HTMLElement) => (() => void) | void`. Fires once after this component/controller instance's DOM connects. If `fn` returns a function, that function is a cleanup that runs when the instance disposes. Calling `mount()` outside any setup window throws `SuiMountOutsideSetupError`.

  ```ts
  mount((el) => {
    const observer = new ResizeObserver(() => recompute())
    observer.observe(el)
    return () => observer.disconnect()
  })
  ```

- **`onCleanup(fn)`** — `fn: () => void`, runs on dispose. Unlike `mount`, it no-ops silently (doesn't throw) if called outside a scope.

  ```ts
  onCleanup(() => subscription.close())
  ```

Neither is lexically restricted to a top-level statement in the setup body — call them from inside a plain helper function that setup invokes, and they still register against the running instance's scope. They cannot be called outside a setup/mount execution window (a later timer callback, a detached handler invoked after the fact, etc.) — `mount()` throws in that case, `onCleanup()` silently does nothing.

Computed-initial cases (`crypto.randomUUID()`, an SDK call, a derivation over `route`) work the same way any other imperative setup does — call it directly in the setup body, or defer to `mount` if it needs the connected DOM:

```ts
export default controller((ctx) => {
  const id = signal(crypto.randomUUID())
  return { id }
})
```

## Compositions are still plain functions

`@ship/sail-std` compositions are ordinary TypeScript functions — call one inside the setup function and whatever `Signal<T>`s (or other values) it returns become ordinary scope members, exactly like state declared directly in the controller:

```ts
import { controller, filteredList, selection } from '@ship/sail-runtime'

export default controller((ctx) => {
  const list = filteredList(ctx.agents, (a, q) => a.name.includes(q))
  const sel = selection(list.filtered)

  return { ...list, ...sel }
})
```

There is no hook registry, no namespacing convention, no `ControllerHook` type — a composition is just a function you call, and you decide how (or whether) to merge, rename, or namespace what it returns before returning it from the controller. The only language type a composition touches is `Signal<T>`.

## What's NOT in the controller

For clarity — these go in the template, never the controller:

- Server data subscriptions (use `<data>` in the template).
- Host-side side effects like keybindings, document title, focus traps (use `<use-*>` template hooks).
- DOM structure (templates own structure, full stop).

If you find yourself wanting to construct DOM inside `mount`, you almost certainly want a template hook or a child component instead.

`use:` in a template (`use:actionName`) is unrelated to controller authoring — it's the directive that attaches an Action to a DOM node, not a controller concept.

## File layout

The controller lives inline. Most common shape:

```
views/<name>/
  page.sui              # template + inline <controller> block (controller optional)
  style.css             # styles (optional)
  manifest.jsonc         # route descriptor (required for views)
```

Components follow the same rule — the file IS the component, and any controller for it is an inline `<controller>` block inside that same `.sui` file:

```
packages/components/ui-components/src/sail-button/
  sail-button.sui            # template + <prop> directives + inline <controller> (only if needed)
  sail-button.css            # extracted styles (optional)
```

A `<tag>.controller.ts` (or `<view>.controller.ts`) sibling file is **not** a valid way to author a controller — the compiler rejects it as a compile error. If you see one in older code or docs, it's the retired paired-file convention; port it into an inline `<controller>` block.

## Handler complexity has no threshold rule

Because there's only one authoring surface — the inline `<controller>` block — there's no tier boundary and no compiler-enforced "multi-statement handler needs a paired file" rule. Any handler, of any complexity, is just a function returned from `controller(fn)`:

```html
<button on:click={() => { count.set(count() + 1); logClick() }}>+</button>
```

```html
<controller>
export default controller((ctx) => {
  const count = signal(0)
  const logClick = () => ctx.ship.analytics.track('click')
  return { count, logClick }
})
</controller>
```

Keep template-inline handlers to trivial single expressions by convention; anything with real logic belongs as a named function returned from the controller and referenced by name (`on:click={increment}`), not because the compiler forces it, but because it reads better.
