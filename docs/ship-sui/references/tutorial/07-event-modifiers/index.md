---
title: "Event modifiers"
description: "preventDefault, stopPropagation, key filters, debounce."
group: "Tutorial"
order: 7
slug: "07-event-modifiers"
prev: "06-bind-value"
next: "08-sui-primitives"
chapter: "templating"
---

`on:event` attributes accept pipe-separated modifiers that tweak how
the handler runs or what it receives — saving you a line of
`e.preventDefault()` and friends inside every handler body.

```sui
<form on:submit|preventDefault={handle}>…</form>
<div on:click|stopPropagation={onCardClick}>…</div>
<div on:click|self={() => dismiss()}>…</div>
<input on:keydown|key:Enter={submit} />
```

The modifiers SUI ships out of the box:

| Modifier | What it does |
|---|---|
| `preventDefault` (or `prevent`) | Calls `e.preventDefault()` before your handler runs |
| `stopPropagation` (or `stop`) | Calls `e.stopPropagation()` before your handler runs |
| `stopImmediatePropagation` | Same, and skips remaining listeners on this element |
| `self` | Only fires when `e.target === e.currentTarget` |
| `trusted` | Only fires when `e.isTrusted` (real user input, not scripted) |
| `key:<KeyName>` | Filters keyboard events to a specific `e.key` value |
| `debounce:<ms>` | Trailing-edge debounce; suppresses calls within the window |
| `throttle:<ms>` | Leading-edge throttle; one call per window |

Modifiers compose: `on:click|preventDefault|stopPropagation` gives
you both. They're parsed at compile time — typos like
`on:click|prevenDefault` are a compile error, not a silent no-op.

## Try it

The chat input fires on Enter via `on:keydown|key:Enter`. Try:

1. **Click-outside to dismiss.** The starter has an outer card; wrap
   it in `on:click|self={() => dismiss()}` so clicking the card
   (but not its children) clears the draft.
2. **Escape closes too.** Add `on:keydown|key:Escape={() => dismiss()}`
   on the input so pressing escape exits without saving.
3. **Debounce a search.** Add `on:input|debounce:250={onSearch}` to
   another input — observe that rapid typing only fires `onSearch`
   once the user pauses for 250ms.
