---
group: Views
order: 2
title: The template surface
description: Everything you can write in a `.sui` file — directives, control flow, bindings, hooks, projection, expressions.
---

# The template surface

A `.sui` file is HTML with a small set of directive elements at the top, mustache control flow inside, and a few binding shorthands on attributes. There is no script block and no style block. Logic lives in an inline `<controller>` block in the same file; styles live in a sibling `style.css` linked by `<link rel="stylesheet">`.

## Directive elements

These appear at the top of the file (before the rendered DOM). They declare reactivity but do not render to HTML.

### Reactive state — declared in the inline `<controller>` block

Reactive component state lives in the controller — there is no `<state>` directive. Templates declare data subscriptions, projection regions, and props; the controller owns the signals templates read.

```ts
<controller>
import { signal } from '@ship/sui-runtime'

export default controller(() => {
  const count = signal(0)
  const open  = signal(false)

  const increment = () => count.set(count() + 1)

  return { count, open, increment }
})
</controller>
```

The template reads them by call (`count()`) and mutates them through handlers returned from the controller. See [controller.md](controller.md) for the full controller surface.

Computed-initial values (`crypto.randomUUID()`, an SDK call, a derivation over `route`) seed a placeholder signal and assign it in `mount`:

```ts
const id = signal('')
mount(() => {
  id.set(crypto.randomUUID())
})
```

### `<data>` — daemon subscription

```html
<data name="agents" source="ship.agents.summaries()" live />
<data name="settings" source="ship.settings.state()" />

<!-- Reactive args via $scope: re-subscribes when selectedId changes.
     `selectedId` is a signal returned by the controller function;
     the template just declares the subscription. -->
<data name="agent" source="ship.agents.detail({id: $scope.selectedId()})" live />
```

Subscribes to a `ship.*` SDK call. With `live`, the runtime opens an SSE channel and patches the signal as the daemon emits updates. Without `live`, the value is read once at mount.

The `source` expression is plain JS that runs inside `live()`'s tracked reactive scope. It receives three arguments: `ship` (the SDK), `route` (a [`RouteSignals`](data-subscriptions.md#reactive-route-in-source-thunks) — `route.params.<name>` and `route.query.<name>` are themselves signals), and `$scope` (every identifier already in view scope — controller-scope signals, earlier `<data>` declarations). Reading a signal via `$scope.<key>()` or `route.params.<key>()` is tracked; when the signal changes, the prior subscription disposes and a new one opens with the updated args.

`<data>` is the **canonical** way to bring server data into a view. See [data-subscriptions.md](data-subscriptions.md).

### `<client>` — host-injected data client

```html
<client name="ship" />
<data name="overview" source="ship.git.overview()" live />

<!-- Any key works — the name is the view's alias into the host's
     clients map, not a fixed vocabulary. A non-Ship host injects
     whatever it has: -->
<client name="api" />
<data name="t" source="api.demo.clock()" live />
```

Declares a named data/ops client the view consumes. The host supplies a keyed map at mount (`mount(target, payload, { clients: { ship } })`) and the framework wires only what's declared: each declared name is destructured from `ctx.clients`, seeded into view scope, and legal as a `<data source>` / handler root. An undeclared root is a compile error (typo-safe); duplicate declarations, non-identifier names, and reserved context names (`route`, `navigate`, `ctx`, `scope`, `props`, `derived`) are rejected.

The client's TYPE comes from declaration-merging the runtime's `Clients` interface — the provider augments its own key, the framework names no host shape:

```ts
// @ship/sdk-client does this for `ship`:
declare module '@ship/sui-runtime' {
  interface Clients { ship: ShipSDK }
}
```

With the declaration in place, a `<data source>` that is a single call on a declared client types its signal from the client's method return (`Clients['ship']['git']['overview']`) in the generated `.sui.d.ts`.

**Compatibility default:** a template that declares NO clients still receives the ambient `ship` (the grandfathered pre-directive behavior). New and migrated templates declare it explicitly — the `sui-client-directive` lint (Warn) flags `ship.*` usage without the declaration.

### `<prop>` — component prop declaration

```html
<prop name="tone" values="default|primary" default="default" reflect />
<prop name="disabled" type="bool" default="false" reflect />
<prop name="label" type="string" default="" />

<button disabled={disabled() || loading()}>{label()}</button>
```

The file IS the component. `<prop>` appears at the top of a component `.sui` file alongside `<data>` — no wrapper element needed. Convention is the contract: directory name is the tag name. Views (top-level page surfaces) do not use `<prop>`.

Prop kinds: `string` (default), `bool`, `number`, `values="a|b|c"` (union). `reflect` mirrors the prop to the host element's attributes.

> **Removed in the light-DOM default.** The `component_meta.shadow`
> compile-opt is gone. Shadow is a per-component containment feature
> declared in the anatomy manifest, paid for only by components that
> genuinely wrap foreign content (the tiptap wrapper, embedded
> third-party widgets, the FFI boundary). The compiler does not
> default-shadow anything, ever.
>
> **Form participation — mostly free in light DOM.** The
> `form-value` and `form-disabled` prop flags still parse and still
> wire `ElementInternals.setFormValue` / `formDisabledCallback`
> (`crates/sui/compiler/src/ir/directive.rs`,
> `packages/sui/runtime/src/component-factory.ts`), so a component
> that needs explicit form association can still declare it. The
> light-DOM default means most components don't: a `<sui-input>`
> descendant of a host `<form>` participates natively without any
> prop flag, because it is a real child of the form in the same
> DOM tree. ARCHITECTURE.md calls this "the ElementInternals
> plumbing mostly dies" — not "is removed". `[ROADMAP:
> form-participation-surface]` — which stdlib components still
> need the flag (for non-native value serialization, custom
> reset/restore behaviour) is decided per component at re-port.

### `<form mutation>` — write operation

```html
<form mutation="ship.agents.create">
  <input name="name" placeholder="Agent name" />
  <button type="submit" disabled={$mutation().kind === 'loading'}>
    {$mutation().kind === 'loading' ? 'Creating…' : 'Create'}
  </button>
</form>
{#if $mutation().kind === 'error'}
  <p>{$mutation().error.message}</p>
{/if}
```

The `mutation` attribute is a `ship.*` SDK chain. `<form mutation>` is the write-side complement to `<data>`. The compiler wires it at compile time: on submit, the form is serialized as `FormData` and the SDK chain is called. `$mutation` is injected into view scope as a `Signal<LiveState<unknown>>` — the same `{ kind: 'loading' | 'ok' | 'error' }` envelope as `<data live>`.

Progressive enhancement: when JS is absent the SSR walker injects `action="/_deck/mutation/{view}?form=bN"` and `method="post"` on the form. The daemon handles the POST and redirects back to `Referer`. See [data-subscriptions.md](data-subscriptions.md#mutations) for the full envelope shape and PE flow.

**Threshold rule.** A template with no `<controller>` block (tier-1) may not have multi-statement `on:event` handler bodies. The compiler enforces this at build time (`crates/sui/compiler/src/check/handler_complexity.rs`):

```html
<!-- fails without controller.ts -->
<button on:click={() => { x.set(1); y.set(2) }}>Save</button>
```

```
page.sui: line 1 col 30: [on:click handler]
multi-statement handler — paired `<view>.ts` controller required for tier-2 work
```

Single-expression handlers and single-statement blocks are fine at tier 1. Add a paired `controller.ts` to lift the restriction. See [controller.md](controller.md#tier-1--tier-2-threshold-rule).

## Control flow

Mustache-style, directly in the template body:

### `{#if}` / `{:else if}` / `{:else}` / `{/if}`

```html
{#if loading()}
  <sui-spinner />
{:else if error()}
  <p>Failed: {error().message}</p>
{:else}
  <p>{count()} items</p>
{/if}
```

### `{#for}` / `{/for}` (with optional `key`)

```html
{#for a of agents().value.items key={a.id}}
  <sui-card>{a.name}</sui-card>
{/for}
```

`key={...}` provides stable identity across re-orders and updates. Without `key`, items are re-created on any list change.

### `{#match}` / `{:case ...}` / `{:default}` / `{/match}`

```html
{#match status()}
  {:case 'idle'}    <p>Ready</p>
  {:case 'loading'} <sui-spinner />
  {:case 'error'}   <p>Failed</p>
  {:default}        <p>Unknown</p>
{/match}
```

### `{#while-loading}` / `{/while-loading}` and `{#while-error err}` / `{/while-error}`

Convenience boundaries for `<data live>` envelopes. Renders the body only when the bound signal is in the matching `LiveState` kind, swaps automatically as the envelope flips. The error variant binds the error value to a local name preceding `of` (mirrors `{#for x of xs}`).

```html
<data name="agents" source="ship.agents.summaries()" live />

{#while-loading of agents}
  <p>Loading…</p>
{/while-loading}

{#while-error err of agents}
  <p>Failed: {err.message}</p>
{/while-error}

{#if agents().kind === 'ok'}
  {#for a of agents().value.items key={a.id}}…{/for}
{/if}
```

The element forms (`<while-loading bind="X">`, `<while-error bind="X" let:err>`) are gone — block syntax only.

## Expressions

Any `{ ... }` opens an expression. Resolved against the [shared view scope](index.md#the-shared-view-scope).

```html
<p>Hello, {user().name}.</p>
<p>{count() === 1 ? '1 item' : count() + ' items'}</p>
<button data-disabled={disabled() || loading()}>...</button>
```

Signals are read by call: `count()`. The framework does not auto-call. Read explicitly, write via `count.set(v)`.

### The renderable subset (for SSR)

Server-side rendering evaluates expressions against initial signal values. Stay in the renderable subset to get free SSR; step outside it and the expression becomes client-only with a placeholder on the server.

**Renderable**: signal calls (`count()`), property access (`user().name`), arithmetic (`count() + 1`), comparison (`x() > 5`), ternaries (`a ? b : c`), template literals with no embedded function calls.

**Not renderable** (client-only): function invocations beyond `signal()` reads, closures, `new ...`, async, `this`, arbitrary JS.

The compiler flags non-renderable expressions with a warning. See [ssr-hydration.md](ssr-hydration.md).

## Bindings

### `on:` — event handlers

```html
<button on:click={() => count.set(count() + 1)}>+</button>
<input on:input={(e) => setQuery(e.target.value)} />
<form on:submit={submit}>...</form>
```

The handler value is an arrow function whose body should be a single call expression. The runtime invokes it with the DOM event. Modifiers will be added later (`on:click:once`, `on:keydown:escape`); current source supports the bare form.

### `bind:` — two-way binding

```html
<input bind:value={query} />
<input type="checkbox" bind:checked={enabled} />
```

The runtime wires both directions: signal → DOM and DOM → signal. Do not also write `on:input={...}` — `bind:` owns the listener. If you need a side effect on change, write a `derived` over the bound signal or an `effect`.

### Attribute interpolation

```html
<div class="card card-{tone()}" data-state={state()}>...</div>
<button disabled={isDisabled()} aria-pressed={pressed()}>...</button>
```

Empty / `null` / `undefined` / `false` auto-omit the attribute. `true` is presence-only.

## Template hooks (`<use-*>`)

Element-shaped registrations for host-side side effects and platform context. Live in the template alongside the structure they augment. See [hooks.md](hooks.md) for the full contract.

```html
<use-kbd>
  <on key="/" action="search" />
  <on key="cmd+n" action="newAgent" />
</use-kbd>
<use-title>{agents().value?.items.length ?? 0} agents</use-title>
```

Authoring rule: hook **attributes** are bindings (`as="X"`, `bind:foo={ref}`), never expressions. Hook **children** are template fragments and may contain expressions, control flow, and projected children.

## Projection

Components invite the call site to fill in their chrome. The model
is **compiler-managed projection**: a component declares the regions
it accepts in its anatomy manifest (and a `<slot>` element in the
body for each region), and a consumer fills those regions by writing
`slot="<region>"` on the projected child. The children live in the
host's light-DOM subtree from first paint; the runtime projects them
into the matching `<slot>` wrapper at connect time. No shadow root,
no native slot redistribution — the runtime reimplements the
`slot=` / `<slot>` semantics in light DOM, since native `<slot>` only
works inside a shadow root.

**Author surface** (compile-plausible today):

- The consumer's projected child carries a `slot="<name>"` attribute
  (the same name as the corresponding `<slot name="<name>">` in the
  component's body). An un-named child goes to the default `<slot>`.
- The component body has `<slot name="…">` elements at the position
  the projected children should appear. The runtime swaps the slot's
  fallback content for the assigned children at connect.
- The component's anatomy manifest lists the regions it accepts —
  the *published contract*. Consumers read it the same way they read
  `props` and `events` (LSP, `@ship/sui-eval`).

**Anatomy manifest shape** (shipped): the compiler emits an `anatomy`
block on each `custom-elements.json` declaration —
`{ "dom": "light" | "shadow", "classes": [...], "slots": [...] }` —
classes from the template (generated `sx-*`/`sui-*` atomics excluded),
slots as the projection points (`(default)` for the unnamed slot).

**`[ROADMAP: scoped-projection-syntax]`** — shadow slots
support `<slot name="x" item={x}>` and consumer `let:item`. The
light-DOM projection is one-shot at connect; the compiler-managed
equivalent for scoped binding is its own design work.

```sui
<!-- in sui-card.sui — body declares the chrome; regions are in the manifest -->
<div class="card">
  <header class="card-head">
    <slot name="header"></slot>
  </header>
  <div class="card-body">
    <slot></slot>
  </div>
  <footer class="card-foot">
    <slot name="footer"></slot>
  </footer>
</div>
```

```sui
<!-- in a consumer -->
<sui-card>
  <sui-chip slot="header" tone="primary">draft</sui-chip>
  <p>This is the body — it lands in the default region.</p>
  <sui-button slot="footer" on:click={save}>Save</sui-button>
</sui-card>
```

What the contract buys:

- **Reactive in the projected scope.** A signal read inside a
  projected child re-renders the same way it would outside —
  projection does not break reactivity.
- **Native form participation.** Projected inputs are real
  descendants of the host; `<form>` traversal sees them.
- **Cross-root ARIA works.** `label[for]` and `aria-describedby`
  resolve against the host's children without any retargeting.
- **Honest devtools / tests.** The DOM is what the user sees.
- **Honest SSR.** The server-rendered HTML is the post-projection
  tree; no declarative-shadow dance, no hydration flash.

**Empty-region reflection** (shipped): the runtime helper
`bindSlotEmpty(slotEl, host, attrName)` (`packages/sui/runtime/src/slot-empty.ts`)
toggles a host attribute based on whether the slot's content is
visible — `slotchange`-driven in shadow, `MutationObserver`-driven in
light. The compiler emits one call per `<slot reflect-empty="<attr>">`
directive; the attribute name is the author's, per slot.

**Projection runtime API** (shipped, internal): `projectSlots(host,
captured)` (`packages/sui/runtime/src/slot-project.ts`) is called by
the component factory after mount — it is not authoring surface, and
no public projection API exists by design: authors write `<slot>` and
`slot="…"`, nothing else.

**Known limit (documented, not hidden):** projection runs once at
connect. Children appended to the host LATER are not re-projected —
the dynamic composition path in SUI templates goes through compiled
call sites, not raw DOM appends, so this covers the authored
surface. Streaming-parse upgrades are the other gap; bundles load
deferred today so the DOM is complete at registration.

## What's NOT in the template

For clarity — these go in the controller, never the template:

- Imperative logic (`async`, `await`, `try/catch`) — use a controller's `mount` and a state signal that updates when the work resolves
- Coordinated state (multiple signals that interact)
- Derived values used in more than one place
- Effects that do something other than render
- Anything that calls `ship.*` outside a `<data>` declaration

If you find yourself wanting a multi-line script block in the template, you want a controller. Add a paired `.ts`.
