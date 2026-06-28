---
title: "untrack and batch"
description: "Two escape hatches for explicit reactivity control."
group: "Tutorial"
order: 18
slug: "18-untrack-batch"
prev: "17-selector"
next: "19-template-hooks"
chapter: "reactivity"
---

Most of the time, automatic subscription is what you want. Two
escape hatches handle the cases where it isn't.

## untrack — read without subscribing

`untrack(fn)` runs `fn` with subscription tracking turned off. Reads
inside don't add the surrounding effect/derived as a dependency.

```ts
import { signal, effect, untrack } from '@ship/sui-runtime'

const count = signal(0)
const label = signal('hits')

effect(() => {
  // Re-runs when count() changes.
  // Doesn't re-run when label() changes (read inside untrack).
  const stamp = untrack(() => Date.now())
  const prefix = untrack(() => label())
  console.log(`${prefix}: ${count()} @${stamp}`)
})
```

Use it for:
- **Timestamps, RNG, peer-peek**: values you want to consult but
  not depend on.
- **"Latest-only" snapshots** in handlers that fire from a tracked
  source (e.g. clicking the row reads `latestData()` once without
  subscribing).

## batch — coalesce multiple writes

`batch(fn)` defers subscriber notifications until `fn` returns.
Multiple writes inside a batch dispatch effects **once at the end**.

```ts
import { signal, effect, batch } from '@ship/sui-runtime'

const x = signal(0), y = signal(0)
effect(() => console.log(`${x()}, ${y()}`))   // logs once on attach

x.set(1)        // logs "1, 0"
y.set(2)        // logs "1, 2"

batch(() => {
  x.set(3)
  y.set(4)
})
// logs "3, 4" — once, not twice
```

The runtime already batches synchronously-set signals within the
same microtask. `batch()` is the explicit version for when you're
calling out to code that does multiple sets across async boundaries
or when you want to make the intent visible.

## Try it

The example combines both: a `metrics` view that updates 3 signals
on each tick, batched. Try:

1. **Drop the batch**: remove `batch(…)` and watch the effect fire
   3 times per tick (3× the log noise).
2. **Read a peer without subscribing**: inside one effect, read a
   second signal via `untrack`. Change the second signal — the
   effect should NOT re-run.
3. **Stamp without dep**: replace the `untrack(Date.now)` with a
   bare `Date.now()` call. Nothing changes — `Date.now()` isn't a
   signal so there's no dependency to track. The `untrack` wrapper
   is only relevant when you're calling a signal inside it.
