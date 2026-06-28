---
title: "<data live> — server-pushed signals"
description: "A directive that subscribes to an SSE stream and unwraps the envelope."
group: "Tutorial"
order: 22
slug: "22-data-live"
prev: "21-forms"
next: "23-while-loading"
chapter: "views"
---

`<data live source="…" name="…">` declares a **live signal** that
subscribes to a server-pushed stream. The runtime opens an
EventSource against the source, parses each message as a `LiveState`
envelope, and exposes the result as a normal signal in template
scope.

```sui
<data name="users" source="ship.users.list()" live />

{#if users().kind === 'ok'}
  <ul>
    {#for u of users().value.items key={u.id}}<li>{u.name}</li>{/for}
  </ul>
{:else if users().kind === 'loading'}
  <p>loading…</p>
{:else}
  <p class="err">{users().error.message}</p>
{/if}
```

The envelope:

```ts
type LiveState<T> =
  | { kind: 'loading' }
  | { kind: 'ok'; value: T }
  | { kind: 'error'; error: { message: string } }
```

Two things to know:

- **Lifecycle scoped to mount.** The EventSource opens at mount, the
  cleanup closes it on unmount. No leaks across navigations.
- **Initial state is `loading`.** The signal emits `{ kind: 'loading' }`
  immediately, so any `{#if users().kind === 'loading'}` branch
  renders on first paint without flicker. Subsequent envelopes from
  the stream advance the signal.

## How the playground simulates the server

This tutorial doesn't run a real Ship daemon. A **service worker**
inside the iframe (`/tutorial/sw.js`) intercepts SSE requests under
`/_deck/data/<domain>/<op>` and plays back a per-step scenario that
the parent loads before the view mounts. The runtime sees a
"real" SSE stream; the playground decides the timing.

For this step the scenario fires:
- `loading` immediately
- `ok` with two users after 600ms
- `ok` with three users after 2.4s
- `error` after 4.8s

Watch the iframe — the list grows, then the error branch takes over.

## Try it

1. **Reorder the branches**: put `{:else if users().kind === 'loading'}`
   first. Verify the loading state still shows up before the data
   arrives.
2. **Show the count**: add a `<p>{users().value.items.length} users</p>`
   inside the `ok` branch. It re-renders on each new envelope.
3. **Recover from error**: add a "retry" button that re-mounts the
   `{#if}` branch. The cleanup-then-resubscribe path is what the
   runtime would do across a route navigation too.
