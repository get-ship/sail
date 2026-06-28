---
title: "{#while-loading} and {#while-error}"
description: "Envelope-aware blocks that unpack `ok`/`loading`/`error` for you."
group: "Tutorial"
order: 23
slug: "23-while-loading"
prev: "22-data-live"
next: "24-morphs"
chapter: "views"
---

Step 22 rendered a live envelope via three `{#if}` branches keyed on
`users().kind`. That works but it's noisy. `{#while-loading}` and
`{#while-error}` are sugar for the loading / error branches; an
optional `{#if .kind === 'ok'}` handles the success body. The
syntax is:

```sui
<data name="users" source="ship.users.list()" live />

{#while-loading of users}
  <p class="muted">loading…</p>
{/while-loading}

{#while-error err of users}
  <p class="err">{err.message}</p>
{/while-error}

{#if users().kind === 'ok'}
  <ul>{#for u of users().value.items key={u.id}}<li>{u.name}</li>{/for}</ul>
{/if}
```

Three things to know:

- **Blocks are boundary-mount.** When the envelope is `loading`, the
  loading block mounts; when it transitions to `ok`, the loading
  block unmounts cleanly (effects torn down, refs cleared). Same for
  `error`.
- **`err` is scoped.** Inside `{#while-error err of …}`, `err` is the
  unwrapped `{ message }` value — no need to write
  `users().error.message`.
- **They compose with normal blocks.** You still need `{#if users().kind
  === 'ok'}` for the success body. The runtime doesn't auto-render
  the `ok` envelope's value — that's your call (you might want to show
  a stale `ok` value during a re-fetch, for example).

## Try it

The example uses the same SSE scenario from step 22, but with the
cleaner block helpers in place of the `{#if .kind === …}` chain.

1. **Swap back to `{#if}`**: rewrite as an `{#if .kind === 'loading'} …
   {:else if .kind === 'error'} … {:else}` chain. Compare the
   readability.
2. **Show stale during re-fetch**: drop the `{#while-loading}` block
   entirely — when the envelope flips back to `loading` after an `ok`,
   the `{#if .kind === 'ok'}` branch stays empty. Add a
   `{#if users().kind !== 'error'}` wrapper that keeps the last `ok`
   value visible during refresh.
3. **Add a refresh button**: remount the `<data>` directive by toggling
   a parent `{#if shown()}` — the live subscription re-runs, the
   loading state fires again, the SSE reconnects.
