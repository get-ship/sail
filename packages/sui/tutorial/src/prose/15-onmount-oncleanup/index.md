---
title: "Lifecycle — there is no onMount"
description: "scope() is mount. onCleanup is unmount. Effects own the rest."
group: "Tutorial"
order: 15
slug: "15-onmount-oncleanup"
prev: "14-class-style"
next: "16-effect-cleanup"
chapter: "lifecycle"
---

SUI's lifecycle model is shorter than most frameworks':

| Frame | Where it lives |
|---|---|
| **Mount** | The body of `scope: () => { … }` — runs once when the view attaches. |
| **Unmount** | Any `onCleanup(fn)` callback you registered during scope. Also: any cleanup function returned from an `effect()`. |
| **Update** | Effects re-fire on signal change; no other update lifecycle exists. |

There's no `onMount`. The reason: if you'd reach for `onMount` to run
something on mount, you can just write it directly in `scope()` —
that body **is** the mount. You only need an explicit cleanup
because you can't know when the unmount will fire.

```ts
import { signal, effect, onCleanup } from '@ship/sui-runtime'

export default controller({
  scope: () => {
    const count = signal(0)

    // Runs at mount — this IS the mount.
    console.log('view attaching')
    const start = performance.now()

    // Runs at unmount.
    onCleanup(() => {
      console.log(`view unmounted after ${performance.now() - start}ms`)
    })

    // Effect with its own cleanup — fires every time a signal it
    // reads changes, cleanup runs before the next firing AND at
    // unmount.
    effect(() => {
      const id = window.setInterval(() => count.set(count() + 1), 1000)
      return () => window.clearInterval(id)
    })

    return { count }
  },
})
```

Three things to know:

- **`onCleanup` is scope-bound.** It registers against the current
  scope (the controller's, or the nearest effect's). Inside an
  effect, `onCleanup(fn)` is identical to returning `fn` from the
  effect body.
- **Cleanup order is reverse of registration.** Last `onCleanup`
  fires first. Important for nested resources (close child before
  parent).
- **Throws in cleanup are swallowed + logged.** A failing cleanup
  doesn't block other cleanups from running.

## Try it

The example wires a clock that ticks every second and logs its
lifetime on unmount. Try:

1. **Add a pause toggle**: a `paused` signal that when true makes
   the interval skip — but the interval itself stays registered.
   Compare with stopping/restarting the interval entirely (recreate
   it inside an effect that reads `paused()`).
2. **Two cleanups, observe order**: register two `onCleanup` calls
   that log "first" and "second". Toggle the view and verify
   "second" logs before "first".
3. **Cleanup throws**: register an `onCleanup` that throws. Verify
   later cleanups still run.
