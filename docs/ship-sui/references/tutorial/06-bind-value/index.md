---
title: "Two-way binding"
description: "bind:value keeps a signal and an input in sync."
group: "Tutorial"
order: 6
slug: "06-bind-value"
prev: "05-bind-if"
next: "07-event-modifiers"
chapter: "templating"
---

`bind:value={mySignal}` is shorthand for "read `mySignal()` into the
input's `value`, and call `mySignal.set(newValue)` whenever the user
edits it." Behind the scenes it's just `prop:value={signal()}` plus
an `on:input` handler — the runtime expands it for you.

```sui
<input bind:value={draft} />
```

The same shape works on `<textarea>`, `<select>`, and checkboxes
(`bind:checked`). For custom elements that fire `sui-input`, the
runtime listens to that too — `bind:value` works against the Ship
input primitives without extra wiring.

```sui
<sui-input bind:value={name} placeholder="your name" />
<p>hello, {name() || 'stranger'}</p>
```

Two-way binding is convenient but not magic. Under the hood it's still
the same one-way signal flow — the input is just a subscriber that
also pushes updates back. If you need a transform (trimming, parsing,
sanitizing), drop down to `prop:value={…}` + `on:input={…}` and do
the work explicitly.

## Try it

The form already binds `name` and `quantity`. Open the **template**
tab. Try:

1. **Add a checkbox.** Use `bind:checked={agreed}` against a `signal(false)`
   and gate the "Submit" button on `agreed()`.
2. **Add a select.** Bind a `<select bind:value={size}>` with options
   "S", "M", "L" — show the chosen size in the preview row.
3. **Reject negatives.** Notice that `quantity` is bound as a string.
   Replace the `bind:value` on quantity with explicit `prop:value` +
   `on:input` that clamps to `Math.max(0, Number(e.target.value))`.

The solution wires the checkbox and select; the quantity-clamp
challenge is left as an exercise.
