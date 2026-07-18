---
group: Views
order: 12
title: Mutations
description: The declarative `<form mutation="…">` path, controller-side mutation via the typed client in a controller event handler, the deck mutation route, and error/pending UX.
---

# Mutations

A mutation is a write to the daemon. There are two ways to express one — a
declarative `<form mutation="…">` in the template, and an imperative
`ship.<domain>.<op>(…)` call from an event handler returned by the
controller. Both go through the same typed SDK client; neither writes its own
HTTP.

## Declarative — `<form mutation="ship.<domain>.<op>">`

Put `mutation="ship.<domain>.<op>"` on a `<form>` and the compiler wires submit
interception for you. The runtime resolves the SDK chain against scope, calls it
with the form's `FormData`, and drives a `loading → ok | error` lifecycle
(`packages/sail/runtime/src/mutation.ts:38`).

```html
<!-- apps/shell/src/settings/page.sail:375 -->
<form mutation="ship.settings.update" on:success={() => flashSaved(savedProvider)}>
  <select name="provider"> … </select>
  <button type="submit">Save</button>
</form>
```

On submit the runtime:

1. Calls `preventDefault()` and sets the form's `$mutation` signal to
   `{ kind: 'loading' }`.
2. Resolves `ship.settings.update` from scope and calls it with a `FormData`
   built from the form fields.
3. On success sets `$mutation` to `{ kind: 'ok', value }` and dispatches a
   `success` `CustomEvent`; on throw sets `{ kind: 'error', error }` and
   dispatches an `error` `CustomEvent`.

`on:success` and `on:error` handlers (above) listen for those events. The
injected `$mutation` signal is available in scope for inline pending/error UX:

```html
{#if $mutation().kind === 'loading'}<sail-spinner></sail-spinner>{/if}
{#if $mutation().kind === 'error'}<sail-error-state>{$mutation().error.message}</sail-error-state>{/if}
```

## Imperative — the typed client in a controller event handler

For anything past a single form post — building a payload, sequencing calls,
optimistic UI — call the SDK directly from an event handler returned by the
controller. Mutations are allowed **only** inside event handlers, `mount`, or
`onCleanup`, never in a `derived` body or template expression (controller
hard rule #3).

```ts
// apps/shell/src/studio/collections/page.sail — inside its <controller> block
export default controller(({ ship }) => {
  const newColOpen    = signal(false)
  const newColDraftId = signal('')

  const createCol = (a: { id: string; name?: string; fields?: Record<string, unknown> }) =>
    ship.collections.create(a)

  const submitNewCol = async () => {
    await createCol({ id: newColDraftId() })
    newColOpen.set(false)
  }

  return { newColOpen, newColDraftId, submitNewCol /* … */ }
})
```

The `ship.<domain>.<op>` methods that return `Promise` are mutations (e.g.
`ship.agents.create`, `ship.agents.update`, `ship.collections.create_item`,
`ship.pipelines.dispatch`); the ones returning `LiveSource` are subscriptions
and belong in `<data>` or a controller `query()`, which routes a `LiveSource`-returning method to its push path (`packages/sdk-client/src/ship-sdk.ts`).
`await` the promise to sequence follow-up state changes; wrap in `try/catch`
for error UX.

## The deck mutation route — 410 tombstone

Deck still composes a `<form>` `action` URL so a no-JS submit has somewhere to
go. The compiler emits a `mutation="bN"` marker but does **not** bake the URL;
the host injects `action="/_deck/mutation/<view>?form=bN" method="post"` at
SSR time, because only the host knows which daemon it runs
(`crates/deck/deck-core/src/form_inject.rs:1`).

That endpoint is a deliberate **410 Gone tombstone**. The progressive-
enhancement server handler was retired — JS-attached mutations call the SDK
directly — but deck keeps the route registered so the no-JS fallback gets a 410,
not a 404 (`crates/deck/deck-axum/src/lib.rs:122`):

```rust
.route(
    "/_deck/mutation/{name}",
    axum::routing::post(|| async { StatusCode::GONE }),
)
```

In practice this means mutations require the client runtime to be attached. The
SSR `action` is a marker for the seam, not a working submit target.

## Error and pending UX

Both paths surface the same `loading → ok | error` shape.

- **Declarative.** Read the injected `$mutation` signal (`{ kind }`), and/or
  attach `on:success` / `on:error` handlers for side effects (toast, close
  drawer). The settings views use `on:success={() => flashSaved(…)}`.
- **Imperative.** Hold a `signal` for the in-flight state, set it around the
  `await`, and branch the template with `{#if}` / `{#match}`:

```ts
const save = async () => {
  saving.set(true)
  try { await createCol({ id: draftId() }); error.set('') }
  catch (e) { error.set(String(e)) }
  finally { saving.set(false) }
}
```
