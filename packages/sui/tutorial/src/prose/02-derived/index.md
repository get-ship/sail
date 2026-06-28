---
title: "Derived"
description: "Memoized computations over signals."
group: "Tutorial"
order: 2
slug: "02-derived"
prev: "01-hello-signal"
next: "03-bindfor"
chapter: "reactivity"
---

A **derived** is a computed value over one or more signals. It
re-computes when any of its tracked dependencies change, and only
notifies downstream subscribers when its own output actually moves.

```ts
import { signal, derived } from '@ship/sui-runtime'

const count = signal(0)
const doubled = derived(() => count() * 2)
doubled()  // → 0
count.set(5)
doubled()  // → 10
```

A few things to know:

- **Lazy.** A derived doesn't run until something reads it. Multiple
  consecutive `set()` calls on its sources only trigger one re-compute.
- **Deduplicated.** If the derived returns an `Object.is`-equal value
  to its previous, downstream subscribers don't notify. Cheap fix for
  the common "derived returns the same array reference" case.
- **Read-only.** Calling `.set()` on a derived throws. You write to
  the source signals, not the derived.

## Try it

The example uses `count` and `name` signals plus two deriveds:
`doubled` and `greeting`. Try:

1. **Chain a derived.** Add a `loudGreeting` derived that returns
   `greeting().toUpperCase()`. Bind it in the template.
2. **Notice the recompute.** Open devtools and add
   `console.log('derived ran')` inside `doubled`'s factory. Click
   `+1` a few times — the log fires per click. Now change `name`
   instead — `doubled`'s factory does NOT fire, because nothing it
   reads changed.

Show solution adds the `loudGreeting` derived and a binding.
