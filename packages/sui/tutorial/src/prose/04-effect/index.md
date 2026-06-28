---
title: "Effects"
description: "Side effects that re-run when their inputs change."
group: "Tutorial"
order: 4
slug: "04-effect"
prev: "03-bindfor"
next: "05-bind-if"
chapter: "reactivity"
---

`effect(fn)` runs `fn` once immediately, then again whenever any
signal it reads changes. It's how you reach **out of** the reactive
graph — touching the DOM directly, hitting `localStorage`, posting a
message, calling `console.log`, anything that isn't a derived value.

```ts
import { signal, effect } from '@ship/sui-runtime'

const count = signal(0)
effect(() => {
  console.log('count is', count())
})
// → "count is 0"  (runs eagerly)
count.set(1)
// → "count is 1"  (re-runs)
```

Effects run after the synchronous batch. If you set the same signal
three times in a row, the effect fires once with the final value, not
three times. The cleanup form lets you tear down an outgoing
subscription before the next run:

```ts
effect(() => {
  const id = window.setInterval(() => console.log('tick'), 1000)
  return () => window.clearInterval(id)
})
```

## Try it

The example wires a counter to `document.title`. Open the
**controller** tab. Try:

1. **Persist the count.** Add an `effect` that writes `count()` to
   `localStorage.setItem('demo:count', String(count()))`, and one
   that hydrates the initial value from `localStorage` when the
   scope factory runs.
2. **Add a cleanup.** Replace the title effect with one that owns a
   `setInterval` ticking every second. Make sure the interval is
   cleared by returning a cleanup function.
3. **Notice scope.** The effect only re-runs when signals it
   **actually read** change. Wrapping a `count()` read in an
   `if (someFlag())` makes it conditional — try toggling a `paused`
   signal that gates the body.

Click **Show solution** to see one implementation.
