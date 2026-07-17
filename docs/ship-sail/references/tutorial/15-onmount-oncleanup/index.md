---
title: "Lifecycle — mount and onCleanup"
description: "The controller function body runs at setup time; mount(fn) runs once the DOM connects; onCleanup(fn) runs on dispose."
group: "Tutorial"
order: 15
slug: "15-onmount-oncleanup"
prev: "14-class-style"
next: "16-effect-cleanup"
chapter: "lifecycle"
---

SAIL's lifecycle model is shorter than most frameworks':

| Frame | Where it lives |
|---|---|
| **Setup** | The body of `controller((ctx) => { … })` — runs once when the controller is instantiated, before the DOM is necessarily connected. |
| **Mount** | Any `mount((el) => { … })` callback you register during setup — runs once after this instance's DOM connects. |
| **Unmount** | Any `onCleanup(fn)` callback you registered, or a cleanup function returned from `mount(fn)`. Also: any cleanup function returned from an `effect()`. |
| **Update** | Effects re-fire on signal change; no other update lifecycle exists. |

`mount` and `onCleanup` are both ambient — call them from anywhere in
the setup call graph (the controller body itself, or a plain helper
function it calls) with no import statement. `mount` gets you the
connected host element and a chance to run DOM-dependent setup;
`onCleanup` is for anything that doesn't need to wait for a
connected DOM.

```ts
export default controller((ctx) => {
  const count = signal(0)

  // Runs at setup — before the DOM is necessarily connected.
  console.log('controller instantiated')
  const start = performance.now()

  // Runs once the DOM connects.
  mount((el) => {
    console.log('view attached', el)

    const id = window.setInterval(() => count.set(count() + 1), 1000)

    // Runs at unmount — the cleanup mount() returns.
    return () => {
      window.clearInterval(id)
      console.log(`view unmounted after ${performance.now() - start}ms`)
    }
  })

  return { count }
})
```

Three things to know:

- **`onCleanup` and `mount`'s returned cleanup are scope-bound.**
  They register against the current instance's scope. Inside an
  effect, `onCleanup(fn)` is identical to returning `fn` from the
  effect body.
- **Cleanup order is reverse of registration.** Last-registered
  cleanup fires first. Important for nested resources (close child
  before parent).
- **Throws in cleanup are swallowed + logged.** A failing cleanup
  doesn't block other cleanups from running.

## Try it

The example wires a clock that ticks every second and logs its
lifetime on unmount. Try:

1. **Add a pause toggle**: a `paused` signal that when true makes
   the interval skip — but the interval itself stays registered.
   Compare with stopping/restarting the interval entirely (recreate
   it inside an effect that reads `paused()`).
2. **Two cleanups, observe order**: register an `onCleanup` alongside
   the `mount` cleanup, log "first" and "second". Toggle the view and
   verify the later-registered one logs first.
3. **Cleanup throws**: register an `onCleanup` that throws. Verify
   later cleanups still run.
