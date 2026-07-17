---
group: Views
order: 5
title: Data subscriptions
description: `<data>` is the canonical bridge to the daemon. Live envelope shape, signal conventions, why hooks don't subscribe.
---

# Data subscriptions

The Ship daemon (`shipd`) is the model. Views bridge to it via `<data>` elements declared in the template. This is the **only** sanctioned way to bring server data into a view. It is not interchangeable with hooks.

> The catalog is also served live at `GET /api/ship/schema` and checked into `packages/sdk/src/generated/sdk-ship-ops.json` for offline browsing.

## The element

```html
<data name="agents" source="ship.agents.summaries()" live />
<data name="settings" source="ship.settings.state()" />
<data name="agent" source="ship.agents.detail({id: route.params.id()})" live />
```

Attributes:

| Attr | Required | Purpose |
|---|---|---|
| `name="..."` | yes | Identifier exposed in view scope. |
| `source="..."` | yes | A `ship.*` SDK call expression. Evaluated with `ship` and `route` in scope. |
| `live` | no | If present, the runtime opens an SSE channel and patches the signal as the daemon emits updates. |

The runtime exposes `<name>` as a signal in [view scope](index.md#the-shared-view-scope). Read it via `name()` in expressions and controller sections.

## Live envelope

`live` signals always wrap the daemon's payload in a uniform envelope. The shape is contractual; hooks rely on it.

```ts
// from @ship/sui-runtime — `live.ts`
type LiveState<T> =
  | { kind: 'loading' }
  | { kind: 'error'; error: Error }
  | { kind: 'ok'; value: T }

type LiveSignal<T> = Signal<LiveState<T>>
```

Three variants. `loading` carries no payload; `error` carries an `Error`; `ok` carries the resolved value. The `value` field lives **only** on the `ok` variant — discriminate by `kind` before reading it.

Read it directly in templates:

```html
{#if agents().kind === 'ok'}
  <p>{agents().value.items.length} agents</p>
{:else if agents().kind === 'error'}
  <p>Failed: {agents().error.message}</p>
{:else}
  <p>loading…</p>      {/* kind === 'loading' */}
{/if}
```

Or use the convenience boundary blocks:

```html
{#while-loading of agents}
  <p>loading…</p>
{/while-loading}

{#while-error err of agents}
  <p>Failed: {err.message}</p>
{/while-error}

{#if agents().kind === 'ok'}
  <!-- happy path -->
{/if}
```

`{#while-loading}` and `{#while-error err}` are syntactic sugar over the kind check. They render their children only while the bound signal is in the matching state and clean up automatically. The error variant binds the error value to the local name preceding `of`.

## Static `<data>` (no `live`)

Without `live`, the SDK call runs once at mount; the result is exposed as a plain signal containing the resolved value (no envelope). Used for one-shot reads where streaming updates aren't relevant — config, lookups.

```html
<data name="settings" source="ship.settings.state()" />

<p>Theme: {settings().theme}</p>
```

If the call is async, the runtime exposes the signal once it resolves; pre-resolution reads return `undefined`. For a loading state, add `live` to the `<data>` element and use `{#while-loading}` or discriminate by `kind`.

## Why subscriptions don't go in hooks

Subscriptions are declarations, not pattern slices. The author should always see, in the template, every server channel a view depends on. Hiding a subscription inside `useAgents()` (or any hook) loses that visibility — readers can't tell what data the view needs without opening the hook.

Concretely:

```html
<!-- right: every subscription visible at the top of the file -->
<data name="agents"   source="ship.agents.summaries()" live />
<data name="settings" source="ship.settings.state()" />

<!-- wrong: subscriptions hidden behind a hook -->
<!-- (don't author this) -->
```

```ts
// wrong: subscriptions in a hook
export const useAgents = () => ({
  live: { agents: ship.agents.summaries() },
})
```

Repetition across views is the right trade. If three views all subscribe to `ship.agents.summaries()`, all three have the `<data>` element. The pattern *of subscribing to agents* is not abstracted; the visibility is preserved.

The hook layer is for **bundling state + derived + handlers**. A hook that *uses* an `agents` signal (filtering, selecting, paginating, sorting) is doing real work and earns its existence. A hook that just wraps a subscription is double-dipping.

## Passing data signals to compositions

When a composition needs a data signal, **pass the signal itself**, not a closure.

```ts
// in a controller
export default controller((ctx) => {
  const list = filteredList(ctx.agents, (a, q) => a.name.includes(q))
  const sel = selection(ctx.agents, a => a.id)

  return { list, sel }
})
```

The composition destructures the envelope internally:

```ts
// inside filteredList
const items = () => {
  const s = source()                              // source: LiveSignal<{ items: T[] }>
  return s.kind === 'ok' ? s.value.items : []
}
```

This is why the envelope is contractual — compositions rely on the shape to unwrap consistently.

## Reactive `route` in source thunks

`route.params.<name>` and `route.query.<name>` are themselves signals. Reading one inside a `<data live source="…">` thunk subscribes the live channel to the param: when navigation changes the URL, the prior SSE subscription disposes and a new one opens with the updated arg. No author-side wiring; this is the same `live()` tracking that re-fires on any controller-owned signal change.

Concrete pattern — a `/agents/<id>` view:

```ts
<controller>
// page.sui — inline <controller> block
import { derived } from '@ship/sui-runtime'
import { defineRoutes } from '@ship/sui-runtime/route'

defineRoutes([{ pattern: '/agents/:id' }])

export default controller(({ agent }) => {
  const name = derived(() => agent().kind === 'ok' ? agent().value.name : '')
  return { name }
})
</controller>
```

```html
<!-- page.sui -->
<data name="agent" source="ship.agents.detail({id: route.params.id()})" live />

<h1>{name()}</h1>
{#if agent().kind === 'loading'}<p>Loading…</p>{/if}
```

Navigation flow:

1. The user lands on `/agents/42`. `route()` (called by the boot script) seeds `route.params.id` from the pathname-to-pattern match: `'42'`.
2. `<data live>` evaluates `ship.agents.detail({id: route.params.id()})`. `route.params.id()` is a signal read; `live()` records it as a dep. The SSE channel for agent 42 opens.
3. The user clicks a link to `/agents/99`. The view's click handler (or the framework's link interceptor) calls `navigate('/agents/99')`. That pushes history and triggers the global `__shipSetRouteUpdate`, which sets `route.params.id` to `'99'`.
4. The `live()` dep fires. The agent-42 SSE channel disposes; a new one opens for agent 99. The signal flips through `loading → ok` exactly as on first mount.

`defineRoutes` is per-view. Without a registered shape that captures `id`, `route.params.id()` returns `undefined`; the source thunk still tracks the signal, so the channel re-binds the moment a future navigation introduces a matching segment. (A compiler-emitted `<route pattern="…" />` directive will make the imperative `defineRoutes` call optional in a follow-up.)

## SDK access

The `ship.*` SDK is the typed bridge to `shipd`. It is *not* a global; it's one key on the host-supplied clients map. A view declares the clients it consumes with a top-level `<client name="ship" />` directive (see [template.md](template.md#client--host-injected-data-client)); the host passes the map at mount (`mount(target, payload, { clients: { ship } })`) and the runtime injects each declared name into:

- The `source=` attribute of `<data>` elements (where the declared clients and `route` are in scope)
- The function form of a controller (`({ ship, clients }) => ...`)
- Each controller section's destructure (`{ ship }` is part of the scope object; so is every other declared client)
- Each controller hook's factory (when it accepts scope)

Authors do not import `ship`. The host owns the value, the runtime owns the wiring, and the SDK provider owns the type (`declare module '@ship/sui-runtime' { interface Clients { ship: ShipSDK } }`). `ship` is just the key Ship hosts use — a non-Ship host injects its own clients under its own names. Templates that declare no clients still receive the ambient `ship` as a grandfathered compatibility default.

## Mutations

`<data>` is the read surface. The symmetric write surface is `<form mutation="ship.X.Y">`.

```html
<form mutation="ship.agents.create">
  <input name="name" placeholder="Agent name" />
  <button type="submit">Create</button>
</form>
```

The `mutation` attribute is a `ship.*` SDK chain. On submit the compiler-wired `wireFormMutation` helper:

1. Prevents the default form submission.
2. Sets `$mutation` to `{ kind: 'loading' }`.
3. Serializes the form as `FormData` and calls the SDK chain.
4. On success: sets `$mutation` to `{ kind: 'ok', value: result }` and dispatches a `success` CustomEvent.
5. On error: sets `$mutation` to `{ kind: 'error', error }` and dispatches an `error` CustomEvent.

`$mutation` is a `Signal<LiveState<unknown>>` injected into view scope by the compiler. Use it the same way you'd use a `<data live>` signal:

```html
<form mutation="ship.agents.create">
  <input name="name" />
  <button type="submit" disabled={$mutation().kind === 'loading'}>
    {$mutation().kind === 'loading' ? 'Creating…' : 'Create'}
  </button>
</form>
{#if $mutation().kind === 'error'}
  <p class="error">{$mutation().error.message}</p>
{/if}
```

**Progressive enhancement path.** When JS is absent, the SSR walker injects `action="/_deck/mutation/{view}?form=bN"` and `method="post"` onto the `<form>` element. The daemon's `POST /_deck/mutation/{view}` endpoint resolves the SDK chain from the binding manifest, dispatches the call, and responds with an HTTP 303 redirect to `Referer` (success) or `Referer?_sui_error=<msg>` (error). Only SDK operations registered in the service registry are reachable via PE.

**One `$mutation` per form.** Each `<form mutation="…">` in a template gets its own `$mutation` signal. If a template has two forms, distinguish them via `on:success` / `on:error` handlers or by scoping the error display to the form's immediate context.
