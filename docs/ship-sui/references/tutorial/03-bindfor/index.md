---
title: "Lists with {#for}"
description: "Keyed list reconciliation."
group: "Tutorial"
order: 3
slug: "03-bindfor"
prev: "02-derived"
next: "04-effect"
chapter: "reactivity"
---

`{#for}` renders a reactive array as a sequence of DOM nodes. Each
row is keyed — the runtime uses the key to track identity across
reorders, inserts, and removes, so focus, scroll, and animation
state survive any list change.

```sui
{#for item of items() key={item.id}}
  <li>{item.label}</li>
{/for}
```

Two things make the keyed path fast in SUI:

- **Monotone fast-path.** When the matched-prev positions are
  strictly increasing — append, prepend, single insert/remove — the
  reconciler skips the O(n log n) LIS scan entirely.
- **Per-field reactivity.** A row body that reads `row.label`
  reactively only re-fires when `row.label` actually changes — not
  when a sibling row's label changes, and not when an unrelated
  field on the same row changes.

## Try it

The example renders a `todos` signal as a `<ul>`. Try:

1. **Add a new todo.** Wire the input + button to push a new
   `{ id: nextId++, label: input.value, done: false }` into `todos`.
2. **Toggle done.** Click a todo to flip its `done` flag. Notice
   the keyed list keeps DOM identity — your click handler still
   targets the same `<li>` element across re-renders.
3. **Reorder.** Add a "shuffle" button that calls `todos.set(...)`
   with a randomized copy. Watch the list update without re-rendering
   any row's content.

Show solution implements `add` and `toggle`.
