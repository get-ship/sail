---
title: "Signals"
description: "The reactive cell at the heart of SUI."
group: "Tutorial"
order: 1
slug: "01-hello-signal"
prev: null
next: "02-derived"
chapter: "reactivity"
---

A **signal** is the smallest reactive primitive in SUI — a callable
cell that notifies its subscribers when it changes.

```ts
import { signal } from '@ship/sail-runtime'

const count = signal(0)
count()          // → 0     (read)
count.set(1)     // notify subscribers
count.set(c => c + 1)  // updater form
```

Anything that reads a signal inside a tracked scope (a `derived`, an
`effect`, or a template binding like `{count()}`) is automatically
subscribed. When the signal changes, only those readers re-run — no
virtual DOM, no diffing, no component re-render.

## Try it

The example to the right declares a `count` signal in its
controller and reads it in the template's `{count()}` binding. The
two buttons call `count.set(c => c ± 1)` to push new values.

Open the `<controller>` block in `sui-app.sui` and try:

1. **Add a reset button.** Add a `reset` handler that sets the
   count to `0` and return it from the controller function, then
   wire it to a new button in the template below.
2. **Watch the binding.** Notice that updating `count` only
   re-runs the text node bound to `{count()}` — nothing else in
   the template re-renders.
3. **Read the controller.** The signal is created once, when the
   controller function runs. Every render of this component re-uses
   the same signal — that's how state survives subsequent updates.

When you're stuck, click **Show solution** to see one valid answer.
The "Reset" button isn't the only correct shape — anything that
calls `count.set(0)` works.
