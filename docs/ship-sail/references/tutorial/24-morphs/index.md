---
title: "Optimistic mutations + server-pushed updates"
description: "POST a mutation; the server's next SSE frame updates every subscriber."
group: "Tutorial"
order: 24
slug: "24-morphs"
prev: "23-while-loading"
next: "25-going-further"
chapter: "views"
---

`<data live>` subscribes you to a stream. To **change** the data, you
POST a mutation to the server, and the server pushes a new envelope
on the same stream — every subscriber (the current tab, other tabs,
peer agents) sees it.

The shape:

```sail
<data name="users" source="ship.users.list()" live />

<form on:submit|preventDefault={(e) => add(name())}>
  <input bind:value={name} placeholder="user name" />
  <button type="submit">add user</button>
</form>

<ul>
  {#for u of users().value.items key={u.id}}<li>{u.name}</li>{/for}
</ul>
```

```ts
export default controller((ctx) => {
  const name = signal('')
  return {
    name,
    add: async (label: string): Promise<void> => {
      if (!label.trim()) return
      const res = await fetch('/_deck/api/users/add', {
        method: 'POST',
        body: JSON.stringify({ name: label }),
      })
      if (res.ok) name.set('')
    },
  }
})
```

Three things to know:

- **No client-side optimistic patching.** The controller doesn't touch
  `users()` at all. The server is the source of truth; the new
  envelope arrives on the SSE channel.
- **Re-subscribers see the latest.** Open the same page in a second
  tab — it subscribes with the current state, not the initial state.
- **For optimistic UI, layer it locally.** If you want the row to
  appear before the server confirms, hold a separate `optimisticAdds`
  signal that merges into the rendered list and clears entries when
  the next envelope confirms them.

## How the playground simulates the server

The tutorial's mock-daemon service worker accepts POST requests on
`/_deck/api/<path>` and looks up a `mutations` handler in the scenario:

```js
// apps/site/public/tutorial/scenarios/24-morphs.js
export default {
  '/users/list': {
    initial: { kind: 'ok', value: { items: [{ id: 1, name: 'alpha' }] } },
    mutations: {
      '/users/add': (value, body) => ({
        items: [...value.items, { id: Date.now(), name: body.name }],
      }),
    },
  },
}
```

The handler computes the new state; the SW pushes the new envelope
to every active SSE controller subscribed to `/users/list`.

## Try it

1. **Add a few users.** Type, submit, watch the list grow. The
   controller never mutates `users()` directly.
2. **Open two tabs**: type in one, the other updates too. (Service
   workers share state across tabs of the same origin.)
3. **Optimistic UI**: add a local `pending` signal that holds names
   you've POSTed but haven't seen confirmed yet. Render them with a
   grey opacity in the `{#for}` loop. Clear entries from `pending`
   when the next envelope contains a matching `name`.
