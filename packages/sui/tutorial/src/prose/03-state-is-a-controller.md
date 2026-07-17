---
title: "State is a controller"
description: "signal, derived, and effect — one owner for state, and templates bind only what the controller returns."
order: 3
slug: "03-state-is-a-controller"
prev: "02-styling-is-sx"
next: "04-templates-react"
chapter: "state"
---

Behavior lives in **one place**: the controller. Whatever the
controller function returns is the template's entire world — a
binding like `{clock()}` can only reach what the controller chose to
expose.

The example is a stopwatch, built from the three reactive primitives:

```
const elapsed = signal(0)          // a value that notifies on change
const running = signal(false)

const clock = derived(() =>       // computed, cached, lazy
  format(elapsed()))

effect(() => {                    // a side-effect that owns its cleanup
  if (!running()) return
  const id = setInterval(() => elapsed.set((ms) => ms + 100), 100)
  return () => clearInterval(id)
})
```

- A **signal** is a callable cell: `elapsed()` reads (and subscribes,
  inside any tracked scope), `elapsed.set(v)` or `elapsed.set(fn)`
  writes. Reading it in a template binding subscribes exactly that DOM
  node — no virtual DOM, no re-render.
- A **derived** recomputes from whatever it reads, only when read, and
  only notifies downstream when its output actually changes. The clock
  string is derived; nobody ever sets it.
- An **effect** is the escape hatch to the outside world — timers,
  storage, sockets. Return a function and the effect *owns its undo*:
  the cleanup runs before every re-run and when the component unmounts.
  There is no dependency array to forget; the effect subscribes to what
  it read. Reach for `effect` rarely — if it does not touch the outside
  world, you probably wanted a `derived`.

## One file

The controller lives in the `.sui` file, in a `<controller>` block above
the template:

```
<controller>
import { controller, signal } from '@ship/sui-runtime'
export default controller(() => {
  const elapsed = signal(0)
  return { elapsed }
})
</controller>

<strong>{elapsed()}</strong>
```

There is no second spelling. A sibling `<name>.controller.ts` is a
compile error: a template whose meaning depends on whether a file you
cannot see exists on disk is a template you cannot read, and it loses
its scope silently the moment that file is renamed.

When a controller grows past wiring, the answer is not another file —
it is a **composition**: a plain function that returns signals, imported
by the block.

```
<controller>
import { controller } from '@ship/sui-runtime'
import { createStopwatch } from './stopwatch'   // plain TS, plain functions

export default controller(() => createStopwatch())
</controller>
```

That is how the real components work: `<sui-tree>` imports a 400-line
table engine and its controller is barely wiring. Logic composes; the
controller stays the seam between it and the template.

## Try it

The starter's clock never ticks:

1. **Write the interval effect** (TODO 1 in the controller). Gate it on
   `running()`, bump `elapsed` every 100ms, and return the
   `clearInterval` cleanup. Flip Start/Pause and convince yourself the
   old timer dies each time the effect re-runs.
2. **Add laps** (TODO 2 + 3). A `laps` signal of `{ id, at }` records
   `clock()` at press time; render it with a keyed `{#for}`.
3. **Persist across reloads.** Add an effect that writes `elapsed()` to
   `localStorage`, and seed the signal from it. State that outlives the
   page is still just a signal plus an effect at the edge.
