---
title: "Controller hooks"
description: "Reusable bundles of state + derived + handlers, called directly inside the controller."
group: "Tutorial"
order: 20
slug: "20-controller-hooks"
prev: "19-template-hooks"
next: "21-forms"
chapter: "composition"
---

A **controller hook** is a reusable slice — state + derived + action
handlers — bundled into a plain function. There's no registration, no
namespace key: call it inside the controller and merge whatever it
returns into the controller's own returned scope however you choose.

```ts
import { controller, signal, filteredList, selection } from '@ship/sail-runtime'

const AGENTS = [
  { id: 'planner',  label: 'Planner — long-horizon work breakdown' },
  { id: 'reviewer', label: 'Reviewer — second-opinion code review' },
  { id: 'qa',       label: 'QA — Playwright + screenshot suite' },
]

export default controller((ctx) => {
  const items = signal(AGENTS)

  const list = filteredList(
    () => AGENTS,
    (a, q) => a.label.toLowerCase().includes(q.toLowerCase()),
  )
  const sel = selection(() => AGENTS, (a) => a.id)

  return { items, list, sel }
})
```

Reading them in the template. Each hook's return value lands under
whatever key you gave it in the returned object — `list` contributes
`list.query` / `list.filtered()`, `sel` contributes `sel.selected()`
and friends:

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
| `filteredList(source, predicate)` | `query`, `filtered`, `setQuery`, `clear` |
| `selection(source, getKey)` | `selectedId`, `selected`, `select`, `clear`, `toggle`, `isSelected` |
| `form({ initial, validate, submit })` | `fields`, `values`, `errors`, `valid`, `dirty`, `pending`, `reset`, `submit` (covered in step 21) |

You can author your own — a controller hook is nothing more than a
plain function built from `signal(...)` and `derived(...)` that
returns an object of signals and helpers. There's no type to implement
and no registration: call it inside the controller, and keep the
returned object however you like — nested under a key (as `list` and
`sel` are above), or spread flat if there's no risk of collision.

## Try it

The example combines `filteredList` + `selection` over a small
agent catalog. Try:

1. **Filter**: type in the input — the list narrows reactively. The
   filter is a `derived` over `list.query`, so every keystroke
   re-evaluates the predicate.
2. **Select**: click a row — `sel.selected()` becomes that item, the
   row highlights. Click again — `sel.toggle()` clears the selection.
3. **Author a hook**: write your own `toggle(initial = false)` that
   returns `{ on, toggle }` — an `on` signal and a handler that calls
   `on.set((v) => !v)`. Call it inside the controller, keep the
   result under a key (say, `const flag = toggle()`), and read
   `flag.on()` in the template. The shape mirrors `selection`: a
   function you call, its return value merged in by hand — no section,
   no namespace, nothing to register.
