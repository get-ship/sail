---
group: Routes
order: 10
title: Effect scopes — cleanup ownership
description: The effect-scope primitive that releases a route's resources (EventSource, observers, timers, hook subscriptions) on unmount.
audience: internal
---

# Effect scopes — cleanup ownership

Resources allocated while a route's controller runs — `EventSource`,
`MutationObserver`, `setInterval`, custom hook subscriptions — need
to be released when the route unmounts. The runtime ships an effect
scope primitive that handles this for you, so you can't accidentally
leak. (Forgotten cleanups were the root cause of multiple OOM
crashes during early shell development; commit `87cf3223`.)

## API

`onCleanup` is ambient — the compiler auto-injects it inside a
`<controller>` block, no import statement needed.

`onCleanup(fn)` registers a teardown callback with the *current*
scope. Called anywhere in the controller function's call graph
(directly in the controller body, inside `mount`, or inside a plain
helper it calls), the current scope is the mount's scope, so
registered cleanups run when the user navigates away. It's ambient —
no import needed, just call the bare identifier.

```ts
export default controller(() => {
  const ws = new WebSocket('wss://server/feed')
  onCleanup(() => ws.close())
  return { /* signals … */ }
})
```

Outside a `withScope` block (top-level module init, tests, SSR
shims), `onCleanup` is a silent no-op. Safe to call defensively from
any allocator.

## When you DON'T need it

- `signal()` and `derived()` — auto-disposed when their last reader
  goes away. No cleanup required.
- `effect()` — already returns a stop fn the framework calls on
  dispose.
- Resources allocated inside the controller's `mount((el) => …)` and
  released via `return () => …`. The mount lifecycle wires that
  cleanup automatically.

You DO need `onCleanup` for:
- Allocator helpers like `query()`, `data()` that open subscriptions
  on the caller's behalf (a `query()` whose client method returns a
  `LiveSource` holds an open channel until disposed)
- Custom hooks that wrap browser APIs (intersection observers, web
  socket clients, broadcast channels)
- One-off allocations inside the controller function that don't
  return a disposer to the framework

## Custom scopes

Most callers stick with `onCleanup`. Two cases want explicit scopes:

### `createScope()` — host primitive

If you're authoring a HOST that wants to own a scope (the only
in-tree example is `mount.ts`), call `createScope()` and run user
code inside `withScope(scope, () => userCode())`:

```ts
import { createScope, withScope } from '@ship/sail-runtime'

const scope = createScope()
withScope(scope, () => userCode())
// … later
scope.dispose() // runs every cleanup userCode registered
```

### `createChildScope()` — sub-lifecycle

When part of your route has its own teardown that's independent of
the outer mount but should still get cleaned up if the mount goes
away first, create a child scope:

```ts
import { createChildScope, withScope } from '@ship/sail-runtime'

const child = createChildScope() // implicitly owned by the current scope
withScope(child, () => {
  // every onCleanup here registers on `child`
})
// `child.dispose()` releases just this sub-lifecycle
// `outer scope dispose` also disposes `child` automatically
```

## Dev-mode leak detection

Wire the leak detector during development to catch missing cleanups
before they ship:

```ts
import { enableLeakWarn } from '@ship/sail-runtime/dev'

// Top of your app entry, behind a dev-only branch:
if (import.meta.env?.DEV) enableLeakWarn()
```

The detector wraps `EventSource`, `MutationObserver`, `ResizeObserver`,
`IntersectionObserver`, `setInterval`, and `clearInterval`. When any
scope disposes with one of those resources still open, you get a
`console.warn` naming the kind of resource and a sample stack trace.

The wrappers come down via the disposer `enableLeakWarn` returns, or
the ref-counted shorthand `enableLeakWarnInScope()` which auto-removes
when the current scope ends. Production bundles that don't import
from `/dev` don't pay the wrapper cost.

## Dispose ordering

Cleanups run in **reverse insertion order** — last registered runs
first. This matches the build-up / tear-down direction most people
intuit (the most recent thing you set up is the first you tear
down).

A throwing cleanup is logged and the others still run. No partial
teardown.
