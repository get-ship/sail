---
group: Views
order: 4
title: Hooks — template and controller
description: Two categories of hook with different jobs and different syntax. Template hooks for host-side side effects; controller hooks for UI pattern bundles. When to reach for which.
---

# Hooks

SUI has **two** categories of hook with different jobs and different syntax. They are not interchangeable.

| Category | Job | Where it lives | hooks |
|---|---|---|---|
| **Template hook** | Element-scoped lifecycle / side effect, may contribute read-only scope identifiers | Template, as `<use-*>` element | `<use-kbd>`, `<use-click-outside>`, `<use-focus-trap>`, `<use-clipboard>`, `<use-title>`, `<use-scroll-anchor>`, `<use-intersection>` |
| **Controller hook** | Bundle of state + derived + handlers as a reusable UI pattern slice | Controller, called as a plain function inside the controller's setup function | `filteredList`, `selection`, `form` |

## When to reach for which

Three questions cover almost every "I need something" moment:

1. **Need data from the platform?** → `<data>` element in the template. Not a hook.
2. **Need a side effect tied to the view's lifecycle?** → Template hook (`<use-*>`).
3. **Need a reusable bundle of state, derived values, and handlers?** → Controller hook — call a composition function inside the controller's setup function.

If the answer doesn't fit one of the three, the case is either expressible as a *combination* of these primitives, or it's signaling a gap in the framework that should be filled deliberately — not accommodated with a fourth pattern.

## Template hooks

Element-shaped registrations for host-side side effects and platform context. Live in the template alongside the structure they augment.

### Authoring contract

A template hook is registered as a custom element of the form `<use-<name>>`. The runtime treats it specially:

- The element does not render to the DOM.
- Its **attributes** are bindings: `as="X"` declares an identifier in template scope; `bind:foo={ref}` passes a controller-side reference into the hook. **No expressions in attributes.**
- Its **children** are template fragments. They may contain expressions, control flow, slots, and nested elements. The hook decides what to do with them.

### Three patterns

**Pattern 1 — contribute a scope identifier:**

```html

<!-- shell is now readable in template and any controller section -->
<header class="theme-{shell.theme}">
  Welcome, {shell.currentUser.name}
</header>
```

**Pattern 2 — receive a controller reference:**

```html
<use-focus-trap bind:target={dialogEl} />
```

The `bind:` attribute passes an identifier from controller / template scope to the hook. The compiler statically requires that the value be a bare identifier (no expression).

**Pattern 3 — accept template-fragment configuration:**

```html
<use-title>{agents().value?.items.length ?? 0} agents — {shell.app.name}</use-title>

<use-kbd>
  <on key="/"        action="search" />
  <on key="cmd+n"    action="newAgent" />
  <on key="escape"   action="closeAll" />
</use-kbd>
```

Children are template fragments. Hooks that take richer configuration use child elements (here `<on key="..." action="..." />`) rather than attribute strings. Action references resolve against the controller's `on.*` section — same scope as `on:click={search}`.

### Authoring a template hook

Template hooks are TypeScript modules registered with the framework's hook registry. They expose:

- A name (the `<use-name>` tag)
- An attribute schema (which `as=`, `bind:` and literal attrs are accepted)
- A child schema (which child elements / template fragments are accepted)
- A factory that runs at mount and may register a `cleanup` for unmount

Template hooks v1 are stable — see [status.md](status.md) for the shipped registry.

### `<use-focus-trap>`

Traps keyboard focus within a target container element until the hook is unmounted. Intercepts Tab and Shift+Tab to cycle through focusable descendants, preventing focus from escaping the container (useful for modals, dialogs, and drawers). Focuses the first focusable descendant on mount and restores the previously focused element on unmount.

```html
<dialog bind:this={dialogEl}>
  <use-focus-trap bind:target={dialogEl} />
  <input placeholder="Name" />
  <button on:click={save}>Save</button>
  <button on:click={close}>Cancel</button>
</dialog>
```

The `bind:target` attribute receives an element reference from controller or template scope. If the target has no focusable descendants, Tab presses are suppressed.

### `<use-clipboard>`

Copies text to the clipboard when the host element is clicked. The `bind:text` binding accepts a plain string or a signal accessor — the value is read at click time, so dynamic text works naturally. The optional `bind:onCopy` callback fires after a successful clipboard write, making it easy to toggle a "Copied!" label.

```html
<button>
  <use-clipboard bind:text={agentUrl} bind:onCopy={markCopied} />
  {copied() ? 'Copied!' : 'Copy link'}
</button>
```

`<use-clipboard>` attaches a click listener to its host element (`ctx.el`). Place it inside or alongside the element that should trigger the copy action.

### `<use-title>`

Binds `document.title` to a signal. Reads the `bind:text` binding inside a reactive effect, so the title updates automatically whenever the underlying signal changes. Restores the previous `document.title` value on unmount.

```html
<use-title bind:text={pageTitle} />
```

`pageTitle` can be a plain string (set once) or a signal accessor (updated on every change). Typical usage combines it with a derived value:

```ts
// controller
export default controller((ctx) => {
  const pageTitle = derived(() => `${ctx.agents().length} agents — Ship`)
  return { pageTitle }
})
```

```html
<use-title bind:text={pageTitle} />
```

### `<use-scroll-anchor>`

Declares the host element as a named scroll target. On mount — and on every `hashchange` event — it checks whether `window.location.hash` equals `#<id>`. When it matches, it calls `el.scrollIntoView({ behavior: 'smooth' })`. This allows SPA navigation to `#section-name` URLs to land at the correct element without full-page reload.

```html
<section id="features">
  <use-scroll-anchor id="features" />
  <h2>Features</h2>
  ...
</section>
```

The `id` attribute is a static string (no signal binding). Assign it the same value as the parent element's `id` attribute so in-page links (`<a href="#features">`) work correctly.

### `<use-intersection>`

Fires a callback when the host element enters the visible viewport, powered by `IntersectionObserver`. Useful for lazy-loading content, triggering entry animations, or logging impressions. The observer is disconnected on unmount.

```html
<div>
  <use-intersection bind:on={reveal} />
  <img src={imgSrc} class={visible() ? 'visible' : 'hidden'} />
</div>
```

```ts
// controller
export default controller((ctx) => {
  const visible = signal(false)
  const reveal = () => visible.set(true)
  return { visible, reveal }
})
```

The `bind:on` callback receives the `IntersectionObserverEntry`. In environments without `IntersectionObserver` support (jsdom, very old browsers), the hook silently skips setup.

## Controller hooks

Bundles of state + derived values + handlers as a reusable UI pattern slice. A controller hook is nothing more than a plain function, called directly inside a controller's setup function — there's no separate registration mechanism and no dedicated composition section.

### Authoring contract

A controller hook is a function that returns a plain object of signals and helpers:

```ts
// packages/sail-std/src/filtered-list.ts
import { signal, derived, type Signal } from '@ship/sail-runtime'

export const filteredList = <T>(
  source: Signal<T[]>,
  predicate: (item: T, query: string) => boolean,
) => {
  const query = signal('')
  const filtered = derived(() => source().filter(i => predicate(i, query())))
  const setQuery = (q: string) => query.set(q)
  return { query, filtered, setQuery }
}
```

Nothing distinguishes this from any other helper function — reactive state is created with `signal(...)`, derived values with `derived(...)`, and handlers are just functions that call `.set(...)`. Use sites call it inline, inside the controller's setup function, and merge its return value into the controller's own returned object however they like:

```ts
// controller
import { filteredList } from '@ship/sail-std'

export default controller((ctx) => {
  const list = filteredList(ctx.agents, (a, q) => a.name.includes(q))

  return {
    list, // nested — reference as list.query / list.filtered in the template
  }
})
```

Or spread it flat if there's no risk of collision:

```ts
export default controller((ctx) => {
  const { query, filtered, setQuery } = filteredList(ctx.agents, (a, q) => a.name.includes(q))
  return { query, filtered, setQuery }
})
```

In the template, access the result however it was merged:

```html
<input bind:value={list.query} on:input={(e) => list.setQuery(e.target.value)} />
{#for a of list.filtered() key={a.id}}
  <sail-card>{a.name}</sail-card>
{/for}
```

### Passing signals to hooks

**Pass signals directly. Do not wrap them in closures.**

```ts
// good
filteredList(agents, (a, q) => a.name.includes(q))

// bad
filteredList(() => agents().value?.items ?? [], pred)
```

Signals are first-class reactive references. Inside the hook, `source()` reads reactively. Closures break that — the hook gets a function that *looks like* a regular getter and authors lose track of what's reactive. Pass the signal; let the hook unwrap.

For live signals with the standard envelope (`{ kind, value: { items } }`), hooks know the shape and unwrap to `.value.items` themselves. See [data-subscriptions.md](data-subscriptions.md#live-envelope).

### Composition rule

There is no automatic namespacing anymore — a controller hook's return value is just an object, merged into the controller's own returned object by ordinary JavaScript: spread it flat, destructure and rename individual members, or nest the whole thing under a key. The author decides.

If two hooks both return a `query` signal, spreading both flat is a real name collision — the second assignment silently wins, and the compiler/lint will flag the duplicate key. Nest one (or both) under a key to keep them apart:

```ts
export default controller((ctx) => {
  const list = filteredList(ctx.agents, (a, q) => a.name.includes(q))
  const search = filteredList(ctx.messages, (m, q) => m.text.includes(q))
  return { list, search } // list.query and search.query, no collision
})
```

This is a manual choice, not a framework guarantee — pick namespacing whenever a flat spread would collide.

## What's NOT a hook

Resist these. They look like hooks but aren't.

- **Subscribing to server data.** That's `<data>` in the template, not a hook. Even if you reuse a subscription across views, repeat the `<data>` element. The author should always see the subscription in the template.
- **Sharing global state.** That's a controller hook instantiated once and imported wherever needed. Same role as any other controller hook; no new primitive. (Yes, this might end up looking like a "store." It's still just a controller hook.)
- **Routing.** Routing is a `shipd` concern, exposed via `<data>` and template route helpers (TBD). Not a hook.
