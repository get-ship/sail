---
title: "Templates react"
description: "Zero-arg reads, event modifiers, and the control-flow blocks — {#if}, {#for key}, {#match}."
order: 4
slug: "04-templates-react"
prev: "03-state-is-a-controller"
next: "05-data-and-forms"
chapter: "state"
---

A template is a **read** of state; handlers **write**. That separation
is the whole mental model, and the compiler enforces the reading half:
a renderable expression — text like `{phase()}`, an attribute binding
like `disabled={phase() === 'building'}` — may only make **zero-arg
calls**. Write `{format(date())}` in markup and the build stops:

```
call `format(…)` with arguments is not allowed in renderable
expressions; only zero-arg signal reads are permitted
```

Work belongs in the controller, exposed as a derived. Handlers are the
exception — `on:click={() => promote(env.id)}` can call anything,
because a handler is a write path, not a render path.

## Modifiers declare intent

```
<input on:input|debounce:300={search}
       on:keydown|key:Escape={clear} />
<form on:submit|prevent={save}>
```

Debouncing, key filtering, and preventDefault are part of the
*declaration*, not boilerplate inside the handler.

## The blocks

- `{#if cond}…{:else}…{/if}` — branch on a read.
- `{#match expr}` with `{:case 'x'}` / `{:default}` — a state machine
  rendered as one construct. The example's deploy phase walks
  idle → building → live/failed, and each branch's badge carries its own
  `motion:enter`, so the swapped-in state announces itself.
- `{#for item of items() key={item.id}}` — `key=` is **required**; the
  compiler rejects a bare `{#for}`. The key is identity: on reorder the
  reconciler *moves* the existing DOM node instead of rebuilding it, so
  focus, selection, and animation state survive.

## Try it

1. **Wire deploy()** (TODO 1). Set the phase to `'building'`, then
   settle it with a `setTimeout` — `'live'` usually, `'failed'`
   sometimes. Click until you have seen every `{#match}` branch.
2. **Finish the match** (TODO 4). Add the `building` and `live` cases
   with different `motion:enter` bodies — a translate for building, a
   scale for live.
3. **Prove identity** (TODO 3 + 5). Implement `promote()` to move an
   environment to the front. With `key={env.id}` the row glides to its
   new slot as a moved node. Now remove the `key=` and read the error —
   the language would rather stop you than lose DOM identity silently.
4. **Feel the modifiers.** Type into the filter — the handler runs 300ms
   after you stop. Press Escape — the input clears without a single
   `if (e.key …)` in the controller.
