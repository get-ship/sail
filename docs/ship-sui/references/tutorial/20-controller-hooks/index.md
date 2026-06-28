---
title: "Controller hooks — use: section"
description: "Reusable bundles of state + derived + handlers."
group: "Tutorial"
order: 20
slug: "20-controller-hooks"
prev: "19-template-hooks"
next: "21-forms"
chapter: "composition"
---

A **controller hook** is a reusable slice — state + derived + action
handlers — composed into a controller's `use:` section, namespaced
by a key.

```ts
import { controller, signal, useFilteredList, useSelection } from '@ship/sui-runtime'

const AGENTS = [
  { id: 'planner',  label: 'Planner — long-horizon work breakdown' },
  { id: 'reviewer', label: 'Reviewer — second-opinion code review' },
  { id: 'qa',       label: 'QA — Playwright + screenshot suite' },
]

export default controller({
  scope: () => ({ items: signal(AGENTS) }),
  use: {
    list: useFilteredList(
      () => AGENTS,
      (a, q) => a.label.toLowerCase().includes(q.toLowerCase()),
    ),
    sel:  useSelection(() => AGENTS, (a) => a.id),
  },
})
```

Reading them in the template. Hooks contribute their `state:` and
`derived:` keys namespaced under the `use:` key; `on:` handlers
become event handler refs:

```sui
<input bind:value={list.query} placeholder="filter…" />

{#for item of list.filtered() key={item.id}}
  <li
    class:selected={sel.selected()?.id === item.id}
    on:click={() => sel.toggle(item.id)}
  >
    {item.label}
  </li>
{/for}

<p>showing {list.filtered().length} of {items().length}</p>
```

`sel.selected()` is the derived current item (`T | null`). The
`?.id === item.id` chain keeps the comparison safe when nothing is
selected — the `?.` returns `undefined`, the `===` is false, the row
isn't highlighted.

Hooks ship today:

| Hook | What it bundles |
|---|---|
| `useFilteredList(source, predicate)` | `state.query`, `derived.filtered`, `on.setQuery`, `on.clear` |
| `useSelection(source, getKey)` | `state.selectedId`, `derived.selected`, `on.select`, `on.clear`, `on.toggle` |
| `useForm(initial, validate)` | `state.values`, `state.errors`, `state.touched`, `on.set`, `on.touch`, `on.submit`, `derived.valid` (covered in step 21) |

You can author your own — anything matching `ControllerHook<Scope>`
shape: `{ state?, derived?, on? }`. Each section is merged into the
scope under the namespace key.

## Try it

The example combines `useFilteredList` + `useSelection` over a small
agent catalog. Try:

1. **Filter**: type in the input — the list narrows reactively. The
   filter is a `derived` over `list.query`, so every keystroke
   re-evaluates the predicate.
2. **Select**: click a row — `sel.selected()` becomes that item, the
   row highlights. Click again — `sel.toggle()` clears the selection.
3. **Author a hook**: write your own `useToggle()` that bundles
   `{ state: { on: false }, on: { toggle: (s) => { s.on = !s.on } } }`.
   Add it to `use:` and read `toggle.on()` in the template. The
   shape mirrors `useSelection` — three sections, each optional,
   namespaced under the `use:` key.
